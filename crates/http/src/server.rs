use std::collections::BTreeMap;
use std::future::Future;
use std::net::{SocketAddr, TcpListener};
use std::pin::Pin;
use std::sync::Arc;
use std::{convert::Infallible, io};

use async_trait::async_trait;
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::server::Builder;
use hyper::service::{make_service_fn, service_fn};
use lazy_static::lazy_static;
use tokio::sync::RwLock;

use crate::{Header, Request, StatusCode};

lazy_static! {
    pub static ref DEFAULT_SERVE_MUX: ServeMux = ServeMux::new();
}

#[async_trait]
pub trait Handler: Send + Sync {
    async fn serve_http(&self, reply: &mut dyn ResponseWriter, request: Request);
}

pub type HandlerFunc = for<'a> fn(
    &'a mut dyn ResponseWriter,
    Request,
) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

#[async_trait]
pub trait ResponseWriter: Send {
    async fn write(&mut self, b: &[u8]) -> io::Result<usize>;

    fn header(&mut self) -> &mut Header;
    fn write_header(&mut self, status_code: StatusCode);
}

//#[derive(Default)]
pub struct Server {
    pub handler: Arc<dyn Handler + 'static>,
    pub addr: String,
}

pub struct ServeMux {
    // m.keys is sorted thanks to BTreeMap
    m: RwLock<BTreeMap<String, Box<dyn Handler + 'static>>>,
}

#[async_trait]
impl Handler for HandlerFunc {
    async fn serve_http(&self, reply: &mut dyn ResponseWriter, request: Request) {
        self(reply, request).await
    }
}

impl Server {
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

impl Server {
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
        Self {
            m: RwLock::default(),
        }
    }

    // @TODO: duplicate patterns
    pub async fn handle<H>(&self, pattern: &str, handler: H)
    where
        H: Handler + 'static,
    {
        if pattern == "" {
            panic!("http: invalid pattern")
        }

        let mut m = self.m.write().await;

        if m.contains_key(pattern) {
            panic!("http: multiple registrations for {}", pattern);
        }

        m.insert(pattern.to_string(), Box::new(handler));
    }

    pub async fn handle_func(&self, pattern: &str, handler: HandlerFunc) {
        self.handle(pattern, handler).await
    }
}

#[async_trait]
impl Handler for &ServeMux {
    async fn serve_http(&self, w: &mut dyn ResponseWriter, r: Request) {
        let path = r.url.path.as_str();

        let m = self.m.read().await;

        let h = match m.get(path) {
            Some(v) => Some(v),
            None => m
                .iter()
                .rev()
                .find(|&(k, _)| k.len() < path.len() && path.starts_with(k))
                .map(|(_, v)| v),
        };

        if let Some(h) = h {
            h.serve_http(w, r).await
        } else {
            not_found(w, r).await
        }
    }
}

pub async fn error<E>(w: &mut dyn ResponseWriter, error: E, code: StatusCode)
where
    E: AsRef<[u8]>,
{
    w.header().set("Content-Type", "text/plain; charset=utf-8");
    w.header().set("X-Content-Type-Options", "nosniff");
    w.write_header(code);
    let _ = w.write(error.as_ref()).await;
    let _ = w.write(b"\n").await;
}

pub async fn handle<H>(pattern: &str, handler: H)
where
    H: Handler + 'static,
{
    DEFAULT_SERVE_MUX.handle(pattern, handler).await
}

pub async fn handle_func(pattern: &str, handler: HandlerFunc) {
    DEFAULT_SERVE_MUX.handle_func(pattern, handler).await
}

#[macro_export]
macro_rules! listen_and_serve {
    ($addr:literal) => {
        $crate::listen_and_serve($addr, &*$crate::DEFAULT_SERVE_MUX)
    };
    ($addr:literal, $handler:ident) => {
        $crate::listen_and_serve($addr, $handler)
    };
}

/// @warn: use the listen_and_serve macro instead.
pub async fn listen_and_serve<H>(addr: &str, handler: H) -> io::Result<()>
where
    H: Handler + 'static,
{
    let addr = if addr.starts_with(":") {
        format!("127.0.0.1{}", addr)
    } else {
        addr.to_string()
    };

    let s = Server {
        handler: Arc::new(handler),
        addr: addr,
    };

    s.listen_and_serve().await
}

pub async fn not_found(w: &mut dyn ResponseWriter, _r: Request) {
    error(w, "404 page not found", StatusCode::NOT_FOUND).await;
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
