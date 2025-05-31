//! Integration tests for the QRZ library.
//!
//! These tests use wiremock to simulate the QRZ API responses
//! and test the complete flow without hitting the real API.

use qrz_xml::client::QrzXmlClientConfig;
use qrz_xml::{ApiVersion, QrzXmlClient, QrzXmlError};
use wiremock::matchers::{method, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SAMPLE_LOGIN_RESPONSE: &str = r#"<?xml version="1.0" ?>
<QRZDatabase version="1.34">
  <Session>
    <Key>test_session_key_12345</Key>
    <Count>42</Count>
    <SubExp>Wed Jan 1 12:34:03 2025</SubExp>
    <GMTime>Sun Aug 16 03:51:47 2024</GMTime>
  </Session>
</QRZDatabase>"#;

const SAMPLE_CALLSIGN_RESPONSE: &str = r#"<?xml version="1.0" ?>
<QRZDatabase version="1.34">
  <Callsign>
    <call>AA7BQ</call>
    <aliases>N6UFT,KJ6RK</aliases>
    <dxcc>291</dxcc>
    <fname>FRED</fname>
    <name>LLOYD</name>
    <addr1>123 TEST ST</addr1>
    <addr2>TESTVILLE</addr2>
    <state>AZ</state>
    <zip>12345</zip>
    <country>United States</country>
    <lat>34.12345</lat>
    <lon>-112.12345</lon>
    <grid>DM32af</grid>
    <class>E</class>
    <email>test@example.com</email>
    <eqsl>Y</eqsl>
    <mqsl>N</mqsl>
    <lotw>Y</lotw>
    <cqzone>3</cqzone>
    <ituzone>2</ituzone>
    <nickname>Test Op</nickname>
  </Callsign>
  <Session>
    <Key>test_session_key_12345</Key>
    <Count>43</Count>
    <SubExp>Wed Jan 1 12:34:03 2025</SubExp>
    <GMTime>Sun Aug 16 03:52:47 2024</GMTime>
  </Session>
</QRZDatabase>"#;

const SAMPLE_DXCC_RESPONSE: &str = r#"<?xml version="1.0" ?>
<QRZDatabase version="1.34">
  <DXCC>
    <dxcc>291</dxcc>
    <cc>US</cc>
    <ccc>USA</ccc>
    <name>United States</name>
    <continent>NA</continent>
    <ituzone>6</ituzone>
    <cqzone>3</cqzone>
    <timezone>-5</timezone>
    <lat>37.788081</lat>
    <lon>-97.470703</lon>
  </DXCC>
  <Session>
    <Key>test_session_key_12345</Key>
    <Count>44</Count>
    <SubExp>Wed Jan 1 12:34:03 2025</SubExp>
    <GMTime>Sun Aug 16 03:53:47 2024</GMTime>
  </Session>
</QRZDatabase>"#;

const SAMPLE_ERROR_RESPONSE: &str = r#"<?xml version="1.0" ?>
<QRZDatabase version="1.34">
  <Session>
    <Error>Not found: INVALIDCALL</Error>
    <Key>test_session_key_12345</Key>
    <GMTime>Sun Aug 16 03:54:47 2024</GMTime>
  </Session>
</QRZDatabase>"#;

const SAMPLE_SESSION_TIMEOUT_RESPONSE: &str = r#"<?xml version="1.0" ?>
<QRZDatabase version="1.34">
  <Session>
    <Error>Session Timeout</Error>
    <GMTime>Sun Aug 16 03:55:47 2024</GMTime>
  </Session>
</QRZDatabase>"#;

const SAMPLE_AUTH_ERROR_RESPONSE: &str = r#"<?xml version="1.0" ?>
<QRZDatabase version="1.34">
  <Session>
    <Error>Username/password incorrect</Error>
    <GMTime>Sun Aug 16 03:56:47 2024</GMTime>
  </Session>
</QRZDatabase>"#;

async fn create_test_client(mock_server_uri: &str) -> QrzXmlClient {
    let config = QrzXmlClientConfig {
        base_url: format!("{}/xml", mock_server_uri),
        user_agent: "qrz-test/1.0".to_string(),
        timeout_seconds: 5,
        max_retries: 1,
    };

    QrzXmlClient::with_config("testuser", "testpass", ApiVersion::Current, config).unwrap()
}

#[tokio::test]
async fn test_successful_authentication() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(query_param("username", "testuser"))
        .and(query_param("password", "testpass"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_LOGIN_RESPONSE))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri()).await;
    let result = client.authenticate().await;

    assert!(result.is_ok());
    assert!(client.is_authenticated().await);
}

#[tokio::test]
async fn test_authentication_failure() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(query_param("username", "testuser"))
        .and(query_param("password", "testpass"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_AUTH_ERROR_RESPONSE))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri()).await;
    let result = client.authenticate().await;

    assert!(result.is_err());
    match result.unwrap_err() {
        QrzXmlError::AuthenticationFailed { reason } => {
            assert!(reason.contains("Username/password incorrect"));
        }
        _ => panic!("Expected AuthenticationFailed error"),
    }
}

#[tokio::test]
async fn test_successful_callsign_lookup() {
    let mock_server = MockServer::start().await;

    // Mock login
    Mock::given(method("GET"))
        .and(query_param("username", "testuser"))
        .and(query_param("password", "testpass"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_LOGIN_RESPONSE))
        .mount(&mock_server)
        .await;

    // Mock callsign lookup
    Mock::given(method("GET"))
        .and(query_param("s", "test_session_key_12345"))
        .and(query_param("callsign", "AA7BQ"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_CALLSIGN_RESPONSE))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri()).await;
    let result = client.lookup_callsign("AA7BQ").await;

    assert!(result.is_ok());
    let callsign_info = result.unwrap();

    assert_eq!(callsign_info.call, "AA7BQ");
    assert_eq!(callsign_info.fname, Some("FRED".to_string()));
    assert_eq!(callsign_info.name, Some("LLOYD".to_string()));
    assert_eq!(callsign_info.full_name(), Some("FRED LLOYD".to_string()));
    assert_eq!(callsign_info.state, Some("AZ".to_string()));
    assert_eq!(callsign_info.grid, Some("DM32af".to_string()));
    assert_eq!(callsign_info.accepts_eqsl(), Some(true));
    assert_eq!(callsign_info.returns_paper_qsl(), Some(false));
    assert_eq!(callsign_info.accepts_lotw(), Some(true));

    let coords = callsign_info.coordinates();
    assert!(coords.is_some());
    let (lat, lon) = coords.unwrap();
    assert!((lat - 34.12345).abs() < 0.001);
    assert!((lon - (-112.12345)).abs() < 0.001);
}

#[tokio::test]
async fn test_callsign_not_found() {
    let mock_server = MockServer::start().await;

    // Mock login
    Mock::given(method("GET"))
        .and(query_param("username", "testuser"))
        .and(query_param("password", "testpass"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_LOGIN_RESPONSE))
        .mount(&mock_server)
        .await;

    // Mock callsign not found
    Mock::given(method("GET"))
        .and(query_param("s", "test_session_key_12345"))
        .and(query_param("callsign", "INVALIDCALL"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_ERROR_RESPONSE))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri()).await;
    let result = client.lookup_callsign("INVALIDCALL").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        QrzXmlError::ApiError { message } => {
            assert!(message.contains("Not found: INVALIDCALL"));
        }
        _ => panic!("Expected ApiError for not found callsign"),
    }
}

#[tokio::test]
async fn test_successful_dxcc_lookup() {
    let mock_server = MockServer::start().await;

    // Mock login
    Mock::given(method("GET"))
        .and(query_param("username", "testuser"))
        .and(query_param("password", "testpass"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_LOGIN_RESPONSE))
        .mount(&mock_server)
        .await;

    // Mock DXCC lookup
    Mock::given(method("GET"))
        .and(query_param("s", "test_session_key_12345"))
        .and(query_param("dxcc", "291"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_DXCC_RESPONSE))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri()).await;
    let result = client.lookup_dxcc_entity(291).await;

    assert!(result.is_ok());
    let dxcc_info = result.unwrap();

    assert_eq!(dxcc_info.dxcc, 291);
    assert_eq!(dxcc_info.name, "United States");
    assert_eq!(dxcc_info.cc, Some("US".to_string()));
    assert_eq!(dxcc_info.ccc, Some("USA".to_string()));
    assert_eq!(dxcc_info.continent, Some("NA".to_string()));
    assert_eq!(dxcc_info.cqzone, Some(3));
    assert_eq!(dxcc_info.ituzone, Some(6));
    assert_eq!(dxcc_info.timezone_hours(), Some(-5.0));

    let coords = dxcc_info.coordinates();
    assert!(coords.is_some());
}

#[tokio::test]
async fn test_session_timeout_and_reauthentication() {
    let mock_server = MockServer::start().await;

    // Mock initial login
    Mock::given(method("GET"))
        .and(query_param("username", "testuser"))
        .and(query_param("password", "testpass"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_LOGIN_RESPONSE))
        .expect(2) // Will be called twice due to re-auth
        .mount(&mock_server)
        .await;

    // Mock session timeout on first callsign request
    Mock::given(method("GET"))
        .and(query_param("s", "test_session_key_12345"))
        .and(query_param("callsign", "AA7BQ"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_SESSION_TIMEOUT_RESPONSE))
        .expect(2)
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri()).await;

    // First authenticate manually
    client.authenticate().await.unwrap();

    // Try to make a request - this should detect the session timeout and NOT retry
    // since our current implementation only retries once and the timeout response
    // doesn't contain a session key
    let result = client.lookup_callsign("AA7BQ").await;

    // This should fail with session expired, not succeed
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_input_handling() {
    let mock_server = MockServer::start().await;
    let client = create_test_client(&mock_server.uri()).await;

    // Test empty callsign
    let result = client.lookup_callsign("").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        QrzXmlError::InvalidInput { .. } => {}
        _ => panic!("Expected InvalidInput error for empty callsign"),
    }
}

#[tokio::test]
async fn test_session_info_tracking() {
    let mock_server = MockServer::start().await;

    // Mock login
    Mock::given(method("GET"))
        .and(query_param("username", "testuser"))
        .and(query_param("password", "testpass"))
        .respond_with(ResponseTemplate::new(200).set_body_string(SAMPLE_LOGIN_RESPONSE))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server.uri()).await;

    // Before authentication
    assert!(!client.is_authenticated().await);
    let session_info = client.session_info().await;
    assert!(session_info.is_some()); // session_info always returns Some, but values may be None
    let (count, sub_exp) = session_info.unwrap();
    assert_eq!(count, None);
    assert_eq!(sub_exp, None);

    // After authentication
    client.authenticate().await.unwrap();
    assert!(client.is_authenticated().await);

    let session_info = client.session_info().await;
    assert!(session_info.is_some());
    let (count, sub_exp) = session_info.unwrap();
    assert_eq!(count, Some(42));
    assert_eq!(sub_exp, Some("Wed Jan 1 12:34:03 2025".to_string()));
}

// Helper functions for testing individual components

#[tokio::test]
async fn test_error_type_properties() {
    // Test error categorization
    assert!(QrzXmlError::SessionExpired.should_reauthenticate());
    assert!(QrzXmlError::NoSessionKey.should_reauthenticate());
    assert!(!QrzXmlError::CallsignNotFound {
        callsign: "TEST".to_string()
    }
    .should_reauthenticate());

    assert!(QrzXmlError::RateLimitExceeded.is_retryable());
    assert!(QrzXmlError::SessionExpired.is_retryable());
    assert!(!QrzXmlError::AuthenticationFailed {
        reason: "bad pass".to_string()
    }
    .is_retryable());

    assert!(QrzXmlError::SubscriptionRequired.is_permission_error());
    assert!(QrzXmlError::ConnectionRefused.is_permission_error());
    assert!(!QrzXmlError::CallsignNotFound {
        callsign: "TEST".to_string()
    }
    .is_permission_error());
}
