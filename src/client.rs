//! QRZ.com XML API client implementation.

use crate::error::{QrzXmlError, Result};
use crate::types::{
    ApiVersion, BiographyData, CallsignInfo, DxccInfo, QrzXmlResponse, SessionInfo,
};
use crate::{DEFAULT_BASE_URL, DEFAULT_USER_AGENT};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use url::Url;

/// Configuration for the QRZ client
#[derive(Debug, Clone)]
pub struct QrzXmlClientConfig {
    /// Base URL for the QRZ XML API
    pub base_url: String,
    /// User agent string for HTTP requests
    pub user_agent: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of automatic retry attempts
    pub max_retries: u32,
}

impl Default for QrzXmlClientConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

/// Internal session state
#[derive(Debug, Clone)]
struct SessionState {
    key: Option<String>,
    count: Option<u32>,
    sub_exp: Option<String>,
}

impl SessionState {
    fn new() -> Self {
        Self {
            key: None,
            count: None,
            sub_exp: None,
        }
    }

    fn update_from_session_info(&mut self, session: &SessionInfo) {
        if let Some(key) = &session.key {
            self.key = Some(key.clone());
        }
        if let Some(count) = session.count {
            self.count = Some(count);
        }
        if let Some(sub_exp) = &session.sub_exp {
            self.sub_exp = Some(sub_exp.clone());
        }
    }

    fn has_valid_session(&self) -> bool {
        self.key.is_some()
    }

    fn clear(&mut self) {
        self.key = None;
        self.count = None;
        self.sub_exp = None;
    }
}

/// Main QRZ.com XML API client
pub struct QrzXmlClient {
    /// HTTP client
    http_client: Client,
    /// QRZ username
    username: String,
    /// QRZ password
    password: String,
    /// API version to use
    api_version: ApiVersion,
    /// Client configuration
    config: QrzXmlClientConfig,
    /// Current session state
    session: Arc<RwLock<SessionState>>,
}

impl QrzXmlClient {
    /// Create a new QRZ client with default configuration
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        api_version: ApiVersion,
    ) -> Result<Self> {
        Self::with_config(
            username,
            password,
            api_version,
            QrzXmlClientConfig::default(),
        )
    }

    /// Create a new QRZ client with custom configuration
    pub fn with_config(
        username: impl Into<String>,
        password: impl Into<String>,
        api_version: ApiVersion,
        config: QrzXmlClientConfig,
    ) -> Result<Self> {
        let http_client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self {
            http_client,
            username: username.into(),
            password: password.into(),
            api_version,
            config,
            session: Arc::new(RwLock::new(SessionState::new())),
        })
    }

    /// Perform initial authentication and establish a session
    pub async fn authenticate(&self) -> Result<()> {
        info!("Authenticating with QRZ.com");
        self.login().await?;
        Ok(())
    }

    /// Look up information for a callsign
    pub async fn lookup_callsign(&self, callsign: &str) -> Result<CallsignInfo> {
        if callsign.is_empty() {
            return Err(QrzXmlError::invalid_input("Callsign cannot be empty"));
        }

        let callsign = callsign.to_uppercase();
        debug!("Looking up callsign: {}", callsign);

        let response = match self
            .make_authenticated_request(&[("callsign", &callsign)])
            .await
        {
            Ok(resp) => resp,
            Err(QrzXmlError::SessionExpired) => {
                warn!("Session expired, re-authenticating and retrying");
                // Clear the old session first
                {
                    let mut session = self.session.write().await;
                    session.clear();
                }
                self.login().await?;
                self.make_authenticated_request(&[("callsign", &callsign)])
                    .await?
            }
            Err(e) => return Err(e),
        };

        match response.callsign {
            Some(callsign_info) => {
                info!("Successfully looked up callsign: {}", callsign_info.call);
                Ok(callsign_info)
            }
            None => {
                if let Some(error) = response.session.error {
                    if error.contains("not found") {
                        Err(QrzXmlError::callsign_not_found(callsign))
                    } else {
                        Err(QrzXmlError::api_error(error))
                    }
                } else {
                    Err(QrzXmlError::unexpected_response(
                        "No callsign data in response".to_string(),
                    ))
                }
            }
        }
    }

    /// Fetch biography/HTML data for a callsign
    pub async fn lookup_biography(&self, callsign: &str) -> Result<BiographyData> {
        if callsign.is_empty() {
            return Err(QrzXmlError::invalid_input("Callsign cannot be empty"));
        }

        let callsign = callsign.to_uppercase();
        debug!("Fetching biography for callsign: {}", callsign);

        // Biography requests return HTML instead of XML
        let html_content = self
            .make_authenticated_html_request(&[("html", &callsign)])
            .await?;

        Ok(BiographyData::new(callsign, html_content))
    }

    /// Look up DXCC entity by entity number
    pub async fn lookup_dxcc_entity(&self, entity: u32) -> Result<DxccInfo> {
        debug!("Looking up DXCC entity: {}", entity);

        let entity_str = entity.to_string();
        let response = self
            .make_authenticated_request(&[("dxcc", &entity_str)])
            .await?;

        match response.dxcc {
            Some(dxcc_info) => {
                info!(
                    "Successfully looked up DXCC entity: {} - {}",
                    entity, dxcc_info.name
                );
                Ok(dxcc_info)
            }
            None => {
                if let Some(_error) = response.session.error {
                    Err(QrzXmlError::dxcc_not_found(entity_str))
                } else {
                    Err(QrzXmlError::unexpected_response(
                        "No DXCC data in response".to_string(),
                    ))
                }
            }
        }
    }

    /// Look up DXCC entity by callsign prefix matching
    pub async fn lookup_dxcc_by_callsign(&self, callsign: &str) -> Result<DxccInfo> {
        if callsign.is_empty() {
            return Err(QrzXmlError::invalid_input("Callsign cannot be empty"));
        }

        let callsign = callsign.to_uppercase();
        debug!("Looking up DXCC entity for callsign: {}", callsign);

        let response = self
            .make_authenticated_request(&[("dxcc", &callsign)])
            .await?;

        match response.dxcc {
            Some(dxcc_info) => {
                info!(
                    "Successfully looked up DXCC entity for {}: {} - {}",
                    callsign, dxcc_info.dxcc, dxcc_info.name
                );
                Ok(dxcc_info)
            }
            None => {
                if let Some(_error) = response.session.error {
                    Err(QrzXmlError::dxcc_not_found(callsign))
                } else {
                    Err(QrzXmlError::unexpected_response(
                        "No DXCC data in response".to_string(),
                    ))
                }
            }
        }
    }

    /// Get all DXCC entities (use sparingly)
    pub async fn lookup_all_dxcc_entities(&self) -> Result<Vec<DxccInfo>> {
        warn!("Fetching all DXCC entities - use sparingly to avoid server overload");

        let _response = self.make_authenticated_request(&[("dxcc", "all")]).await?;

        // The "all" response returns multiple DXCC records
        // This is a bit tricky to handle with our current structure
        // For now, we'll return an error suggesting to use the individual lookup methods
        Err(QrzXmlError::invalid_input(
            "Bulk DXCC lookup not yet implemented - use individual entity lookups".to_string(),
        ))
    }

    /// Get current session information
    pub async fn session_info(&self) -> Option<(Option<u32>, Option<String>)> {
        let session = self.session.read().await;
        Some((session.count, session.sub_exp.clone()))
    }

    /// Check if currently authenticated
    pub async fn is_authenticated(&self) -> bool {
        let session = self.session.read().await;
        session.has_valid_session()
    }

    /// Force re-authentication (clears current session)
    pub async fn reauthenticate(&self) -> Result<()> {
        {
            let mut session = self.session.write().await;
            session.clear();
        }
        self.authenticate().await
    }

    /// Internal method to perform login
    async fn login(&self) -> Result<SessionInfo> {
        let url = self.build_url("")?;

        let params = [
            ("username", self.username.as_str()),
            ("password", self.password.as_str()),
            ("agent", &self.config.user_agent),
        ];

        debug!("Performing login to QRZ.com");
        let response = self.make_request(&url, &params).await?;

        let session_info = response.session.clone();

        if let Some(error) = &session_info.error {
            if error.contains("Connection refused") {
                return Err(QrzXmlError::ConnectionRefused);
            } else if error.contains("password") || error.contains("username") {
                return Err(QrzXmlError::auth_failed(error.clone()));
            } else {
                return Err(QrzXmlError::api_error(error.clone()));
            }
        }

        if !session_info.has_valid_session() {
            return Err(QrzXmlError::NoSessionKey);
        }

        // Update our internal session state
        {
            let mut session = self.session.write().await;
            session.update_from_session_info(&session_info);
        }

        info!("Successfully authenticated with QRZ.com");
        Ok(session_info)
    }

    /// Make an authenticated request that returns XML
    async fn make_authenticated_request(&self, params: &[(&str, &str)]) -> Result<QrzXmlResponse> {
        let session_key = {
            let session = self.session.read().await;
            session.key.clone()
        };

        let session_key = match session_key {
            Some(key) => key,
            None => {
                // Need to authenticate first
                self.login().await?;
                let session = self.session.read().await;
                session.key.clone().ok_or(QrzXmlError::NoSessionKey)?
            }
        };

        let url = self.build_url("")?;
        let mut all_params = vec![("s", session_key.as_str())];
        all_params.extend_from_slice(params);

        let response = self.make_request(&url, &all_params).await?;

        // Update session info from response
        {
            let mut session = self.session.write().await;
            session.update_from_session_info(&response.session);
        }

        // Check for session-related errors
        if let Some(error) = &response.session.error {
            if error.contains("Session Timeout") || error.contains("session") {
                return Err(QrzXmlError::SessionExpired);
            }
            if error.contains("not found") {
                // This is handled by the calling function
            }
            // Other errors are returned as API errors
            if !error.contains("not found") {
                return Err(QrzXmlError::api_error(error.clone()));
            }
        }

        // Check if we have a valid session key in response
        if !response.session.has_valid_session() {
            return Err(QrzXmlError::SessionExpired);
        }

        Ok(response)
    }

    /// Make an authenticated request that returns HTML (for biography)
    async fn make_authenticated_html_request(&self, params: &[(&str, &str)]) -> Result<String> {
        let session_key = {
            let session = self.session.read().await;
            session.key.clone()
        };

        let session_key = match session_key {
            Some(key) => key,
            None => {
                // Need to authenticate first
                self.login().await?;
                let session = self.session.read().await;
                session.key.clone().ok_or(QrzXmlError::NoSessionKey)?
            }
        };

        let url = self.build_url("")?;
        let mut all_params = vec![("s", session_key.as_str())];
        all_params.extend_from_slice(params);

        let query_string = all_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let full_url = format!("{}?{}", url, query_string);

        debug!("Making HTML request to: {}", full_url);

        let response = self
            .http_client
            .get(&full_url)
            .send()
            .await?
            .error_for_status()?;

        let html_content = response.text().await?;

        // Check if the response looks like an error (starts with XML)
        if html_content.trim_start().starts_with("<?xml") {
            // This might be an error response, try to parse as XML
            match quick_xml::de::from_str::<QrzXmlResponse>(&html_content) {
                Ok(xml_resp) => {
                    if let Some(error) = xml_resp.session.error {
                        return Err(QrzXmlError::api_error(error));
                    }
                }
                Err(_) => {
                    // Not valid XML, just return the HTML as-is
                }
            }
        }

        Ok(html_content)
    }

    /// Make a raw HTTP request and parse XML response
    async fn make_request(&self, url: &str, params: &[(&str, &str)]) -> Result<QrzXmlResponse> {
        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let full_url = if query_string.is_empty() {
            url.to_string()
        } else {
            format!("{}?{}", url, query_string)
        };

        debug!("Making request to: {}", full_url);

        let response = self
            .http_client
            .get(&full_url)
            .send()
            .await?
            .error_for_status()?;

        let xml_content = response.text().await?;
        debug!("Received XML response: {}", xml_content);

        let parsed_response: QrzXmlResponse =
            quick_xml::de::from_str(&xml_content).map_err(|e| {
                warn!("Failed to parse XML response: {}", e);
                warn!("Response content: {}", xml_content);
                e
            })?;

        Ok(parsed_response)
    }

    /// Build URL for API requests
    pub fn build_url(&self, path: &str) -> Result<String> {
        let mut url = Url::parse(&self.config.base_url)?;

        // Ensure the base URL ends with a slash");

        // Add version path if not legacy
        match &self.api_version {
            ApiVersion::Legacy => {}
            ApiVersion::Current => {
                url = url.join("xml/current/")?;
            }
            ApiVersion::Specific(version) => {
                let version_path = format!("xml/{}/", version);
                url = url.join(&version_path)?;
            }
        }

        if !path.is_empty() {
            url = url.join(path)?;
        }
        println!("Building URL for QRZ API: {}", url);
        Ok(url.to_string())
    }
}

// Add a helper trait for URL encoding
mod urlencoding {
    pub fn encode(input: &str) -> String {
        url::form_urlencoded::byte_serialize(input.as_bytes()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = QrzXmlClient::new("test", "test", ApiVersion::Current);
        assert!(client.is_ok());
    }

    #[test]
    fn test_url_building() {
        let config = QrzXmlClientConfig::default();
        let client =
            QrzXmlClient::with_config("test", "test", ApiVersion::Current, config).unwrap();

        let url = client.build_url("").unwrap();
        assert!(url.contains("current"));

        let client = QrzXmlClient::with_config(
            "test",
            "test",
            ApiVersion::Legacy,
            QrzXmlClientConfig::default(),
        )
        .unwrap();
        let url = client.build_url("").unwrap();
        assert!(!url.contains("current"));
        assert_eq!(url, "https://xmldata.qrz.com/xml");
    }

    #[test]
    fn test_session_state() {
        let mut session = SessionState::new();
        assert!(!session.has_valid_session());

        let session_info = SessionInfo {
            key: Some("test_key".to_string()),
            count: Some(42),
            sub_exp: Some("test_exp".to_string()),
            gm_time: None,
            message: None,
            error: None,
        };

        session.update_from_session_info(&session_info);
        assert!(session.has_valid_session());
        assert_eq!(session.key, Some("test_key".to_string()));
        assert_eq!(session.count, Some(42));
    }
}
