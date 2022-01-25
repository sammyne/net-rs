use http::{handler_func, Handler, HandlerFunc, Request, ResponseWriter};

fn new_people_handler() -> impl Handler {
    #[handler_func]
    async fn f(w: &mut dyn ResponseWriter, _r: Request) {
        let _ = w.write(b"This is the people handler.\n").await;
    }

    f as HandlerFunc
}

#[tokio::main]
async fn main() {
    let mux = http::new_serve_mux();

    // Create sample handler to returns 404
    mux.handle("/resources", http::not_found_handler()).await;

    // Create sample handler that returns 200
    mux.handle("/resources/people/", new_people_handler()).await;

    if let Err(err) = http::listen_and_serve(":8080", mux).await {
        panic!("fail to listen and serve: {}", err);
    }
}
