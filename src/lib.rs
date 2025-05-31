//! # QRZ.com XML API Client
//!
//! A safe, async Rust client library for the QRZ.com XML API.
//!
//! This library provides a complete interface to QRZ.com's XML subscription data service,
//! including callsign lookups, DXCC entity information, and biography data retrieval.
//!
//! ## Features
//!
//! - **Safe & Type-safe**: All API responses are parsed into strongly-typed Rust structs
//! - **Async**: Built on tokio and reqwest for async/await support
//! - **Session Management**: Automatic session handling with intelligent re-authentication
//! - **Error Handling**: Comprehensive error types for all failure modes
//! - **Rate Limiting**: Respects QRZ.com's usage guidelines
//! - **Versioned API**: Support for QRZ's versioned XML interface
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use qrz_xml::{QrzXmlClient, ApiVersion};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = QrzXmlClient::new("your_username", "your_password", ApiVersion::Current)?;
//!     
//!     // Look up a callsign
//!     let callsign_info = client.lookup_callsign("AA7BQ").await?;
//!     println!("Found: {} - {}", callsign_info.call, callsign_info.fname.unwrap_or_default());
//!     
//!     // Look up DXCC entity
//!     let dxcc_info = client.lookup_dxcc_entity(291).await?;
//!     println!("DXCC 291: {}", dxcc_info.name);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Authentication
//!
//! You need a valid QRZ.com username and password. While any QRZ user can authenticate,
//! most features require an active QRZ Logbook Data subscription.

pub mod client;
pub mod error;
pub mod types;

pub use client::QrzXmlClient;
pub use error::{QrzXmlError, Result};
pub use types::{ApiVersion, BiographyData, CallsignInfo, DxccInfo, SessionInfo};

/// Re-export commonly used types from chrono for convenience
pub use chrono::{DateTime, Utc};

/// The default base URL for QRZ's XML API
pub const DEFAULT_BASE_URL: &str = "https://xmldata.qrz.com/xml";

/// Default user agent string for requests
pub const DEFAULT_USER_AGENT: &str = concat!("qrz-xml-rs/", env!("CARGO_PKG_VERSION"));

#[allow(clippy::const_is_empty)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(!DEFAULT_BASE_URL.is_empty());
        assert!(DEFAULT_USER_AGENT.contains("qrz-xml-rs"));
    }
}
