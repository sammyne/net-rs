use std::convert::From;
use std::io;

use tokio::io::AsyncRead;
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;

pub struct Response {
    pub status: String,
    pub status_code: u16,
    pub body: Box<dyn AsyncRead + Unpin + Send + 'static>,
}

impl From<hyper::Response<hyper::Body>> for Response {
    fn from(v: hyper::Response<hyper::Body>) -> Self {
        let (parts, body) = v.into_parts();

        let body = {
            let v = body.map(|v| v.map_err(|err| io::Error::new(io::ErrorKind::Other, err)));
            Box::new(StreamReader::new(v))
        };

        let status = if let Some(v) = parts.status.canonical_reason() {
            format!("{} {}", parts.status.as_str(), v)
        } else {
            parts.status.to_string()
        };

        Self {
            status,
            status_code: parts.status.as_u16(),
            body: body,
        }
    }
}
