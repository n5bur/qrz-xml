//! Type definitions for QRZ API responses.

use serde::{Deserialize, Serialize};
use std::fmt;

/// API version enum for specifying which version of the QRZ XML interface to use
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiVersion {
    /// Use the current/latest version
    Current,
    /// Use a specific version (e.g., "1.34")
    Specific(String),
    /// Use legacy version (no version specified, defaults to 1.24)
    Legacy,
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiVersion::Current => write!(f, "current"),
            ApiVersion::Specific(version) => write!(f, "{}", version),
            ApiVersion::Legacy => write!(f, ""),
        }
    }
}

impl ApiVersion {
    /// Create a specific version
    pub fn version(version: impl Into<String>) -> Self {
        Self::Specific(version.into())
    }
}

/// Root response container for all QRZ XML responses
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename = "QRZDatabase")]
pub struct QrzXmlResponse {
    /// API version
    #[serde(rename = "@version")]
    pub version: Option<String>,

    /// XML namespace
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,

    /// Session information (always present)
    #[serde(rename = "Session")]
    pub session: SessionInfo,

    /// Callsign information (present for callsign lookups)
    #[serde(rename = "Callsign")]
    pub callsign: Option<CallsignInfo>,

    /// DXCC information (present for DXCC lookups)
    #[serde(rename = "DXCC")]
    pub dxcc: Option<DxccInfo>,
}

/// Session information and status
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionInfo {
    /// Session key for authenticated requests
    #[serde(rename = "Key")]
    pub key: Option<String>,

    /// Number of lookups performed in current 24-hour period
    #[serde(rename = "Count")]
    pub count: Option<u32>,

    /// Subscription expiration date or "non-subscriber"
    #[serde(rename = "SubExp")]
    pub sub_exp: Option<String>,

    /// Current GMT time
    #[serde(rename = "GMTime")]
    pub gm_time: Option<String>,

    /// Informational message
    #[serde(rename = "Message")]
    pub message: Option<String>,

    /// Error message
    #[serde(rename = "Error")]
    pub error: Option<String>,
}

impl SessionInfo {
    /// Check if session has a valid key
    pub fn has_valid_session(&self) -> bool {
        self.key.is_some()
    }

    /// Check if there's an error
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Get the error message if present
    pub fn error_message(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Get the informational message if present
    pub fn info_message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

/// Comprehensive callsign information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CallsignInfo {
    /// Primary callsign
    #[serde(rename = "call")]
    pub call: String,

    /// Cross reference callsign that returned this record
    #[serde(rename = "xref")]
    pub xref: Option<String>,

    /// Other callsigns that resolve to this record
    #[serde(rename = "aliases")]
    pub aliases: Option<String>,

    /// DXCC entity ID (country code)
    #[serde(rename = "dxcc")]
    pub dxcc: Option<u32>,

    /// First name
    #[serde(rename = "fname")]
    pub fname: Option<String>,

    /// Last name
    #[serde(rename = "name")]
    pub name: Option<String>,

    /// Address line 1 (house number and street)
    #[serde(rename = "addr1")]
    pub addr1: Option<String>,

    /// Address line 2 (city)
    #[serde(rename = "addr2")]
    pub addr2: Option<String>,

    /// State (USA only)
    #[serde(rename = "state")]
    pub state: Option<String>,

    /// ZIP/postal code
    #[serde(rename = "zip")]
    pub zip: Option<String>,

    /// Country name for QSL mailing address
    #[serde(rename = "country")]
    pub country: Option<String>,

    /// DXCC entity code for mailing address country
    #[serde(rename = "ccode")]
    pub ccode: Option<u32>,

    /// Latitude (signed decimal, S < 0 > N)
    #[serde(rename = "lat")]
    pub lat: Option<f64>,

    /// Longitude (signed decimal, W < 0 > E)
    #[serde(rename = "lon")]
    pub lon: Option<f64>,

    /// Grid locator
    #[serde(rename = "grid")]
    pub grid: Option<String>,

    /// County name (USA)
    #[serde(rename = "county")]
    pub county: Option<String>,

    /// FIPS county identifier (USA)
    #[serde(rename = "fips")]
    pub fips: Option<String>,

    /// DXCC country name of the callsign
    #[serde(rename = "land")]
    pub land: Option<String>,

    /// License effective date (USA)
    #[serde(rename = "efdate")]
    pub efdate: Option<String>,

    /// License expiration date (USA)
    #[serde(rename = "expdate")]
    pub expdate: Option<String>,

    /// Previous callsign
    #[serde(rename = "p_call")]
    pub p_call: Option<String>,

    /// License class
    #[serde(rename = "class")]
    pub class: Option<String>,

    /// License type codes (USA)
    #[serde(rename = "codes")]
    pub codes: Option<String>,

    /// QSL manager info
    #[serde(rename = "qslmgr")]
    pub qslmgr: Option<String>,

    /// Email address
    #[serde(rename = "email")]
    pub email: Option<String>,

    /// Web page address
    #[serde(rename = "url")]
    pub url: Option<String>,

    /// QRZ web page views
    #[serde(rename = "u_views")]
    pub u_views: Option<u32>,

    /// Biography size in bytes
    #[serde(rename = "bio")]
    pub bio: Option<String>,

    /// Biography last update date
    #[serde(rename = "biodate")]
    pub biodate: Option<String>,

    /// Full URL of primary image
    #[serde(rename = "image")]
    pub image: Option<String>,

    /// Image dimensions (height:width:size)
    #[serde(rename = "imageinfo")]
    pub imageinfo: Option<String>,

    /// QRZ database serial number
    #[serde(rename = "serial")]
    pub serial: Option<u32>,

    /// Last modified date
    #[serde(rename = "moddate")]
    pub moddate: Option<String>,

    /// Metro Service Area (USPS)
    #[serde(rename = "MSA")]
    pub msa: Option<String>,

    /// Telephone area code (USA)
    #[serde(rename = "AreaCode")]
    pub area_code: Option<String>,

    /// Time zone (USA)
    #[serde(rename = "TimeZone")]
    pub time_zone: Option<String>,

    /// GMT time offset
    #[serde(rename = "GMTOffset")]
    pub gmt_offset: Option<String>,

    /// Daylight saving time observed
    #[serde(rename = "DST")]
    pub dst: Option<String>,

    /// Will accept eQSL (Y/N or blank)
    #[serde(rename = "eqsl")]
    pub eqsl: Option<String>,

    /// Will return paper QSL (Y/N or blank)
    #[serde(rename = "mqsl")]
    pub mqsl: Option<String>,

    /// CQ Zone identifier
    #[serde(rename = "cqzone")]
    pub cqzone: Option<u32>,

    /// ITU Zone identifier
    #[serde(rename = "ituzone")]
    pub ituzone: Option<u32>,

    /// Operator's birth year
    #[serde(rename = "born")]
    pub born: Option<u32>,

    /// User who manages this callsign on QRZ
    #[serde(rename = "user")]
    pub user: Option<String>,

    /// Will accept LOTW (Y/N or blank)
    #[serde(rename = "lotw")]
    pub lotw: Option<String>,

    /// IOTA designator
    #[serde(rename = "iota")]
    pub iota: Option<String>,

    /// Source of lat/long data
    #[serde(rename = "geoloc")]
    pub geoloc: Option<String>,

    /// Attention address line (new in v1.34)
    #[serde(rename = "attn")]
    pub attn: Option<String>,

    /// Nickname (new in v1.34)
    #[serde(rename = "nickname")]
    pub nickname: Option<String>,

    /// Combined full name and nickname (new in v1.34)
    #[serde(rename = "name_fmt")]
    pub name_fmt: Option<String>,
}

impl CallsignInfo {
    /// Get the full name (combining first and last name)
    pub fn full_name(&self) -> Option<String> {
        match (&self.fname, &self.name) {
            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
            (Some(first), None) => Some(first.clone()),
            (None, Some(last)) => Some(last.clone()),
            (None, None) => None,
        }
    }

    /// Get coordinates as a tuple (lat, lon) if both are present
    pub fn coordinates(&self) -> Option<(f64, f64)> {
        match (self.lat, self.lon) {
            (Some(lat), Some(lon)) => Some((lat, lon)),
            _ => None,
        }
    }

    /// Check if QSL information indicates acceptance of eQSL
    pub fn accepts_eqsl(&self) -> Option<bool> {
        self.eqsl.as_ref().map(|s| s.eq_ignore_ascii_case("y"))
    }

    /// Check if QSL information indicates will return paper QSL
    pub fn returns_paper_qsl(&self) -> Option<bool> {
        self.mqsl.as_ref().map(|s| s.eq_ignore_ascii_case("y"))
    }

    /// Check if LOTW is accepted
    pub fn accepts_lotw(&self) -> Option<bool> {
        self.lotw.as_ref().map(|s| s.eq_ignore_ascii_case("y"))
    }
}

/// DXCC entity information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DxccInfo {
    /// DXCC entity number
    #[serde(rename = "dxcc")]
    pub dxcc: u32,

    /// 2-letter country code (ISO-3166)
    #[serde(rename = "cc")]
    pub cc: Option<String>,

    /// 3-letter country code (ISO-3166)
    #[serde(rename = "ccc")]
    pub ccc: Option<String>,

    /// Long country name
    #[serde(rename = "name")]
    pub name: String,

    /// 2-letter continent designator
    #[serde(rename = "continent")]
    pub continent: Option<String>,

    /// ITU Zone
    #[serde(rename = "ituzone")]
    pub ituzone: Option<u32>,

    /// CQ Zone
    #[serde(rename = "cqzone")]
    pub cqzone: Option<u32>,

    /// UTC timezone offset +/-
    #[serde(rename = "timezone")]
    pub timezone: Option<String>,

    /// Latitude (approximate center)
    #[serde(rename = "lat")]
    pub lat: Option<f64>,

    /// Longitude (approximate center)
    #[serde(rename = "lon")]
    pub lon: Option<f64>,

    /// Special notes and exceptions
    #[serde(rename = "notes")]
    pub notes: Option<String>,
}

impl DxccInfo {
    /// Get coordinates as a tuple (lat, lon) if both are present
    pub fn coordinates(&self) -> Option<(f64, f64)> {
        match (self.lat, self.lon) {
            (Some(lat), Some(lon)) => Some((lat, lon)),
            _ => None,
        }
    }

    /// Parse timezone offset as hours (may include fractions)
    pub fn timezone_hours(&self) -> Option<f32> {
        self.timezone.as_ref().and_then(|tz| {
            // Handle formats like "+5", "-8", "545" (5 hours 45 minutes)
            let tz = tz.trim_start_matches('+');
            if tz.len() >= 3 {
                // Format like "545" means 5:45
                if let (Ok(hours), Ok(minutes)) = (
                    tz[..tz.len() - 2].parse::<i32>(),
                    tz[tz.len() - 2..].parse::<i32>(),
                ) {
                    return Some(hours as f32 + minutes as f32 / 60.0);
                }
            }
            tz.parse::<f32>().ok()
        })
    }
}

/// Biography/HTML data container
#[derive(Debug, Clone)]
pub struct BiographyData {
    /// The callsign this biography belongs to
    pub callsign: String,
    /// Raw HTML content
    pub html_content: String,
}

impl BiographyData {
    /// Create new biography data
    pub fn new(callsign: impl Into<String>, html_content: impl Into<String>) -> Self {
        Self {
            callsign: callsign.into(),
            html_content: html_content.into(),
        }
    }

    /// Get the HTML content
    pub fn html(&self) -> &str {
        &self.html_content
    }

    /// Check if the biography is empty
    pub fn is_empty(&self) -> bool {
        self.html_content.trim().is_empty()
    }
}

// Implement Default for CallsignInfo to help with testing
#[allow(clippy::derivable_impls)]
impl Default for CallsignInfo {
    fn default() -> Self {
        Self {
            call: String::new(),
            xref: None,
            aliases: None,
            dxcc: None,
            fname: None,
            name: None,
            addr1: None,
            addr2: None,
            state: None,
            zip: None,
            country: None,
            ccode: None,
            lat: None,
            lon: None,
            grid: None,
            county: None,
            fips: None,
            land: None,
            efdate: None,
            expdate: None,
            p_call: None,
            class: None,
            codes: None,
            qslmgr: None,
            email: None,
            url: None,
            u_views: None,
            bio: None,
            biodate: None,
            image: None,
            imageinfo: None,
            serial: None,
            moddate: None,
            msa: None,
            area_code: None,
            time_zone: None,
            gmt_offset: None,
            dst: None,
            eqsl: None,
            mqsl: None,
            cqzone: None,
            ituzone: None,
            born: None,
            user: None,
            lotw: None,
            iota: None,
            geoloc: None,
            attn: None,
            nickname: None,
            name_fmt: None,
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for DxccInfo {
    fn default() -> Self {
        Self {
            dxcc: 0,
            cc: None,
            ccc: None,
            name: String::new(),
            continent: None,
            ituzone: None,
            cqzone: None,
            timezone: None,
            lat: None,
            lon: None,
            notes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_display() {
        assert_eq!(ApiVersion::Current.to_string(), "current");
        assert_eq!(ApiVersion::version("1.34").to_string(), "1.34");
        assert_eq!(ApiVersion::Legacy.to_string(), "");
    }

    #[test]
    fn test_callsign_full_name() {
        let mut info = CallsignInfo {
            call: "TEST".to_string(),
            fname: Some("John".to_string()),
            name: Some("Doe".to_string()),
            ..Default::default()
        };

        assert_eq!(info.full_name(), Some("John Doe".to_string()));

        info.name = None;
        assert_eq!(info.full_name(), Some("John".to_string()));
    }

    #[test]
    fn test_coordinates() {
        let info = CallsignInfo {
            call: "TEST".to_string(),
            lat: Some(40.7128),
            lon: Some(-74.0060),
            ..Default::default()
        };

        assert_eq!(info.coordinates(), Some((40.7128, -74.0060)));
    }

    #[test]
    fn test_qsl_flags() {
        let info = CallsignInfo {
            call: "TEST".to_string(),
            eqsl: Some("Y".to_string()),
            mqsl: Some("N".to_string()),
            lotw: Some("y".to_string()),
            ..Default::default()
        };

        assert_eq!(info.accepts_eqsl(), Some(true));
        assert_eq!(info.returns_paper_qsl(), Some(false));
        assert_eq!(info.accepts_lotw(), Some(true));
    }

    #[test]
    fn test_dxcc_timezone_parsing() {
        let mut dxcc = DxccInfo {
            dxcc: 291,
            name: "Test".to_string(),
            timezone: Some("-5".to_string()),
            ..Default::default()
        };

        assert_eq!(dxcc.timezone_hours(), Some(-5.0));

        dxcc.timezone = Some("545".to_string());
        assert_eq!(dxcc.timezone_hours(), Some(5.75)); // 5 hours 45 minutes
    }
}
