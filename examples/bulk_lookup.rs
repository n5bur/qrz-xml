//! Advanced bulk lookup example for the QRZ library.
//!
//! This example demonstrates:
//! - Bulk callsign lookups with rate limiting
//! - Comprehensive error handling and recovery
//! - Progress reporting and statistics
//! - CSV output generation
//! - Graceful handling of mixed success/failure scenarios
//!
//! Usage:
//! ```
//! QRZ_USERNAME=your_username QRZ_PASSWORD=your_password cargo run --example bulk_lookup -- callsigns.txt output.csv
//! ```
//!
//! Input file format (one callsign per line):
//! ```
//! AA7BQ
//! W1AW
//! VK2DEF
//! JA1ABC
//! ```

use qrz_xml::{ApiVersion, CallsignInfo, QrzXmlClient, QrzXmlError};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct LookupResult {
    callsign: String,
    success: bool,
    info: Option<CallsignInfo>,
    error: Option<String>,
    lookup_time: Duration,
}

#[derive(Debug, Default)]
struct Statistics {
    total: usize,
    successful: usize,
    failed: usize,
    not_found: usize,
    subscription_required: usize,
    other_errors: usize,
    total_time: Duration,
}

impl Statistics {
    fn add_result(&mut self, result: &LookupResult) {
        self.total += 1;
        self.total_time += result.lookup_time;

        if result.success {
            self.successful += 1;
        } else {
            self.failed += 1;

            if let Some(error) = &result.error {
                if error.contains("not found") {
                    self.not_found += 1;
                } else if error.contains("subscription") {
                    self.subscription_required += 1;
                } else {
                    self.other_errors += 1;
                }
            }
        }
    }

    fn print_summary(&self) {
        println!("\n=== Lookup Statistics ===");
        println!("Total lookups: {}", self.total);
        println!(
            "Successful: {} ({:.1}%)",
            self.successful,
            self.successful as f64 / self.total as f64 * 100.0
        );
        println!(
            "Failed: {} ({:.1}%)",
            self.failed,
            self.failed as f64 / self.total as f64 * 100.0
        );

        if self.not_found > 0 {
            println!("  Not found: {}", self.not_found);
        }
        if self.subscription_required > 0 {
            println!("  Subscription required: {}", self.subscription_required);
        }
        if self.other_errors > 0 {
            println!("  Other errors: {}", self.other_errors);
        }

        if self.total > 0 {
            let avg_time = self.total_time / self.total as u32;
            println!("Average lookup time: {:.2}ms", avg_time.as_millis());
            println!("Total time: {:.2}s", self.total_time.as_secs_f64());
        }
    }
}

fn read_callsigns_from_file<P: AsRef<Path>>(
    filename: P,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut callsigns = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let callsign = line.trim().to_uppercase();
        if !callsign.is_empty() && !callsign.starts_with('#') {
            callsigns.push(callsign);
        }
    }

    Ok(callsigns)
}

async fn lookup_with_retry(
    client: &QrzXmlClient,
    callsign: &str,
    max_retries: u32,
) -> LookupResult {
    let start_time = Instant::now();
    let mut last_error = None;

    for attempt in 1..=max_retries {
        match client.lookup_callsign(callsign).await {
            Ok(info) => {
                return LookupResult {
                    callsign: callsign.to_string(),
                    success: true,
                    info: Some(info),
                    error: None,
                    lookup_time: start_time.elapsed(),
                };
            }
            Err(e) => {
                last_error = Some(e.to_string());

                // Don't retry certain errors
                match e {
                    QrzXmlError::CallsignNotFound { .. }
                    | QrzXmlError::SubscriptionRequired
                    | QrzXmlError::AuthenticationFailed { .. }
                    | QrzXmlError::ConnectionRefused => break,
                    _ => {}
                }

                if attempt < max_retries {
                    // Exponential backoff for retries
                    let delay = Duration::from_millis(1000 * 2_u64.pow(attempt - 1));
                    sleep(delay).await;
                }
            }
        }
    }

    LookupResult {
        callsign: callsign.to_string(),
        success: false,
        info: None,
        error: last_error,
        lookup_time: start_time.elapsed(),
    }
}

fn write_csv_output<P: AsRef<Path>>(
    filename: P,
    results: &[LookupResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(filename)?;

    // Write CSV header
    writeln!(
        file,
        "callsign,success,name,country,grid,lat,lon,email,class,dxcc,error"
    )?;

    for result in results {
        if let Some(info) = &result.info {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{},{},{},",
                result.callsign,
                result.success,
                info.full_name().unwrap_or_default().replace(',', ";"),
                info.country.as_deref().unwrap_or("").replace(',', ";"),
                info.grid.as_deref().unwrap_or(""),
                info.lat.map(|l| l.to_string()).unwrap_or_default(),
                info.lon.map(|l| l.to_string()).unwrap_or_default(),
                info.email.as_deref().unwrap_or(""),
                info.class.as_deref().unwrap_or(""),
                info.dxcc.map(|d| d.to_string()).unwrap_or_default(),
            )?;
        } else {
            writeln!(
                file,
                "{},{},,,,,,,,,{}",
                result.callsign,
                result.success,
                result.error.as_deref().unwrap_or("").replace(',', ";"),
            )?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        eprintln!("Example: {} callsigns.txt results.csv", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];

    // Get credentials
    let username = env::var("QRZ_USERNAME").expect("QRZ_USERNAME environment variable must be set");
    let password = env::var("QRZ_PASSWORD").expect("QRZ_PASSWORD environment variable must be set");

    // Read callsigns from file
    println!("Reading callsigns from: {}", input_file);
    let callsigns = read_callsigns_from_file(input_file)?;
    println!("Found {} callsigns to look up", callsigns.len());

    if callsigns.is_empty() {
        eprintln!("No callsigns found in input file");
        std::process::exit(1);
    }

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

    // Show session info
    if let Some((count, sub_exp)) = client.session_info().await {
        println!("Session info:");
        if let Some(count) = count {
            println!("  Lookups used today: {}", count);
        }
        if let Some(sub_exp) = sub_exp {
            println!("  Subscription expires: {}", sub_exp);
        }
    }

    println!("\nStarting bulk lookup with rate limiting...");
    println!("This may take a while for large lists.\n");

    let mut results = Vec::new();
    let mut stats = Statistics::default();
    let total_start = Instant::now();

    for (i, callsign) in callsigns.iter().enumerate() {
        // Progress indicator
        if i % 10 == 0 || i == callsigns.len() - 1 {
            println!(
                "Progress: {}/{} ({:.1}%)",
                i + 1,
                callsigns.len(),
                (i + 1) as f64 / callsigns.len() as f64 * 100.0
            );
        }

        // Perform lookup with retry logic
        let result = lookup_with_retry(&client, callsign, 3).await;

        // Update statistics
        stats.add_result(&result);

        // Print result
        if result.success {
            if let Some(info) = &result.info {
                println!(
                    "  ✓ {} - {}",
                    callsign,
                    info.full_name().unwrap_or_default()
                );
            }
        } else {
            println!(
                "  ✗ {} - {}",
                callsign,
                result.error.as_deref().unwrap_or("Unknown error")
            );
        }

        results.push(result);

        // Rate limiting - be respectful to QRZ servers
        if i < callsigns.len() - 1 {
            sleep(Duration::from_millis(500)).await;
        }
    }

    let total_elapsed = total_start.elapsed();

    // Print final statistics
    stats.print_summary();
    println!("Wall clock time: {:.2}s", total_elapsed.as_secs_f64());

    // Write results to CSV
    println!("\nWriting results to: {}", output_file);
    write_csv_output(output_file, &results)?;

    println!("Bulk lookup completed successfully!");

    // Provide guidance on results
    if stats.subscription_required > 0 {
        println!("\nNote: Some lookups failed due to subscription requirements.");
        println!("Consider upgrading to a QRZ Logbook Data subscription for complete access.");
    }

    if stats.not_found > 0 {
        println!(
            "\nNote: {} callsigns were not found in the QRZ database.",
            stats.not_found
        );
    }

    Ok(())
}
