use std::convert::From;
use std::io;

use tokio::io::AsyncRead;
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;

use crate::StatusCode;

pub struct Response {
    pub status: StatusCode,
    pub body: Box<dyn AsyncRead + Unpin + Send + 'static>,
}

impl From<hyper::Response<hyper::Body>> for Response {
    fn from(v: hyper::Response<hyper::Body>) -> Self {
        let (parts, body) = v.into_parts();

        let body = {
            let v = body.map(|v| v.map_err(|err| io::Error::new(io::ErrorKind::Other, err)));
            Box::new(StreamReader::new(v))
        };

        Self {
            status: parts.status,
            body: body,
        }
    }
}
