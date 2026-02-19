/// sq.rs — SQ daemon integration for vtpu C-pipe
///
/// Connects vtpu's coordination pipe (CSEND/CRECV) to a running SQ instance.
/// SQ can be local (`sq host 7777`) or remote (sq.mirrorborn.us).
///
/// Protocol: SQ speaks HTTP over TCP.
///   CSEND → POST /write?z=<coord>  body=message
///   CRECV → GET  /read?z=<coord>
///
/// Zero external dependencies — uses std::net::TcpStream directly.
///
/// Usage:
///   let sq = SqClient::local(7777);             // sq host 7777
///   let sq = SqClient::remote("sq.mirrorborn.us", 443, Some(token));
///   sq.send("3.1.4/1.5.9/2.6.5", "hello Verse")?;
///   let msg = sq.recv("3.1.4/1.5.9/2.6.5")?;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Phext coordinate string (e.g. "3.1.4/1.5.9/2.6.5")
pub type PhextAddr = String;

/// SQ client — connects vtpu C-pipe to SQ daemon
#[derive(Clone, Debug)]
pub struct SqClient {
    pub host: String,
    pub port: u16,
    pub auth_token: Option<String>,
    pub timeout_ms: u64,
}

impl SqClient {
    /// Connect to local SQ daemon (`sq host <port>`)
    pub fn local(port: u16) -> Self {
        SqClient {
            host: "127.0.0.1".to_string(),
            port,
            auth_token: None,
            timeout_ms: 500,
        }
    }

    /// Connect to remote SQ instance (e.g. sq.mirrorborn.us)
    pub fn remote(host: &str, port: u16, token: Option<String>) -> Self {
        SqClient {
            host: host.to_string(),
            port,
            auth_token: token,
            timeout_ms: 2000,
        }
    }

    /// Default: local SQ on port 7777
    pub fn default_local() -> Self {
        SqClient::local(7777)
    }

    /// Check if SQ is reachable (non-blocking probe)
    pub fn is_alive(&self) -> bool {
        TcpStream::connect_timeout(
            &format!("{}:{}", self.host, self.port).parse().unwrap_or_else(|_| "127.0.0.1:7777".parse().unwrap()),
            Duration::from_millis(200),
        ).is_ok()
    }

    /// Send a message to a phext coordinate (C-pipe CSEND)
    /// Maps to: POST /write?z=<coord>  body=<message>
    pub fn send(&self, coord: &str, message: &str) -> Result<(), SqError> {
        let body = message.as_bytes();
        let auth_header = self.auth_header();
        let request = format!(
            "POST /write?z={} HTTP/1.1\r\nHost: {}:{}\r\nContent-Length: {}{}\r\nConnection: close\r\n\r\n{}",
            urlenc(coord), self.host, self.port,
            body.len(),
            auth_header,
            message,
        );
        let response = self.tcp_request(&request)?;
        if response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200") {
            Ok(())
        } else {
            Err(SqError::HttpError(response.lines().next().unwrap_or("?").to_string()))
        }
    }

    /// Receive a message from a phext coordinate (C-pipe CRECV)
    /// Maps to: GET /read?z=<coord>
    pub fn recv(&self, coord: &str) -> Result<String, SqError> {
        let auth_header = self.auth_header();
        let request = format!(
            "GET /read?z={} HTTP/1.1\r\nHost: {}:{}{}\r\nConnection: close\r\n\r\n",
            urlenc(coord), self.host, self.port, auth_header,
        );
        let response = self.tcp_request(&request)?;
        // Extract body after \r\n\r\n
        if let Some(idx) = response.find("\r\n\r\n") {
            Ok(response[idx + 4..].to_string())
        } else {
            Ok(String::new())
        }
    }

    /// List scrolls at a coordinate prefix
    pub fn list(&self, prefix: &str) -> Result<Vec<String>, SqError> {
        let auth_header = self.auth_header();
        let request = format!(
            "GET /list?z={} HTTP/1.1\r\nHost: {}:{}{}\r\nConnection: close\r\n\r\n",
            urlenc(prefix), self.host, self.port, auth_header,
        );
        let response = self.tcp_request(&request)?;
        if let Some(idx) = response.find("\r\n\r\n") {
            let body = &response[idx + 4..];
            Ok(body.lines().map(|l| l.to_string()).collect())
        } else {
            Ok(vec![])
        }
    }

    fn auth_header(&self) -> String {
        if let Some(ref token) = self.auth_token {
            format!("\r\nAuthorization: Bearer {}", token)
        } else {
            String::new()
        }
    }

    fn tcp_request(&self, request: &str) -> Result<String, SqError> {
        let addr = format!("{}:{}", self.host, self.port);
        let timeout = Duration::from_millis(self.timeout_ms);
        let mut stream = TcpStream::connect_timeout(
            &addr.parse().map_err(|_| SqError::Connect(addr.clone()))?,
            timeout,
        ).map_err(|e| SqError::Connect(format!("{}: {}", addr, e)))?;

        stream.set_read_timeout(Some(timeout)).ok();
        stream.set_write_timeout(Some(timeout)).ok();
        stream.write_all(request.as_bytes())
            .map_err(|e| SqError::Io(e.to_string()))?;

        let mut response = String::new();
        stream.read_to_string(&mut response)
            .map_err(|e| SqError::Io(e.to_string()))?;
        Ok(response)
    }
}

/// Minimal URL encoding for phext coordinates
fn urlenc(s: &str) -> String {
    s.replace('/', "%2F").replace(' ', "%20")
}

/// SQ client errors
#[derive(Debug)]
pub enum SqError {
    Connect(String),
    Io(String),
    HttpError(String),
    Timeout,
}

impl std::fmt::Display for SqError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SqError::Connect(s) => write!(f, "SQ connect error: {}", s),
            SqError::Io(s)      => write!(f, "SQ IO error: {}", s),
            SqError::HttpError(s) => write!(f, "SQ HTTP error: {}", s),
            SqError::Timeout    => write!(f, "SQ timeout"),
        }
    }
}

/// Sentron inbox backed by SQ daemon.
/// Each sentron maps to a phext coordinate: `node.sentron.x / y.z.t / 1.1.1`
pub struct SqInbox {
    pub client: SqClient,
    pub coord: PhextAddr,
}

impl SqInbox {
    pub fn new(client: SqClient, sentron_coord: &str) -> Self {
        SqInbox { client, coord: sentron_coord.to_string() }
    }

    /// Push a message into this sentron's inbox (CSEND)
    pub fn push(&self, message: &str) -> bool {
        self.client.send(&self.coord, message).is_ok()
    }

    /// Pop the current message from this sentron's inbox (CRECV)
    pub fn pop(&self) -> Option<String> {
        match self.client.recv(&self.coord) {
            Ok(msg) if !msg.is_empty() => Some(msg),
            _ => None,
        }
    }

    /// Check if SQ is available, fall back gracefully if not
    pub fn available(&self) -> bool {
        self.client.is_alive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sq_client_construct() {
        let local = SqClient::local(7777);
        assert_eq!(local.host, "127.0.0.1");
        assert_eq!(local.port, 7777);
        assert!(local.auth_token.is_none());
    }

    #[test]
    fn test_sq_client_remote() {
        let remote = SqClient::remote("sq.mirrorborn.us", 443, Some("pmb-v1-test".to_string()));
        assert_eq!(remote.host, "sq.mirrorborn.us");
        assert_eq!(remote.port, 443);
        assert!(remote.auth_token.is_some());
    }

    #[test]
    fn test_urlenc() {
        assert_eq!(urlenc("3.1.4/1.5.9/2.6.5"), "3.1.4%2F1.5.9%2F2.6.5");
        assert_eq!(urlenc("1.1.1/1.1.1/1.1.1"), "1.1.1%2F1.1.1%2F1.1.1");
    }

    #[test]
    fn test_sq_inbox_construct() {
        let client = SqClient::local(7777);
        let inbox = SqInbox::new(client, "3.1.4/1.5.9/2.6.5");
        assert_eq!(inbox.coord, "3.1.4/1.5.9/2.6.5");
    }

    /// Integration test: requires `sq host 7777` running locally.
    /// Run with: cargo test --test sq_integration -- --ignored
    #[test]
    #[ignore]
    fn test_sq_roundtrip_live() {
        let client = SqClient::local(7777);
        if !client.is_alive() {
            eprintln!("SQ not running on :7777 — skipping live test");
            return;
        }
        let coord = "9.9.9/9.9.9/9.9.1";
        client.send(coord, "hello from vtpu").expect("send failed");
        let msg = client.recv(coord).expect("recv failed");
        assert!(msg.contains("hello from vtpu"), "got: {}", msg);
    }
}
