use clap::ValueEnum;
use eyre::Context;
use eyre::Result;
use reqwest::Client;
use reqwest::StatusCode;
use std::net::ToSocketAddrs;
use std::time::Duration;
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::task;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PingMode {
    Icmp,
    Tcp,
    HttpGet,
    HttpHead,
}

impl PingMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PingMode::Icmp => "icmp",
            PingMode::Tcp => "tcp",
            PingMode::HttpGet => "http-get",
            PingMode::HttpHead => "http-head",
        }
    }

    fn default_port(&self) -> u16 {
        match self {
            PingMode::Tcp => 80,
            PingMode::HttpGet | PingMode::HttpHead => 443,
            PingMode::Icmp => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Destination {
    pub display: String,
    pub host: String,
    pub port: u16,
    pub url: Option<String>,
}

pub fn parse_destination(input: &str, mode: PingMode) -> Destination {
    if input.starts_with("http://") || input.starts_with("https://") {
        let url = input.to_string();
        let without_scheme = url
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        let host_part = without_scheme.split('/').next().unwrap_or(without_scheme);
        let (host_only, port) = if let Some((h, p)) = host_part.split_once(':') {
            let port = p
                .parse()
                .unwrap_or_else(|_| match input.starts_with("https://") {
                    true => 443,
                    false => 80,
                });
            (h.to_string(), port)
        } else {
            let default_port = if input.starts_with("https://") {
                443
            } else {
                80
            };
            (host_part.to_string(), default_port)
        };
        Destination {
            display: input.to_string(),
            host: host_only,
            port,
            url: Some(url),
        }
    } else {
        let (host, port) = if let Some((h, p)) = input.rsplit_once(':') {
            if let Ok(port) = p.parse() {
                (h.to_string(), port)
            } else {
                (input.to_string(), mode.default_port())
            }
        } else {
            (input.to_string(), mode.default_port())
        };
        let url = match mode {
            PingMode::HttpGet | PingMode::HttpHead => Some(format!("https://{}:{}", host, port)),
            _ => None,
        };
        Destination {
            display: input.to_string(),
            host,
            port,
            url,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PingOutcome {
    pub host: String,
    pub mode: PingMode,
    pub latency: Option<Duration>,
    pub status: Option<StatusCode>,
    pub success: bool,
    pub error: Option<String>,
}

impl PingOutcome {
    pub fn success(
        host: &str,
        mode: PingMode,
        latency: Duration,
        status: Option<StatusCode>,
    ) -> Self {
        Self {
            host: host.to_string(),
            mode,
            latency: Some(latency),
            status,
            success: true,
            error: None,
        }
    }

    pub fn failure(host: &str, mode: PingMode, error: eyre::Error) -> Self {
        Self {
            host: host.to_string(),
            mode,
            latency: None,
            status: None,
            success: false,
            error: Some(error.to_string()),
        }
    }
}

pub fn build_http_client() -> Result<Client> {
    Ok(Client::builder().timeout(Duration::from_secs(10)).build()?)
}

pub async fn execute_ping(
    client: &Client,
    mode: PingMode,
    destination: &Destination,
) -> PingOutcome {
    match mode {
        PingMode::Tcp => match tcp_ping(&destination.host, destination.port).await {
            Ok(latency) => PingOutcome::success(&destination.host, mode, latency, None),
            Err(e) => PingOutcome::failure(&destination.host, mode, eyre::eyre!(e)),
        },
        PingMode::HttpGet => match http_ping(
            client,
            destination.url.as_deref().unwrap_or(&destination.display),
            false,
        )
        .await
        {
            Ok((latency, status)) => {
                PingOutcome::success(&destination.host, mode, latency, Some(status))
            }
            Err(e) => PingOutcome::failure(&destination.host, mode, eyre::eyre!(e)),
        },
        PingMode::HttpHead => match http_ping(
            client,
            destination.url.as_deref().unwrap_or(&destination.display),
            true,
        )
        .await
        {
            Ok((latency, status)) => {
                PingOutcome::success(&destination.host, mode, latency, Some(status))
            }
            Err(e) => PingOutcome::failure(&destination.host, mode, eyre::eyre!(e)),
        },
        PingMode::Icmp => match icmp_ping(&destination.host).await {
            Ok(latency) => PingOutcome::success(&destination.host, mode, latency, None),
            Err(e) => PingOutcome::failure(&destination.host, mode, e),
        },
    }
}

async fn tcp_ping(
    host: &str,
    port: u16,
) -> Result<Duration, Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();
    let _stream = TcpStream::connect((host, port)).await?;
    Ok(start.elapsed())
}

async fn http_ping(
    client: &Client,
    url: &str,
    use_head: bool,
) -> Result<(Duration, StatusCode), reqwest::Error> {
    let start = Instant::now();
    let response = if use_head {
        client.head(url).send().await?
    } else {
        client.get(url).send().await?
    };
    Ok((start.elapsed(), response.status()))
}

async fn icmp_ping(host: &str) -> Result<Duration> {
    let host = host.to_string();
    task::spawn_blocking(move || {
        use ping::ping;
        let addr = format!("{}:0", host);
        let ip = addr
            .to_socket_addrs()
            .wrap_err("Unable to resolve host")?
            .next()
            .ok_or_else(|| eyre::eyre!("No IP resolved for host"))?
            .ip();
        let start = Instant::now();
        let timeout = Some(Duration::from_secs(2));
        ping(ip, timeout, None, None, None, None).wrap_err("ICMP echo failed")?;
        Ok(start.elapsed())
    })
    .await
    .unwrap_or_else(|e| Err(eyre::eyre!("ICMP task failed: {e}")))
}
