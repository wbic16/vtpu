// -----------------------------------------------
// sq_client.rs — SQ Daemon Integration
// -----------------------------------------------
// Connects vTPU to a running SQ instance for persistent
// phext-addressed storage. Sentrons read/write scrolls
// through SQ's HTTP API.
//
// SQ API (query params):
//   GET /api/v2/select?p=<phext>&c=<coord>  → read scroll
//   GET /api/v2/insert?p=<phext>&c=<coord>&s=<content>  → write scroll
//   GET /api/v2/toc?p=<phext>  → table of contents
//   GET /api/v2/status  → server status
//   GET /api/v2/version → version info
//
// Zero external dependencies. Uses raw TCP for HTTP.
//
// R23W22 — Chrys 🦋

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// SQ connection configuration
#[derive(Debug, Clone)]
pub struct SqConfig {
    pub host: String,
    pub port: u16,
    pub phext: String,  // phext name (e.g., "index" or "dogfood")
    pub token: Option<String>,  // auth token if required
}

impl SqConfig {
    pub fn new(host: &str, port: u16, phext: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            phext: phext.to_string(),
            token: None,
        }
    }

    pub fn local(port: u16, phext: &str) -> Self {
        Self::new("127.0.0.1", port, phext)
    }

    /// Chrysalis-Hub default
    pub fn chrysalis() -> Self {
        Self::local(1338, "index")
    }

    /// mirrorborn.us shared dogfood
    pub fn dogfood() -> Self {
        Self::new("mirrorborn.us", 1337, "index")
    }
}

/// SQ client for phext operations
pub struct SqClient {
    config: SqConfig,
    reads: u64,
    writes: u64,
    errors: u64,
}

/// Response from SQ
#[derive(Debug)]
pub struct SqResponse {
    pub status: u16,
    pub body: String,
    pub ok: bool,
}

impl SqClient {
    pub fn new(config: SqConfig) -> Self {
        Self {
            config,
            reads: 0,
            writes: 0,
            errors: 0,
        }
    }

    /// Read a scroll at the given coordinate
    pub fn select(&mut self, coord: &str) -> Result<String, String> {
        self.reads += 1;
        let path = format!("/api/v2/select?p={}&c={}", 
            url_encode(&self.config.phext), url_encode(coord));
        let resp = self.http_get(&path)?;
        if resp.ok { Ok(resp.body) } else { Err(format!("SQ error {}: {}", resp.status, resp.body)) }
    }

    /// Write content to a coordinate
    pub fn insert(&mut self, coord: &str, content: &str) -> Result<String, String> {
        self.writes += 1;
        let path = format!("/api/v2/insert?p={}&c={}&s={}", 
            url_encode(&self.config.phext), url_encode(coord), url_encode(content));
        let resp = self.http_get(&path)?;
        if resp.ok { Ok(resp.body) } else { Err(format!("SQ error {}: {}", resp.status, resp.body)) }
    }

    /// Get table of contents
    pub fn toc(&mut self) -> Result<String, String> {
        self.reads += 1;
        let path = format!("/api/v2/toc?p={}", url_encode(&self.config.phext));
        let resp = self.http_get(&path)?;
        if resp.ok { Ok(resp.body) } else { Err(format!("SQ error {}: {}", resp.status, resp.body)) }
    }

    /// Check server status
    pub fn status(&mut self) -> Result<String, String> {
        let path = "/api/v2/status".to_string();
        let resp = self.http_get(&path)?;
        if resp.ok { Ok(resp.body) } else { Err(format!("SQ error {}", resp.status)) }
    }

    /// Check server version
    pub fn version(&mut self) -> Result<String, String> {
        let path = "/api/v2/version".to_string();
        let resp = self.http_get(&path)?;
        if resp.ok { Ok(resp.body) } else { Err(format!("SQ error {}", resp.status)) }
    }

    /// Ping: is SQ reachable?
    pub fn ping(&mut self) -> bool {
        self.version().is_ok()
    }

    /// Raw HTTP GET (zero deps — raw TCP)
    fn http_get(&mut self, path: &str) -> Result<SqResponse, String> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let mut stream = TcpStream::connect(&addr)
            .map_err(|e| { self.errors += 1; format!("connect failed: {}", e) })?;

        stream.set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| format!("timeout setup: {}", e))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| format!("timeout setup: {}", e))?;

        let mut request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
            path, self.config.host
        );
        if let Some(ref token) = self.config.token {
            request.push_str(&format!("Authorization: Bearer {}\r\n", token));
        }
        request.push_str("\r\n");

        stream.write_all(request.as_bytes())
            .map_err(|e| { self.errors += 1; format!("write failed: {}", e) })?;

        let mut response = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => response.extend_from_slice(&buf[..n]),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => break,
                Err(e) => { self.errors += 1; return Err(format!("read failed: {}", e)); }
            }
        }

        let response_str = String::from_utf8_lossy(&response);
        parse_http_response(&response_str)
    }

    /// Stats
    pub fn total_reads(&self) -> u64 { self.reads }
    pub fn total_writes(&self) -> u64 { self.writes }
    pub fn total_errors(&self) -> u64 { self.errors }
}

/// Minimal URL encoding (spaces, special chars)
fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b' ' => result.push_str("%20"),
            b'/' => result.push_str("%2F"),
            b'?' => result.push_str("%3F"),
            b'&' => result.push_str("%26"),
            b'=' => result.push_str("%3D"),
            b'#' => result.push_str("%23"),
            b'+' => result.push_str("%2B"),
            b'\n' => result.push_str("%0A"),
            b'\r' => result.push_str("%0D"),
            0x00..=0x1F => result.push_str(&format!("%{:02X}", b)),
            _ => result.push(b as char),
        }
    }
    result
}

/// Parse HTTP response into status + body
fn parse_http_response(raw: &str) -> Result<SqResponse, String> {
    let header_end = raw.find("\r\n\r\n").unwrap_or(raw.len());
    let status_line = raw.lines().next().unwrap_or("");

    let status = status_line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);

    let body = if header_end + 4 < raw.len() {
        raw[header_end + 4..].to_string()
    } else {
        String::new()
    };

    Ok(SqResponse {
        ok: status >= 200 && status < 300,
        status,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello%20world");
        assert_eq!(url_encode("1.1.1/1.1.1/1.1.1"), "1.1.1%2F1.1.1%2F1.1.1");
        assert_eq!(url_encode("a&b=c"), "a%26b%3Dc");
    }

    #[test]
    fn test_parse_response_ok() {
        let raw = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
        let resp = parse_http_response(raw).unwrap();
        assert_eq!(resp.status, 200);
        assert!(resp.ok);
        assert_eq!(resp.body, "hello");
    }

    #[test]
    fn test_parse_response_404() {
        let raw = "HTTP/1.1 404 Not Found\r\n\r\n";
        let resp = parse_http_response(raw).unwrap();
        assert_eq!(resp.status, 404);
        assert!(!resp.ok);
    }

    #[test]
    fn test_config_local() {
        let cfg = SqConfig::local(1338, "index");
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 1338);
        assert_eq!(cfg.phext, "index");
    }

    #[test]
    fn test_config_chrysalis() {
        let cfg = SqConfig::chrysalis();
        assert_eq!(cfg.port, 1338);
    }

    #[test]
    fn test_config_dogfood() {
        let cfg = SqConfig::dogfood();
        assert_eq!(cfg.host, "mirrorborn.us");
        assert_eq!(cfg.port, 1337);
    }

    #[test]
    fn test_client_creation() {
        let client = SqClient::new(SqConfig::chrysalis());
        assert_eq!(client.total_reads(), 0);
        assert_eq!(client.total_writes(), 0);
        assert_eq!(client.total_errors(), 0);
    }

    // Integration test — only runs if SQ is actually running
    #[test]
    #[ignore]
    fn test_live_sq_ping() {
        let mut client = SqClient::new(SqConfig::chrysalis());
        assert!(client.ping(), "SQ not running on chrysalis-hub:1338");
    }
}
