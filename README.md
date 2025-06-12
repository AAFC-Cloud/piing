<div align="center">
    <h1>📡 Piing</h1>
    <br/>

[Voir la version française](./README.fr_ca.md)

</div>

## Description

A modern HTTP and TCP ping utility written in Rust. Piing provides multiple measurement methods to accurately test network connectivity and latency when ICMP packets are not available.

## Features

- **TCP Connect Mode**: Most accurate ping-like measurement using raw TCP connections
- **HTTP GET/HEAD Requests**: Traditional HTTP-based connectivity testing
- **Colorized Output**: Visual feedback with color-coded response times
- **Flexible Timing**: Human-readable interval parsing (e.g., "1s", "500ms", "2.5s")
- **Multiple Protocols**: Support for HTTP and HTTPS with automatic detection

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)

### Build from Source

```powershell
git clone https://github.com/AAFC-Cloud/piing
cd piing
cargo build --release
```

The executable will be available at `target/release/piing.exe`.

## Usage

### Basic HTTP Ping
```powershell
piing google.com
```

### TCP Connect Ping (Most Accurate)
```powershell
piing google.com --tcp
```

### HTTP HEAD Requests (Faster than GET)
```powershell
piing google.com --head
```

### Custom Interval
```powershell
piing google.com --interval 500ms
```

### Custom Port for TCP Ping
```powershell
piing google.com --tcp --port 443
```

### Complete Example
```powershell
piing https://example.com --tcp --port 443 --interval 2s
```

## Command Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--tcp` | | Use TCP connect for most accurate ping-like measurement |
| `--head` | | Use HTTP HEAD instead of GET (no response body) |
| `--interval` | `-i` | Refresh interval (e.g., "1s", "500ms", "2.5s") |
| `--port` | `-p` | Port to use for TCP ping (default: 80 for HTTP, 443 for HTTPS) |
| `--help` | `-h` | Show help information |

## Output

The utility displays timestamped results with color-coded response times:

- **Green**: Response time < 100ms
- **Yellow**: Response time 100-500ms  
- **Red**: Response time > 500ms

### Example Output

```
TCP pinging google.com:443 every 1s

Thu, 12 Jun 2025 08:48:10 -0400 - TCP Connect: SUCCESS - Duration: 29.2ms
Thu, 12 Jun 2025 08:48:11 -0400 - TCP Connect: SUCCESS - Duration: 28.3ms
Thu, 12 Jun 2025 08:48:12 -0400 - TCP Connect: SUCCESS - Duration: 33.5ms
```

## Measurement Accuracy

### TCP Connect Mode (Recommended)
- **Most accurate** for ping-like measurements
- Measures only network + TCP handshake time
- Excludes HTTP/TLS overhead and server processing
- Closest equivalent to ICMP ping when ICMP is unavailable

### HTTP HEAD Mode
- More accurate than GET requests
- Includes TLS handshake but no response body download
- Good balance between accuracy and protocol compliance

### HTTP GET Mode
- Full HTTP request/response cycle
- Includes all network, TLS, HTTP, and server processing overhead
- Useful for testing complete application stack

## Copyright

Copyright belongs to © His Majesty the King in Right of Canada, as represented by the Minister of Agriculture and Agri-Food, 2025.