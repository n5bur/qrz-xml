//! Example demonstrating XDG-compliant session token storage
//!
//! This example shows how to persist QRZ session tokens using XDG Base Directory
//! specification for cross-platform cache storage.
//!
//! Usage:
//! ```
//! QRZ_USERNAME=your_username QRZ_PASSWORD=your_password cargo run --example persist_session
//! ```

use qrz_xml::{ApiVersion, QrzXmlClient, QrzXmlError};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct CachedSession {
    session_key: String,
    username: String,
    created_at: u64,
    expires_at: Option<String>,
    lookup_count: Option<u32>,
}

impl CachedSession {
    fn new(
        session_key: String,
        username: String,
        expires_at: Option<String>,
        lookup_count: Option<u32>,
    ) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            session_key,
            username,
            created_at,
            expires_at,
            lookup_count,
        }
    }

    fn is_expired(&self) -> bool {
        // Consider sessions older than 23 hours as potentially expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.created_at > 23 * 3600
    }
}

struct XdgSessionStore {
    cache_dir: PathBuf,
}

impl XdgSessionStore {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let cache_dir = if let Ok(xdg_cache) = env::var("XDG_CACHE_HOME") {
            PathBuf::from(xdg_cache)
        } else if let Ok(home) = env::var("HOME") {
            PathBuf::from(home).join(".cache")
        } else {
            return Err("Cannot determine cache directory".into());
        };

        let app_cache_dir = cache_dir.join("qrz-xml");
        fs::create_dir_all(&app_cache_dir)?;

        Ok(Self {
            cache_dir: app_cache_dir,
        })
    }

    fn session_file_path(&self, username: &str) -> PathBuf {
        self.cache_dir.join(format!("session_{}.json", username))
    }

    fn load_session(&self, username: &str) -> Option<CachedSession> {
        let path = self.session_file_path(username);

        let content = fs::read_to_string(path).ok()?;
        let session: CachedSession = serde_json::from_str(&content).ok()?;

        if session.is_expired() {
            self.clear_session(username);
            return None;
        }

        Some(session)
    }

    fn save_session(&self, session: &CachedSession) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.session_file_path(&session.username);
        let content = serde_json::to_string_pretty(session)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn clear_session(&self, username: &str) {
        let path = self.session_file_path(username);
        let _ = fs::remove_file(path);
    }
}

/// Custom QRZ client with XDG session persistence
struct PersistentQrzXmlClient {
    client: QrzXmlClient,
    store: XdgSessionStore,
    username: String,
}

impl PersistentQrzXmlClient {
    fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        api_version: ApiVersion,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let username_str = username.into();
        let client = QrzXmlClient::new(&username_str, password, api_version)?;
        let store = XdgSessionStore::new()?;

        Ok(Self {
            client,
            store,
            username: username_str,
        })
    }

    async fn ensure_authenticated(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Try to load cached session
        if let Some(cached) = self.store.load_session(&self.username) {
            println!(
                "Found cached session from {}",
                chrono::DateTime::from_timestamp(cached.created_at as i64, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
            );

            // Test if the cached session still works
            match self.client.lookup_callsign("AA7BQ").await {
                Ok(_) => {
                    println!("Cached session is still valid");
                    return Ok(());
                }
                Err(
                    QrzXmlError::SessionExpired
                    | QrzXmlError::NoSessionKey
                    | QrzXmlError::ApiError { .. },
                ) => {
                    println!("Cached session expired, re-authenticating...");
                    self.store.clear_session(&self.username);
                }
                Err(e) => return Err(e.into()),
            }
        }

        // Authenticate and cache the new session
        self.client.authenticate().await?;

        if let Some((count, sub_exp)) = self.client.session_info().await {
            // Note: In a real implementation, you'd need to extract the actual session key
            // from the client. This would require exposing it or creating a custom client.
            let dummy_session = CachedSession::new(
                "session_key_placeholder".to_string(),
                self.username.clone(),
                sub_exp.clone(),
                count,
            );

            self.store.save_session(&dummy_session)?;
            println!("New session cached, expires at: {:?}", sub_exp.as_deref());
        }

        Ok(())
    }

    async fn lookup_callsign(
        &mut self,
        callsign: &str,
    ) -> Result<qrz_xml::CallsignInfo, Box<dyn std::error::Error>> {
        self.ensure_authenticated().await?;
        Ok(self.client.lookup_callsign(callsign).await?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let username = env::var("QRZ_USERNAME")?;
    let password = env::var("QRZ_PASSWORD")?;

    let mut client = PersistentQrzXmlClient::new(&username, &password, ApiVersion::Current)?;

    println!("QRZ XML client with XDG session storage");
    println!("Cache directory: {:?}", client.store.cache_dir);

    // First lookup - will authenticate and cache session
    println!("\nFirst lookup (will cache session):");
    let info = client.lookup_callsign("AA7BQ").await?;
    println!(
        "Found: {} - {}",
        info.call,
        info.full_name().unwrap_or_default()
    );

    // Second lookup - should use cached session
    println!("\nSecond lookup (should use cached session):");
    let info = client.lookup_callsign("W1AW").await?;
    println!(
        "Found: {} - {}",
        info.call,
        info.full_name().unwrap_or_default()
    );

    // Show cache file location
    let session_file = client.store.session_file_path(&username);
    if session_file.exists() {
        println!("\nSession cached at: {}", session_file.display());
        let content = fs::read_to_string(&session_file)?;
        let cached: CachedSession = serde_json::from_str(&content)?;
        println!(
            "Session created: {}",
            chrono::DateTime::from_timestamp(cached.created_at as i64, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
        );
        if let Some(expires) = cached.expires_at {
            println!("Subscription expires: {}", expires);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cached_session_expiry() {
        let session =
            CachedSession::new("test_key".to_string(), "test_user".to_string(), None, None);

        assert!(!session.is_expired());

        // Create an old session
        let old_session = CachedSession {
            created_at: 0, // Unix epoch
            ..session
        };

        assert!(old_session.is_expired());
    }

    #[tokio::test]
    async fn test_xdg_store() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let store = XdgSessionStore {
            cache_dir: temp_dir.path().to_path_buf(),
        };

        let session = CachedSession::new(
            "test_key".to_string(),
            "testuser".to_string(),
            Some("2025-12-31".to_string()),
            Some(42),
        );

        // Save and load
        store.save_session(&session)?;
        let loaded = store.load_session("testuser").unwrap();

        assert_eq!(loaded.session_key, "test_key");
        assert_eq!(loaded.username, "testuser");
        assert_eq!(loaded.lookup_count, Some(42));

        // Clear session
        store.clear_session("testuser");
        assert!(store.load_session("testuser").is_none());

        Ok(())
    }
}
