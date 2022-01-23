use std::sync::Mutex;

use async_trait::async_trait;
use http::{self, Handler, Request, ResponseWriter};

struct CountHandler {
    c: Mutex<i32>,
}

#[async_trait]
impl Handler for CountHandler {
    async fn serve_http(&self, w: &mut dyn ResponseWriter, _r: Request) {
        let n = {
            let mut n = self.c.lock().unwrap();
            *n += 1;
            *n
        };

        let reply = format!("count is {}\n", n);
        let _ = w.write(reply.as_bytes()).await;
    }
}

#[tokio::main]
async fn main() {
    let handler = CountHandler { c: Mutex::new(0) };

    http::handle("/count", handler).await;

    if let Err(err) = http::listen_and_serve!(":8080").await {
        panic!("fail to listen and serve: {}", err);
    }
}
