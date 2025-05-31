//! DXCC entity lookup example for the QRZ library.
//!
//! This example demonstrates how to:
//! - Look up DXCC entities by number
//! - Look up DXCC entities by callsign prefix
//! - Handle timezone and coordinate information
//!
//! Usage:
//! ```
//! # Look up by entity number
//! QRZ_USERNAME=your_username QRZ_PASSWORD=your_password cargo run --example dxcc_lookup -- --entity 291
//!
//! # Look up by callsign prefix
//! QRZ_USERNAME=your_username QRZ_PASSWORD=your_password cargo run --example dxcc_lookup -- --callsign JA1ABC
//! ```

use qrz_xml::{ApiVersion, QrzXmlClient, QrzXmlError};
use std::env;

#[derive(Debug)]
enum LookupType {
    Entity(u32),
    Callsign(String),
}

fn parse_args() -> Result<LookupType, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        return Err(format!(
            "Usage: {} --entity <number> | --callsign <callsign>\n\
             Examples:\n\
             {} --entity 291\n\
             {} --callsign JA1ABC",
            args[0], args[0], args[0]
        ));
    }

    match args[1].as_str() {
        "--entity" => {
            let entity = args[2]
                .parse::<u32>()
                .map_err(|_| "Entity number must be a valid integer".to_string())?;
            Ok(LookupType::Entity(entity))
        }
        "--callsign" => Ok(LookupType::Callsign(args[2].clone())),
        _ => Err("First argument must be --entity or --callsign".to_string()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let lookup_type = match parse_args() {
        Ok(lt) => lt,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    // Get credentials from environment variables
    let username = env::var("QRZ_USERNAME").expect("QRZ_USERNAME environment variable must be set");
    let password = env::var("QRZ_PASSWORD").expect("QRZ_PASSWORD environment variable must be set");

    // Create client
    println!("Creating QRZ client...");
    let client = QrzXmlClient::new(&username, &password, ApiVersion::Current)?;

    // Authenticate
    println!("Authenticating with QRZ.com...");
    match client.authenticate().await {
        Ok(()) => println!("Authentication successful!"),
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
            std::process::exit(1);
        }
    }

    // Perform the lookup
    let dxcc_info = match lookup_type {
        LookupType::Entity(entity) => {
            println!("Looking up DXCC entity: {}", entity);
            client.lookup_dxcc_entity(entity).await
        }
        LookupType::Callsign(ref callsign) => {
            println!("Looking up DXCC entity for callsign: {}", callsign);
            client.lookup_dxcc_by_callsign(callsign).await
        }
    };

    match dxcc_info {
        Ok(info) => {
            println!("\n=== DXCC Entity Information ===");
            println!("Entity Number: {}", info.dxcc);
            println!("Name: {}", info.name);

            if let Some(cc) = &info.cc {
                println!("Country Code (2-letter): {}", cc);
            }

            if let Some(ccc) = &info.ccc {
                println!("Country Code (3-letter): {}", ccc);
            }

            if let Some(continent) = &info.continent {
                println!("Continent: {}", continent);
            }

            // Zone information
            if let (Some(cq), Some(itu)) = (info.cqzone, info.ituzone) {
                println!("Zones: CQ {}, ITU {}", cq, itu);
            } else {
                if let Some(cq) = info.cqzone {
                    println!("CQ Zone: {}", cq);
                }
                if let Some(itu) = info.ituzone {
                    println!("ITU Zone: {}", itu);
                }
            }

            // Timezone information
            if let Some(tz) = &info.timezone {
                println!("Timezone: {} UTC", tz);
                if let Some(hours) = info.timezone_hours() {
                    if hours >= 0.0 {
                        println!("  (UTC+{:.1} hours)", hours);
                    } else {
                        println!("  (UTC{:.1} hours)", hours);
                    }
                }
            }

            // Geographic coordinates
            if let Some((lat, lon)) = info.coordinates() {
                println!("Coordinates: {:.4}°, {:.4}°", lat, lon);

                // Format coordinates in a more readable way
                let lat_dir = if lat >= 0.0 { "N" } else { "S" };
                let lon_dir = if lon >= 0.0 { "E" } else { "W" };
                println!(
                    "  ({:.4}° {}, {:.4}° {})",
                    lat.abs(),
                    lat_dir,
                    lon.abs(),
                    lon_dir
                );
            }

            // Special notes
            if let Some(notes) = &info.notes {
                println!("Notes: {}", notes);
            }

            // Additional context for the lookup
            match lookup_type {
                LookupType::Entity(_) => {
                    println!("\nThis entity information represents the country/territory");
                    println!("associated with DXCC entity {}.", info.dxcc);
                }
                LookupType::Callsign(ref callsign) => {
                    println!("\nThis DXCC entity was determined by prefix matching");
                    println!("for callsign: {}", callsign);
                    println!("The actual callsign location may be different due to");
                    println!("portable operations, reciprocal agreements, etc.");
                }
            }
        }
        Err(QrzXmlError::DxccNotFound { entity }) => {
            eprintln!("DXCC entity not found: {}", entity);
            std::process::exit(1);
        }
        Err(QrzXmlError::SubscriptionRequired) => {
            eprintln!("A QRZ subscription is required to access DXCC data.");
        }
        Err(e) => {
            eprintln!("DXCC lookup error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
