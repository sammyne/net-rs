use std::io;
use std::net::SocketAddr;

use hyper::{header::HeaderValue, Uri};
use tokio::io::AsyncRead;
use tokio_stream::StreamExt;
use tokio_util::io::{ReaderStream, StreamReader};
use url::URL;

use crate::{Header, Method, Proto};

const HEADER_KEY_CONTENT_LENGTH: &'static str = "Content-Length";
const HEADER_KEY_HOST: &'static str = "Host";

pub type NopBody = tokio::io::Empty;
pub use tokio::io::empty as new_nop_body;

/// TODO: make body generic
/// TODO: get_body
pub struct Request {
    pub method: Method,
    pub url: URL,
    pub proto: Proto,
    pub header: Header,
    pub body: Box<dyn AsyncRead + Unpin + Send + 'static>,
    pub content_length: Option<u64>,
    pub host: String,
    pub remote_addr: SocketAddr,
}

impl Request {
    pub fn new(
        method: Method,
        url: String,
        body: Option<Box<dyn AsyncRead + Unpin + Send + 'static>>,
    ) -> Result<Self, String> {
        let url = url::parse(&url).map_err(|err| err.to_string())?;

        let body = body.unwrap_or_else(|| Box::new(new_nop_body()));

        let out = Self {
            method,
            url,
            body,
            ..Default::default()
        };

        Ok(out)
    }
}

impl Request {
    pub(crate) fn from_hyper(req: hyper::Request<hyper::Body>, remote_addr: SocketAddr) -> Self {
        let method = req.method().clone();

        let (parts, body) = req.into_parts();

        let url = url::parse_request_uri(&parts.uri.to_string()).expect("parse url");
        let proto = Proto::from(parts.version);
        let mut header = Header(parts.headers);

        let host = header
            .get(HEADER_KEY_HOST)
            .map(|v| v.to_string())
            .unwrap_or_default();
        header.del(HEADER_KEY_HOST);

        let content_length: Option<u64> = match header.get(HEADER_KEY_CONTENT_LENGTH) {
            Some(v) => v.parse().ok(),
            None => None,
        };

        let body = {
            let v = body.map(|v| v.map_err(|err| io::Error::new(io::ErrorKind::Other, err)));
            Box::new(StreamReader::new(v))
        };

        Self {
            method,
            url,
            proto,
            header,
            body,
            content_length,
            host,
            remote_addr,
        }
    }

    pub(crate) fn to_hyper(self) -> Result<hyper::Request<hyper::Body>, String> {
        let body = hyper::Body::wrap_stream(ReaderStream::new(self.body));

        let mut out = hyper::Request::new(body);

        *out.method_mut() = self.method;
        *out.uri_mut() =
            Uri::try_from(self.url.to_string()).map_err(|err| format!("bad url: {}", err))?;
        *out.version_mut() = self
            .proto
            .try_into()
            .map_err(|err| format!("bad version: {}", err))?;

        {
            println!("version = {:?}", out.version());
        }

        *out.headers_mut() = self.header.0;
        if self.host.len() > 0 {
            let v = HeaderValue::from_str(&self.host)
                .map_err(|err| format!("bad host value: {}", err))?;
            out.headers_mut().insert(HEADER_KEY_HOST, v);
        }
        if let Some(v) = self.content_length {
            let v = HeaderValue::from_str(&v.to_string()).expect("bad content length");
            out.headers_mut().insert(HEADER_KEY_CONTENT_LENGTH, v);
        }

        Ok(out)
    }
}

impl Default for Request {
    fn default() -> Self {
        let body = Box::new(tokio::io::empty());
        let remote_addr = "127.0.0.1:80".parse().expect("bad default remote addr");
        Self {
            method: Method::GET,
            url: URL::default(),
            proto: Proto::default(),
            header: Header::default(),
            body,
            content_length: None,
            host: String::default(),
            remote_addr,
        }
    }
}

pub fn parse_http_version(vers: &str) -> Result<(i32, i32), ()> {
    const BIG: i32 = 100000;

    let _ = match vers {
        "HTTP/1.1" => return Ok((1, 1)),
        "HTTP/1.0" => return Ok((1, 0)),
        _ => {}
    };

    let vers = vers.strip_prefix("HTTP/").ok_or(())?;

    let mut itr = vers.splitn(2, '.');
    let major = itr.next().ok_or(())?.parse::<i32>().map_err(|_| ())?;
    if major < 0 || major > BIG {
        return Err(());
    }
    let minor = itr.next().ok_or(())?.parse::<i32>().map_err(|_| ())?;
    if minor < 0 || minor > BIG {
        return Err(());
    }

    Ok((major, minor))
}
