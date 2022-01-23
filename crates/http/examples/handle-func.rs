use http::{self, handler_func, Request, ResponseWriter};

#[tokio::main]
async fn main() {
    #[handler_func]
    async fn hello_handler(w: &mut dyn ResponseWriter, _r: Request) {
        let _ = w.write(b"Hello, world!\n").await;
    }

    http::handle_func("/hello", hello_handler).await;

    if let Err(err) = http::listen_and_serve!("127.0.0.1:8080").await {
        panic!("fail to listen and serve: {}", err);
    }
}
