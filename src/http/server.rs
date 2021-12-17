use std::future::Future;
use std::pin::Pin;
use std::{convert::Infallible, io, net::SocketAddr};

use async_trait::async_trait;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response};

use crate::http::Request;

#[async_trait]
//pub trait Handler: Clone + Send + Sync {
pub trait Handler: Clone + Send + Sync {
    //async fn serve_http<R>(reply: &mut R, request: &Request)
    //where
    //    R: ResponseWriter;
    async fn serve_http(&mut self, reply: &mut ResponseWriter, request: &Request);
}

//pub trait ResponseWriter {}
pub type ResponseWriter = hyper::Response<Body>;

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
                        let mut reply = ResponseWriter::new(Body::empty());
                        h.serve_http(&mut reply, &request).await;
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
async fn handle(_req: Request) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("hello world!")))
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
