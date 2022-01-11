use std::io;

use tokio::io::AsyncRead;
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;

use crate::{Header, Method};

pub struct Request {
    pub method: Method,
    pub header: Header,
    pub body: Box<dyn AsyncRead + Send + 'static>,
}

impl Request {
    pub(crate) fn from_hyper(req: hyper::Request<hyper::Body>) -> Self {
        let method = req.method().clone();

        let (parts, body) = req.into_parts();

        let header = Header(parts.headers);

        let body = {
            let v = body.map(|v| v.map_err(|err| io::Error::new(io::ErrorKind::Other, err)));
            Box::new(StreamReader::new(v))
        };

        Self {
            method,
            header,
            body,
        }
    }
}
