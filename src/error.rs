//! Error types for the QRZ client library.

use thiserror::Error;

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, QrzXmlError>;

/// Comprehensive error type for all QRZ API operations
#[derive(Error, Debug)]
pub enum QrzXmlError {
    /// Network or HTTP-related errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// XML parsing errors
    #[error("XML parsing error: {0}")]
    XmlParsing(#[from] quick_xml::DeError),

    /// URL parsing errors
    #[error("URL parsing error: {0}")]
    UrlParsing(#[from] url::ParseError),

    /// QRZ API returned an error message
    #[error("QRZ API error: {message}")]
    ApiError { message: String },

    /// Authentication failed
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    /// Session expired or invalid
    #[error("Session expired or invalid - re-authentication required")]
    SessionExpired,

    /// Callsign not found
    #[error("Callsign not found: {callsign}")]
    CallsignNotFound { callsign: String },

    /// DXCC entity not found
    #[error("DXCC entity not found: {entity}")]
    DxccNotFound { entity: String },

    /// Invalid input provided
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// QRZ service is refusing connections
    #[error("QRZ service is refusing connections - try again in 24 hours")]
    ConnectionRefused,

    /// Subscription required for this operation
    #[error("A subscription is required to access this data")]
    SubscriptionRequired,

    /// Rate limit exceeded
    #[error("Rate limit exceeded - too many requests")]
    RateLimitExceeded,

    /// No session key present in response
    #[error("No session key received - authentication may have failed")]
    NoSessionKey,

    /// Invalid API version specified
    #[error("Invalid API version: {version}")]
    InvalidApiVersion { version: String },

    /// Generic API error for unexpected responses
    #[error("Unexpected API response: {message}")]
    UnexpectedResponse { message: String },
}

impl QrzXmlError {
    /// Create a new API error
    pub fn api_error(message: impl Into<String>) -> Self {
        Self::ApiError {
            message: message.into(),
        }
    }

    /// Create a new authentication error
    pub fn auth_failed(reason: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            reason: reason.into(),
        }
    }

    /// Create a new callsign not found error
    pub fn callsign_not_found(callsign: impl Into<String>) -> Self {
        Self::CallsignNotFound {
            callsign: callsign.into(),
        }
    }

    /// Create a new DXCC not found error
    pub fn dxcc_not_found(entity: impl Into<String>) -> Self {
        Self::DxccNotFound {
            entity: entity.into(),
        }
    }

    /// Create a new invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    /// Create a new unexpected response error
    pub fn unexpected_response(message: impl Into<String>) -> Self {
        Self::UnexpectedResponse {
            message: message.into(),
        }
    }

    /// Check if this error indicates we should retry with authentication
    pub fn should_reauthenticate(&self) -> bool {
        matches!(
            self,
            QrzXmlError::SessionExpired | QrzXmlError::NoSessionKey
        )
    }

    /// Check if this error is retryable (temporary)
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            QrzXmlError::Network(_) | QrzXmlError::SessionExpired | QrzXmlError::RateLimitExceeded
        )
    }

    /// Check if this error is due to insufficient permissions/subscription
    pub fn is_permission_error(&self) -> bool {
        matches!(
            self,
            QrzXmlError::SubscriptionRequired | QrzXmlError::ConnectionRefused
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_construction() {
        let error = QrzXmlError::api_error("test message");
        assert!(error.to_string().contains("test message"));

        let error = QrzXmlError::callsign_not_found("TEST");
        assert!(error.to_string().contains("TEST"));
    }

    #[test]
    fn test_error_properties() {
        assert!(QrzXmlError::SessionExpired.should_reauthenticate());
        assert!(QrzXmlError::RateLimitExceeded.is_retryable());
        assert!(QrzXmlError::SubscriptionRequired.is_permission_error());
        assert!(!QrzXmlError::CallsignNotFound {
            callsign: "TEST".to_string()
        }
        .is_retryable());
    }
}
