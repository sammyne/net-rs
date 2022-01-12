use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::{convert::Infallible, io};

use async_trait::async_trait;
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::server::Builder;
use hyper::service::{make_service_fn, service_fn};

use crate::{Header, Request, StatusCode};

#[async_trait]
pub trait Handler: Send + Sync {
    async fn serve_http(&self, reply: &mut dyn ResponseWriter, request: Request);
}

//pub struct HandleFunc<F, W, O>
//where
//    W: ResponseWriter,
//    F: Fn(&mut W) -> O,
//    O: Future<Output = ()>,
//{
//    inner: F,
//    _maker: std::marker::PhantomData<(W, O)>,
//}

//#[async_trait]
//impl<F, W, O> Handler for HandleFunc<F, W, O>
//where
//    W: ResponseWriter,
//    F: Fn(&mut W) -> O,
//    O: Future<Output = ()>,
//{
//    async fn serve_http(&mut self, reply: &mut W, _request: Request) {}
//}

#[async_trait]
pub trait ResponseWriter: Send {
    async fn write(&mut self, b: &[u8]) -> io::Result<usize>;

    fn header(&mut self) -> &mut Header;
    fn write_header(&mut self, status_code: StatusCode);
}

//impl std::any::Any for ResponseWriter {}

#[derive(Default)]
pub struct Server<H>
where
    H: Handler + 'static,
{
    pub handler: Arc<H>,
    pub addr: String,
}

pub struct ServeMux {}

impl<H> Server<H>
where
    H: Handler + 'static,
{
    pub async fn listen_and_serve(&self) -> io::Result<()> {
        let addr: SocketAddr = self
            .addr
            .parse()
            .map_err(|err| to_other_io_error(err, "parse addr"))?;

        self.serve_hyper(hyper::Server::bind(&addr)).await
    }

    pub async fn serve(&self, l: TcpListener) -> io::Result<()> {
        let b = hyper::Server::from_tcp(l)
            .map_err(|err| to_other_io_error(err, "from standard listener"))?;

        self.serve_hyper(b).await
    }
}

impl<H> Server<H>
where
    H: Handler + 'static,
{
    async fn serve_hyper(&self, b: Builder<AddrIncoming>) -> io::Result<()> {
        let h = self.handler.clone();
        // ref: https://docs.rs/hyper/0.14.16/hyper/server/conn/index.html#example
        // https://docs.rs/hyper/0.14.16/hyper/service/fn.make_service_fn.html
        let handler = make_service_fn(|socket: &AddrStream| {
            let remote_addr = socket.remote_addr();
            let h = h.clone();
            async move {
                let f = move |request: hyper::Request<hyper::Body>| {
                    let h = h.clone();
                    async move {
                        let mut response_writer = MiniResponseWriter::new();

                        let request = Request::from_hyper(request, remote_addr);
                        h.serve_http(&mut response_writer, request).await;

                        Ok::<_, Infallible>(response_writer.to_hyper())
                    }
                };

                Ok::<_, Infallible>(service_fn(f))
            }
        });

        b.serve(handler)
            .await
            .map_err(|err| to_other_io_error(err, "serve"))
    }
}

impl ServeMux {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle<H>(pattern: &str, handler: H)
    where
        H: Handler + 'static,
    {
        todo!();
    }
}

#[async_trait]
impl Handler for ServeMux {
    async fn serve_http(&self, reply: &mut dyn ResponseWriter, request: Request) {
        todo!();
    }
}

pub fn handler_func<H, R>(_pattern: &str, _handler: H)
where
    H: Fn(&mut R, &Request),
    R: ResponseWriter,
{
    todo!();
}

pub async fn listen_and_serve<H>(addr: &str, handler: H) -> io::Result<()>
where
    H: Handler + 'static,
{
    let handler = Arc::new(handler);
    let s = Server {
        handler: handler,
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
