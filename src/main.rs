use clap::Parser;
use color_eyre::{owo_colors::OwoColorize, Result};
use colored::*;
use reqwest::Client;
use std::time::{Duration, Instant};
use tokio::time;
use tokio::net::TcpStream;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Mode {
    Icmp,
    Tcp,
    HttpGet,
    HttpHead,
}

impl Mode {
    fn description(&self) -> &'static str {
        match self {
            Mode::Icmp => "ICMP",
            Mode::Tcp => "TCP",
            Mode::HttpGet => "HTTP GET",
            Mode::HttpHead => "HEAD",
        }
    }
}

#[derive(Parser)]
#[command(name = "piing")]
#[command(about = "A simple HTTP ping utility")]
#[command(version)]
struct Args {
    /// Destination URL or host to ping
    destination: String,

    /// Refresh interval (e.g., "1s", "500ms", "2.5s")
    #[arg(short, long, default_value = "1s", value_parser=humantime::parse_duration)]
    interval: Duration,

    /// Ping mode: icmp, tcp, http-get, http-head
    #[arg(long, value_enum, default_value_t = Mode::Icmp)]
    mode: Mode,
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

fn parse_destination(destination: &str, mode: Mode) -> (String, String, u16) {
    // Returns (url, host, port)
    if destination.starts_with("http://") {
        let host = destination.trim_start_matches("http://");
        let host = host.split('/').next().unwrap_or(host);
        let port = if let Some(port_str) = host.split(':').nth(1) {
            port_str.parse().unwrap_or(80)
        } else {
            80
        };
        (destination.to_string(), host.split(':').next().unwrap_or(host).to_string(), port)
    } else if destination.starts_with("https://") {
        let host = destination.trim_start_matches("https://");
        let host = host.split('/').next().unwrap_or(host);
        let port = if let Some(port_str) = host.split(':').nth(1) {
            port_str.parse().unwrap_or(443)
        } else {
            443
        };
        (destination.to_string(), host.split(':').next().unwrap_or(host).to_string(), port)
    } else {
        // For TCP, allow host:port, otherwise default to 443 for ICMP/HTTP
        let (host, port) = if let Some((h, p)) = destination.rsplit_once(':') {
            if let Ok(port) = p.parse() {
                (h.to_string(), port)
            } else {
                (destination.to_string(), match mode {
                    Mode::Tcp => 80,
                    Mode::HttpGet | Mode::HttpHead => 443,
                    Mode::Icmp => 443,
                })
            }
        } else {
            (destination.to_string(), match mode {
                Mode::Tcp => 80,
                Mode::HttpGet | Mode::HttpHead => 443,
                Mode::Icmp => 443,
            })
        };
        let url = match mode {
            Mode::HttpGet | Mode::HttpHead => format!("https://{}:{}", host, port),
            _ => host.clone(),
        };
        (url, host, port)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let mode = args.mode;

    let (url, host, port) = parse_destination(&args.destination, mode);

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    println!(
        "{} pinging {}{} every {}",
        mode.description(),
        host.cyan(),
        if let Mode::Tcp = mode {
            format!(":{}", port).cyan().to_string()
        } else {
            String::new()
        },
        humantime::format_duration(args.interval).cyan()
    );
    println!();

    loop {
        let current_time = chrono::Local::now().to_rfc2822();

        match mode {
            Mode::Icmp => {
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
            }
            Mode::Tcp => {
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
            }
            Mode::HttpHead => {
                match http_ping(&client, &url, true).await {
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
            Mode::HttpGet => {
                match http_ping(&client, &url, false).await {
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
        }

        time::sleep(args.interval).await;
    }
}
