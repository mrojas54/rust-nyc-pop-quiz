//! The Discord backend's transport: percent- and form-encoding, and a small
//! HTTP/1.1 client over TCP or TLS that reads one response. The parent decides
//! what to send and what an answer means; nothing here retries or reads a
//! payload's fields.

use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

use super::Secret;

const MAX_BODY: usize = 64 * 1024;

// --------------------------------------------------------------------------
// Encoding.
// --------------------------------------------------------------------------

/// Percent-encode everything but RFC 3986's unreserved characters.
pub(super) fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// `application/x-www-form-urlencoded`.
pub(super) fn form(pairs: &[(&str, &str)]) -> Vec<u8> {
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<_>>()
        .join("&")
        .into_bytes()
}

// --------------------------------------------------------------------------
// HTTP/1.1: one request per connection, over TCP or TLS. Enough for three
// Discord routes: Content-Length, chunked or to-EOF bodies, no redirects.
// --------------------------------------------------------------------------

pub(super) struct Req {
    method: &'static str,
    url: String,
    bearer: Option<Secret>,
    form: Option<Vec<u8>>,
}

impl Req {
    pub(super) fn get(url: &str, bearer: &str) -> Req {
        Req {
            method: "GET",
            url: url.to_string(),
            bearer: Some(Secret::new(bearer.to_string())),
            form: None,
        }
    }

    pub(super) fn form(url: &str, body: Vec<u8>) -> Req {
        Req {
            method: "POST",
            url: url.to_string(),
            bearer: None,
            form: Some(body),
        }
    }
}

pub(super) struct Resp {
    pub(super) status: u16,
    retry_after: Option<String>,
    body: Vec<u8>,
}

impl Resp {
    pub(super) fn json<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        serde_json::from_slice(&self.body).ok()
    }

    /// Discord's `Retry-After`, in seconds (fractions allowed), from the header
    /// or else the body's `retry_after`.
    pub(super) fn retry_after(&self) -> Option<Duration> {
        #[derive(Deserialize)]
        struct Body {
            retry_after: f64,
        }
        let secs = self
            .retry_after
            .as_deref()
            .and_then(|v| v.trim().parse::<f64>().ok())
            .or_else(|| self.json::<Body>().map(|b| b.retry_after))?;
        (secs.is_finite() && secs >= 0.0).then(|| Duration::from_secs_f64(secs.min(3600.0)))
    }
}

trait Io: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Io for T {}

pub(super) struct Client {
    tls: tokio_rustls::TlsConnector,
}

impl Client {
    pub(super) fn new() -> Client {
        let roots =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let mut config = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .expect("ring supports the default protocol versions")
        .with_root_certificates(roots)
        .with_no_client_auth();
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        Client {
            tls: tokio_rustls::TlsConnector::from(Arc::new(config)),
        }
    }

    pub(super) async fn send(&self, req: &Req) -> std::io::Result<Resp> {
        let bad =
            |why: &str| std::io::Error::new(std::io::ErrorKind::InvalidInput, why.to_string());
        let (tls, rest) = if let Some(r) = req.url.strip_prefix("https://") {
            (true, r)
        } else if let Some(r) = req.url.strip_prefix("http://") {
            (false, r)
        } else {
            return Err(bad("not an http(s) URL"));
        };
        let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
        let path = if path.is_empty() { "/" } else { path };
        let default_port = if tls { 443 } else { 80 };
        let (host, port) = match authority.strip_prefix('[') {
            // `[::1]:8080`
            Some(v6) => {
                let (h, rest) = v6.split_once(']').ok_or_else(|| bad("bad host"))?;
                match rest.strip_prefix(':') {
                    Some(p) => (h, p.parse::<u16>().map_err(|_| bad("bad port"))?),
                    None => (h, default_port),
                }
            }
            None => match authority.split_once(':') {
                Some((h, p)) => (h, p.parse::<u16>().map_err(|_| bad("bad port"))?),
                None => (authority, default_port),
            },
        };
        let tcp = tokio::net::TcpStream::connect((host, port)).await?;
        tcp.set_nodelay(true)?;
        let io: Box<dyn Io> = if tls {
            let name = rustls::pki_types::ServerName::try_from(host.to_string())
                .map_err(|_| bad("bad host"))?;
            Box::new(self.tls.connect(name, tcp).await?)
        } else {
            Box::new(tcp)
        };
        let mut conn = BufReader::new(io);

        let mut head = format!(
            "{} {path} HTTP/1.1\r\nHost: {authority}\r\nUser-Agent: popquiz-room (https://popquiz.rustnyc.org, 1)\r\nAccept: application/json\r\nConnection: close\r\n",
            req.method
        );
        if let Some(b) = &req.bearer {
            head.push_str("Authorization: Bearer ");
            head.push_str(b.expose());
            head.push_str("\r\n");
        }
        let body: &[u8] = req.form.as_deref().unwrap_or(&[]);
        if req.form.is_some() {
            head.push_str("Content-Type: application/x-www-form-urlencoded\r\n");
        }
        if req.form.is_some() || req.method != "GET" {
            head.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }
        head.push_str("\r\n");
        let w = conn.get_mut();
        w.write_all(head.as_bytes()).await?;
        w.write_all(body).await?;
        w.flush().await?;

        let proto =
            |why: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, why.to_string());
        let mut line = String::new();
        if conn.read_line(&mut line).await? == 0 {
            return Err(proto("closed before a response"));
        }
        let status: u16 = line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| proto("bad status line"))?;
        let (mut chunked, mut length, mut retry_after) = (false, None, None);
        for _ in 0..200 {
            line.clear();
            conn.read_line(&mut line).await?;
            let l = line.trim_end();
            if l.is_empty() {
                break;
            }
            if let Some((k, v)) = l.split_once(':') {
                let v = v.trim();
                match k.trim().to_ascii_lowercase().as_str() {
                    "transfer-encoding" => chunked = v.to_ascii_lowercase().contains("chunked"),
                    "content-length" => {
                        length = Some(
                            v.parse::<usize>()
                                .map_err(|_| proto("bad content-length"))?,
                        )
                    }
                    "retry-after" => retry_after = Some(v.to_string()),
                    _ => {}
                }
            }
        }
        let mut body = Vec::new();
        if chunked {
            read_chunked(&mut conn, &mut body).await?;
        } else if let Some(n) = length {
            if n > MAX_BODY {
                return Err(proto("body too large"));
            }
            body.resize(n, 0);
            conn.read_exact(&mut body).await?;
        } else {
            (&mut conn)
                .take(MAX_BODY as u64 + 1)
                .read_to_end(&mut body)
                .await?;
            if body.len() > MAX_BODY {
                return Err(proto("body too large"));
            }
        }
        Ok(Resp {
            status,
            retry_after,
            body,
        })
    }
}

async fn read_chunked<R: AsyncBufReadExt + Unpin>(
    r: &mut R,
    out: &mut Vec<u8>,
) -> std::io::Result<()> {
    let proto = |why: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, why.to_string());
    let mut line = String::new();
    loop {
        line.clear();
        r.read_line(&mut line).await?;
        let size = usize::from_str_radix(line.trim().split(';').next().unwrap_or(""), 16)
            .map_err(|_| proto("bad chunk size"))?;
        if size == 0 {
            loop {
                line.clear();
                if r.read_line(&mut line).await? == 0 || line.trim().is_empty() {
                    return Ok(());
                }
            }
        }
        if out.len() + size > MAX_BODY {
            return Err(proto("body too large"));
        }
        let start = out.len();
        out.resize(start + size, 0);
        r.read_exact(&mut out[start..]).await?;
        line.clear();
        r.read_line(&mut line).await?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_leaves_unreserved_alone() {
        assert_eq!(
            encode("identify guilds.members.read"),
            "identify%20guilds.members.read"
        );
        assert_eq!(
            encode("http://localhost:3000/auth/discord/callback"),
            "http%3A%2F%2Flocalhost%3A3000%2Fauth%2Fdiscord%2Fcallback"
        );
        assert_eq!(encode("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn retry_after_reads_the_header_then_the_body() {
        let r = Resp {
            status: 429,
            retry_after: Some("0.25".into()),
            body: b"{}".to_vec(),
        };
        assert_eq!(r.retry_after(), Some(Duration::from_millis(250)));
        let r = Resp {
            status: 429,
            retry_after: None,
            body: br#"{"retry_after": 1.5}"#.to_vec(),
        };
        assert_eq!(r.retry_after(), Some(Duration::from_millis(1500)));
        let r = Resp {
            status: 429,
            retry_after: Some("soon".into()),
            body: Vec::new(),
        };
        assert_eq!(r.retry_after(), None);
    }

    #[tokio::test]
    async fn chunked_bodies() {
        let raw = b"4\r\nWiki\r\n6;x=y\r\npedia \r\n0\r\nTrailer: t\r\n\r\n";
        let mut r = BufReader::new(&raw[..]);
        let mut out = Vec::new();
        read_chunked(&mut r, &mut out).await.unwrap();
        assert_eq!(out, b"Wikipedia ");
    }
}
