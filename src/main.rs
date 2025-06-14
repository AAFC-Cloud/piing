use clap::Parser;
use color_eyre::{owo_colors::OwoColorize, Result};
use colored::*;
use reqwest::Client;
use std::time::{Duration, Instant};
use tokio::time;
use tokio::net::TcpStream;

#[derive(Parser)]
#[command(name = "piing")]
#[command(about = "A simple HTTP ping utility")]
#[command(version)]
struct Args {
    /// Destination URL to ping
    destination: String,

    /// Refresh interval (e.g., "1s", "500ms", "2.5s")
    #[arg(short, long, default_value = "1s", value_parser=humantime::parse_duration)]
    interval: Duration,

    /// Use TCP connect instead of HTTP for more accurate ping-like measurement
    #[arg(long)]
    tcp: bool,

    /// Use HTTP HEAD instead of GET (no response body)
    #[arg(long)]
    head: bool,

    /// Port to use for TCP ping (default: 80 for HTTP, 443 for HTTPS)
    #[arg(short, long)]
    port: Option<u16>,

    /// Use ICMP echo (real ping) instead of HTTP/TCP
    #[arg(long)]
    icmp: bool,
}

async fn tcp_ping(host: &str, port: u16) -> Result<Duration, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = Instant::now();
    let _stream = TcpStream::connect((host, port)).await?;
    Ok(start_time.elapsed())
}

async fn http_ping(client: &Client, url: &str, use_head: bool) -> Result<(Duration, reqwest::StatusCode), reqwest::Error> {
    let start_time = Instant::now();
    let response = if use_head {
        client.head(url).send().await?
    } else {
        client.get(url).send().await?
    };
    let duration = start_time.elapsed();
    Ok((duration, response.status()))
}

async fn icmp_ping(host: &str) -> Result<Duration, Box<dyn std::error::Error + Send + Sync>> {
    use ping::ping;
    use std::net::ToSocketAddrs;
    let addr = format!("{}:0", host);
    let ip = addr.to_socket_addrs()?.next().ok_or("Unable to resolve host")?.ip();
    let start_time = Instant::now();
    let timeout = Some(Duration::from_secs(2));
    // The ping crate expects: ip, timeout, ttl, ident, seq_cnt, payload
    // We'll use defaults for all except ip and timeout
    match ping(ip, timeout, None, None, None, None) {
        Ok(()) => Ok(start_time.elapsed()),
        Err(e) => Err(Box::new(e)),
    }
}

fn parse_destination(destination: &str) -> (String, String, u16) {
    if destination.starts_with("http://") {
        let host = destination.trim_start_matches("http://");
        let host = host.split('/').next().unwrap_or(host);
        (destination.to_string(), host.to_string(), 80)
    } else if destination.starts_with("https://") {
        let host = destination.trim_start_matches("https://");
        let host = host.split('/').next().unwrap_or(host);
        (destination.to_string(), host.to_string(), 443)
    } else {
        // Assume HTTPS for URL, but extract host for TCP
        (format!("https://{}", destination), destination.to_string(), 443)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let (url, host, default_port) = parse_destination(&args.destination);
    let port = args.port.unwrap_or(default_port);

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    if args.icmp {
        println!(
            "ICMP pinging {} every {}",
            host.cyan(),
            humantime::format_duration(args.interval).cyan()
        );
    } else if args.tcp {
        println!(
            "TCP pinging {}:{} every {}",
            host.cyan(),
            port.to_string().cyan(),
            humantime::format_duration(args.interval).cyan()
        );
    } else {
        println!(
            "{} pinging {} every {}",
            if args.head { "HEAD" } else { "HTTP GET" },
            url.cyan(),
            humantime::format_duration(args.interval).cyan()
        );
    }
    println!();

    loop {
        let current_time = chrono::Local::now().to_rfc2822();

        if args.icmp {
            match icmp_ping(&host).await {
                Ok(duration) => {
                    let duration_str = format!("{:.1}ms", duration.as_micros() as f64 / 1000.0);
                    let colored_duration = if duration.as_millis() > 500 {
                        duration_str.red().to_string()
                    } else if duration.as_millis() > 100 {
                        duration_str.yellow().to_string()
                    } else {
                        duration_str.green().to_string()
                    };
                    println!(
                        "{} - ICMP Echo: {} - Duration: {}",
                        current_time,
                        "SUCCESS".green(),
                        colored_duration
                    );
                }
                Err(e) => {
                    println!(
                        "{} - ICMP Echo: {} - Error: {}",
                        current_time,
                        "FAILED".red(),
                        e.to_string().red()
                    );
                }
            }
        } else if args.tcp {
            // TCP connect measurement - most accurate for ping-like behavior
            match tcp_ping(&host, port).await {
                Ok(duration) => {
                    let duration_str = format!("{:.1}ms", duration.as_micros() as f64 / 1000.0);
                    let colored_duration = if duration.as_millis() > 500 {
                        duration_str.red().to_string()
                    } else if duration.as_millis() > 100 {
                        duration_str.yellow().to_string()
                    } else {
                        duration_str.green().to_string()
                    };

                    println!(
                        "{} - TCP Connect: {} - Duration: {}",
                        current_time,
                        "SUCCESS".green(),
                        colored_duration
                    );
                }
                Err(e) => {
                    println!(
                        "{} - TCP Connect: {} - Error: {}",
                        current_time,
                        "FAILED".red(),
                        e.to_string().red()
                    );
                }
            }
        } else {
            // HTTP measurement
            match http_ping(&client, &url, args.head).await {
                Ok((duration, status_code)) => {
                    let duration_str = format!("{:.1}ms", duration.as_micros() as f64 / 1000.0);
                    let colored_duration = if duration.as_millis() > 500 {
                        duration_str.red().to_string()
                    } else if duration.as_millis() > 100 {
                        duration_str.yellow().to_string()
                    } else {
                        duration_str.green().to_string()
                    };

                    println!(
                        "{} - Status: {} - Duration: {}",
                        current_time,
                        status_code.to_string().green(),
                        colored_duration
                    );
                }
                Err(e) => {
                    println!(
                        "{} - Error: {}",
                        current_time,
                        e.to_string().red()
                    );
                }
            }
        }

        time::sleep(args.interval).await;
    }
}
