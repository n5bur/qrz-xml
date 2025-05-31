//! Basic callsign lookup example for the QRZ library.
//!
//! This example demonstrates how to:
//! - Create a QRZ client
//! - Authenticate with QRZ.com
//! - Look up callsign information
//! - Handle errors gracefully
//!
//! Usage:
//! ```
//! QRZ_USERNAME=your_username QRZ_PASSWORD=your_password cargo run --example basic_lookup -- AA7BQ
//! ```

use qrz_xml::{ApiVersion, QrzXmlClient, QrzXmlError};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Get credentials from environment variables
    let username = env::var("QRZ_USERNAME").expect("QRZ_USERNAME environment variable must be set");
    let password = env::var("QRZ_PASSWORD").expect("QRZ_PASSWORD environment variable must be set");

    // Get callsign from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <callsign>", args[0]);
        eprintln!("Example: {} AA7BQ", args[0]);
        std::process::exit(1);
    }
    let callsign = &args[1];

    // Create client using the current API version
    println!("Creating QRZ client...");
    let client = QrzXmlClient::new(&username, &password, ApiVersion::Current)?;

    // Authenticate (this happens automatically on first request, but we can do it explicitly)
    println!("Authenticating with QRZ.com...");
    match client.authenticate().await {
        Ok(()) => println!("Authentication successful!"),
        Err(QrzXmlError::AuthenticationFailed { reason }) => {
            eprintln!("Authentication failed: {}", reason);
            std::process::exit(1);
        }
        Err(QrzXmlError::ConnectionRefused) => {
            eprintln!("QRZ is refusing connections. Try again in 24 hours.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Authentication error: {}", e);
            std::process::exit(1);
        }
    }

    // Look up the callsign
    println!("Looking up callsign: {}", callsign);
    match client.lookup_callsign(callsign).await {
        Ok(info) => {
            println!("\n=== Callsign Information ===");
            println!("Call: {}", info.call);

            if let Some(name) = info.full_name() {
                println!("Name: {}", name);
            }

            if let Some(nickname) = &info.nickname {
                println!("Nickname: {}", nickname);
            }

            if let Some(email) = &info.email {
                println!("Email: {}", email);
            }

            // Address information
            if let (Some(addr1), Some(addr2)) = (&info.addr1, &info.addr2) {
                println!("Address: {}, {}", addr1, addr2);
                if let Some(state) = &info.state {
                    if let Some(zip) = &info.zip {
                        println!("         {}, {}", state, zip);
                    }
                }
            }

            if let Some(country) = &info.country {
                println!("Country: {}", country);
            }

            // License information
            if let Some(class) = &info.class {
                println!("License Class: {}", class);
            }

            if let (Some(efdate), Some(expdate)) = (&info.efdate, &info.expdate) {
                println!("License: {} to {}", efdate, expdate);
            }

            // Grid and coordinates
            if let Some(grid) = &info.grid {
                println!("Grid: {}", grid);
            }

            if let Some((lat, lon)) = info.coordinates() {
                println!("Coordinates: {:.4}°, {:.4}°", lat, lon);
            }

            // QSL information
            let mut qsl_methods = Vec::new();
            if info.accepts_eqsl() == Some(true) {
                qsl_methods.push("eQSL");
            }
            if info.returns_paper_qsl() == Some(true) {
                qsl_methods.push("Paper QSL");
            }
            if info.accepts_lotw() == Some(true) {
                qsl_methods.push("LoTW");
            }
            if !qsl_methods.is_empty() {
                println!("QSL Methods: {}", qsl_methods.join(", "));
            }

            // Zones
            if let (Some(cq), Some(itu)) = (info.cqzone, info.ituzone) {
                println!("Zones: CQ {}, ITU {}", cq, itu);
            }

            if let Some(dxcc) = info.dxcc {
                println!("DXCC Entity: {}", dxcc);
            }

            // Biography information
            if info.bio.is_some() {
                println!("Biography: Available (use biography lookup to fetch)");
            }

            if let Some(url) = &info.url {
                println!("Website: {}", url);
            }

            if let Some(views) = info.u_views {
                println!("QRZ Page Views: {}", views);
            }
        }
        Err(QrzXmlError::CallsignNotFound { callsign }) => {
            eprintln!("Callsign not found: {}", callsign);
            std::process::exit(1);
        }
        Err(QrzXmlError::SubscriptionRequired) => {
            eprintln!("A QRZ subscription is required to access complete callsign data.");
            eprintln!("You may see limited information without a subscription.");
        }
        Err(e) => {
            eprintln!("Lookup error: {}", e);
            std::process::exit(1);
        }
    }

    // Show session information
    if let Some((count, sub_exp)) = client.session_info().await {
        println!("\n=== Session Information ===");
        if let Some(count) = count {
            println!("Lookups today: {}", count);
        }
        if let Some(sub_exp) = sub_exp {
            println!("Subscription expires: {}", sub_exp);
        }
    }

    Ok(())
}
