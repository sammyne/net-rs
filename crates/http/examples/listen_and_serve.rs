use std::{future::Future, pin::Pin};

use http::{self, Request, ResponseWriter};

fn hello_handler<'a>(
    w: &'a mut dyn ResponseWriter,
    _r: Request,
) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
    let v = async {
        let _ = w.write(b"Hello, world!\n").await;
    };

    Box::pin(v)
}

#[tokio::main]
async fn main() {
    // this failed due to explicit lifetime requirement
    //let hello_handler =
    //    |w: &mut dyn ResponseWriter, _r: Request| -> Pin<Box<dyn Future<Output = ()> + Send>> {
    //        let v = async {
    //            let _ = w.write(b"Hello, world!\n").await;
    //        };

    //        Box::pin(v)
    //    };

    http::handle_func("/hello", hello_handler).await;

    if let Err(err) = http::listen_and_serve!("127.0.0.1:8080").await {
        panic!("fail to listen and serve: {}", err);
    }
}
