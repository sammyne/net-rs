use async_trait::async_trait;

use net::http::{self, Handler, ResponseWriter};

#[derive(Clone)]
struct HelloWorld;

#[async_trait]
impl Handler for HelloWorld {
    //async fn serve_http<W>(&mut self, reply: Arc<Mutex<W>>, _request: &http::Request)
    async fn serve_http<W>(&mut self, reply: &mut W, _request: &http::Request)
    where
        W: ResponseWriter,
    {
        //*reply.body_mut() = Body::from("hello world");
        //let _ = reply.lock().await.write(b"hello world").await;
        let _ = reply.write(b"hello world\n").await;
        let _ = reply.write(b"hello world2\n").await;
    }
}

#[tokio::main]
async fn main() {
    let h = Some(HelloWorld);
    if let Err(err) = http::listen_and_serve("127.0.0.1:8080", h).await {
        panic!("fail to listen and serve: {}", err);
    }
}
