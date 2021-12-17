use async_trait::async_trait;

use hyper::Body;
use net::http;
use net::http::Handler;

#[derive(Clone)]
struct HelloWorld;

#[async_trait]
impl Handler for HelloWorld {
    async fn serve_http(&mut self, reply: &mut http::ResponseWriter, _request: &http::Request) {
        *reply.body_mut() = Body::from("hello world");
    }
}

#[tokio::main]
async fn main() {
    let h = Some(HelloWorld);
    if let Err(err) = http::listen_and_serve("127.0.0.1:8080", h).await {
        panic!("fail to listen and serve: {}", err);
    }
}
