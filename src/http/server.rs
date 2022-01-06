use std::{convert::Infallible, io, net::SocketAddr};

use async_trait::async_trait;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};

use crate::http::{Header, Request, StatusCode};

#[async_trait]
pub trait Handler: Clone + Send + Sync {
    //async fn serve_http<W>(&mut self, reply: &mut W, request: &Request)
    async fn serve_http<W>(&mut self, reply: &mut W, _request: &Request)
    where
        W: ResponseWriter;
}

#[async_trait]
pub trait ResponseWriter: Send {
    async fn write(&mut self, b: &[u8]) -> io::Result<usize>;

    fn header(&mut self) -> &mut Header;
    fn write_header(&mut self, status_code: StatusCode);
}

#[derive(Default)]
pub struct Server<H>
where
    H: Handler + 'static,
{
    pub handler: Option<H>,
    pub addr: String,
}

impl<H> Server<H>
where
    H: Handler + 'static,
{
    pub async fn listen_and_serve(&self) -> io::Result<()> {
        let addr: SocketAddr = self
            .addr
            .parse()
            .map_err(|err| to_other_io_error(err, "parse addr"))?;

        let h = self.handler.clone().unwrap();
        // ref: https://docs.rs/hyper/0.14.16/hyper/server/conn/index.html#example
        // https://docs.rs/hyper/0.14.16/hyper/service/fn.make_service_fn.html
        let handler = make_service_fn(|socket: &AddrStream| {
            let remote_addr = socket.remote_addr();
            let h = h.clone();
            async move {
                let f = move |request: Request| {
                    let mut h = h.clone();
                    let remote_addr = remote_addr.clone();
                    async move {
                        println!("remote addr = {}", remote_addr);
                        let mut response_writer = MiniResponseWriter::new();

                        h.serve_http(&mut response_writer, &request).await;

                        let reply = response_writer.to_hyper();

                        Ok::<_, Infallible>(reply)
                    }
                };

                Ok::<_, Infallible>(service_fn(f))
            }
        });
        hyper::Server::bind(&addr)
            .serve(handler)
            .await
            .map_err(|err| to_other_io_error(err, "serve"))
    }
}

/*
pub fn handler_func<H, R>(_pattern: &str, _handler: H)
where
    H: Fn(&mut R, &Request),
    R: ResponseWriter,
{
    todo!();
}
*/

pub async fn listen_and_serve<H>(addr: &str, _handler: Option<H>) -> io::Result<()>
where
    H: Handler + 'static,
{
    let s = Server {
        handler: _handler,
        addr: addr.to_string(),
    };

    s.listen_and_serve().await
}

// internal APIs

struct MiniResponseWriter {
    inner: hyper::Response<Vec<u8>>,
    header: Header,
}

#[async_trait]
impl ResponseWriter for MiniResponseWriter {
    async fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        println!("hello");
        self.inner.body_mut().extend_from_slice(b);
        Ok(b.len())
    }

    fn header(&mut self) -> &mut Header {
        &mut self.header
    }

    fn write_header(&mut self, status_code: StatusCode) {
        *self.inner.status_mut() = status_code;
    }
}

impl MiniResponseWriter {
    pub fn new() -> Self {
        Self {
            inner: hyper::Response::new(vec![]),
            header: Header::new(),
        }
    }

    pub fn to_hyper(self) -> hyper::Response<hyper::Body> {
        let (mut parts, body) = self.inner.into_parts();

        parts.headers = self.header.to_hyper();

        hyper::Response::from_parts(parts, hyper::Body::from(body))
    }
}

fn to_other_io_error<E>(err: E, desc: &str) -> io::Error
where
    E: ToString,
{
    io::Error::new(
        io::ErrorKind::Other,
        format!("{}: {}", &desc, err.to_string()),
    )
}
