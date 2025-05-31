# qrz-xml

[![CI](https://github.com/n5bur/qrz-xml/actions/workflows/ci.yml/badge.svg)](https://github.com/n5bur/qrz-xml/actions/workflows/ci.yml)

A safe, async Rust client library for the [QRZ.com](https://qrz.com) XML API.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qrz_xml = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## Example Usage

```rust
use qrz_xml::{QrzXmlClient, ApiVersion};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client using your QRZ.com credentials
    let client = QrzXmlClient::new(
        "your_username", 
        "your_password", 
        ApiVersion::Current
    )?;
    
    // Look up a callsign
    let callsign_info = client.lookup_callsign("AA7BQ").await?;
    println!("Found: {} - {}", 
        callsign_info.call, 
        callsign_info.full_name().unwrap_or_default()
    );
    
    // Look up DXCC entity
    let dxcc_info = client.lookup_dxcc_entity(291).await?;
    println!("DXCC 291: {}", dxcc_info.name);
    
    Ok(())
}
```

## Authentication

You need a valid QRZ.com username and password. While any QRZ user can authenticate, most features require an active **QRZ Logbook Data subscription**.

Visit [QRZ.com subscriptions](https://www.qrz.com/i/subscriptions.html) for more information about subscription plans.

## API Coverage

### Callsign Lookups

```rust
let callsign_info = client.lookup_callsign("W1AW").await?;

// Access comprehensive information
println!("Name: {}", callsign_info.full_name().unwrap_or_default());
println!("Grid: {}", callsign_info.grid.unwrap_or_default());
println!("Country: {}", callsign_info.country.unwrap_or_default());

// Geographic coordinates
if let Some((lat, lon)) = callsign_info.coordinates() {
    println!("Location: {:.4}°, {:.4}°", lat, lon);
}

// QSL preferences
if callsign_info.accepts_eqsl() == Some(true) {
    println!("Accepts eQSL");
}
```

### DXCC Entity Lookups

```rust
// Look up by entity number
let usa = client.lookup_dxcc_entity(291).await?;
println!("{} - {}", usa.cc.unwrap(), usa.name);

// Look up by callsign prefix
let dxcc = client.lookup_dxcc_by_callsign("JA1ABC").await?;
println!("Japan: DXCC {}", dxcc.dxcc);
```

### Biography Data

```rust
let bio = client.lookup_biography("AA7BQ").await?;
println!("Biography HTML length: {}", bio.html().len());

// The biography contains raw HTML as it appears on QRZ.com
if !bio.is_empty() {
    // Process or display the HTML content
    println!("Has biography data available");
}
```

## Error Handling

The library provides comprehensive error handling with specific error types:

```rust
use qrz_xml::QrzXmlError;

match client.lookup_callsign("INVALID").await {
    Ok(info) => println!("Found: {}", info.call),
    Err(QrzXmlError::CallsignNotFound { callsign }) => {
        println!("Callsign {} not found", callsign);
    }
    Err(QrzXmlError::SubscriptionRequired) => {
        println!("This feature requires a QRZ subscription");
    }
    Err(QrzXmlError::AuthenticationFailed { reason }) => {
        println!("Auth failed: {}", reason);
    }
    Err(e) => println!("Other error: {}", e),
}
```

## Configuration

Customize the client behavior with `QrzXmlClientConfig`:

```rust
use qrz::{QrzXmlClient, ApiVersion};
use qrz::client::QrzXmlClientConfig

let config = QrzXmlClientConfig {
    base_url: "https://xmldata.qrz.com/xml".to_string(),
    user_agent: "my-app/1.0".to_string(),
    timeout_seconds: 30,
    max_retries: 3,
};

let client = QrzXmlClient::with_config(
    "username", 
    "password", 
    ApiVersion::Current, 
    config
)?;
```

## API Versions

QRZ.com provides a versioned XML interface. You can specify which version to use:

```rust
// Use the latest version (recommended)
let client = QrzXmlClient::new("user", "pass", ApiVersion::Current)?;

// Use a specific version
let client = QrzXmlClient::new("user", "pass", ApiVersion::version("1.34"))?;

// Use legacy version (1.24)
let client = QrzXmlClient::new("user", "pass", ApiVersion::Legacy)?;
```

## Session Management

The client automatically handles session management:

- **Automatic Login**: Sessions are established automatically on first request
- **Session Caching**: Session keys are cached and reused efficiently  
- **Auto Re-authentication**: Expired sessions are detected and renewed automatically
- **Session Info**: Access lookup counts and subscription status

```rust
// Check authentication status
if client.is_authenticated().await {
    println!("Ready to make requests");
}

// Get session information
if let Some((count, sub_exp)) = client.session_info().await {
    println!("Lookups today: {:?}", count);
    println!("Subscription expires: {:?}", sub_exp);
}

// Force re-authentication if needed
client.reauthenticate().await?;
```

## Rate Limiting

The library respects QRZ.com's usage guidelines:

- Session keys are cached and reused to minimize server load
- Failed requests are not automatically retried (except for session expiration)
- The library tracks lookup counts returned by the API

You should implement your own rate limiting if making many requests:

```rust
use tokio::time::{sleep, Duration};

for callsign in callsigns {
    let result = client.lookup_callsign(&callsign).await?;
    // Process result...
    
    // Be respectful - add a small delay between requests
    sleep(Duration::from_millis(100)).await;
}
```

## Examples

The crate includes several examples in the `examples/` directory:

```bash
# Basic callsign lookup
QRZ_USERNAME=xxx QRZ_PASSWORD=yyy cargo run --example basic_lookup -- AA7BQ

# DXCC entity lookup  
QRZ_USERNAME=xxx QRZ_PASSWORD=yyy cargo run --example dxcc_lookup -- --entity 291
```

## Testing

Run the test suite:

```bash
cargo test
```

The tests include both unit tests and integration tests with mocked API responses, so they don't require QRZ.com credentials.

## TLS Support

The library supports both native TLS and rustls:

```toml
# Use native TLS (default)
qrz_xml = "0.1"

# Use rustls instead
qrz_xml = { version = "0.1", default-features = false, features = ["rustls-tls"] }
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Disclaimer

This library is not affiliated with or endorsed by QRZ.com. QRZ.com is a trademark of QRZ LLC.

Users of this library must comply with QRZ.com's Terms of Service and API usage guidelines.

## See Also

- [QRZ.com XML API Documentation](https://www.qrz.com/docs/xml/current_spec.html)
- [QRZ.com Subscription Plans](https://www.qrz.com/i/subscriptions.html)