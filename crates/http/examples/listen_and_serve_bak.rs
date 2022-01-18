use std::sync::Mutex;

use async_trait::async_trait;
use http::{self, Handler, Request, ResponseWriter};
use tokio::io::AsyncReadExt;

struct HelloWorld {
    c: Mutex<i32>,
}

#[async_trait]
impl Handler for HelloWorld {
    async fn serve_http(&self, reply: &mut dyn ResponseWriter, request: Request) {
        {
            *self.c.lock().unwrap() += 1;
        }
        {
            let v = *self.c.lock().unwrap();
            println!("c = {}", v);
        }
        println!("url = {}", request.url);
        {
            let mut body = request.body;
            let mut msg = String::new();
            let _ = body.read_to_string(&mut msg).await;
            println!("body = '{}'", msg);
        }

        println!("header = {:?}", request.header);

        reply.header().add("hello", "world");
        let _ = reply.write(b"hello world\n").await;
        let _ = reply.write(b"hello world2\n").await;
    }
}

#[tokio::main]
async fn main() {
    let handler = HelloWorld { c: Mutex::new(1) };
    if let Err(err) = http::listen_and_serve("127.0.0.1:8080", handler).await {
        panic!("fail to listen and serve: {}", err);
    }
}
