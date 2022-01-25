use async_trait::async_trait;
use http::{handler_func, Handler, Request, ResponseWriter};

struct ApiHandler {}

#[async_trait]
impl Handler for ApiHandler {
    async fn serve_http(&self, _w: &mut dyn ResponseWriter, _r: Request) {}
}

#[tokio::main]
async fn main() {
    let mux = http::new_serve_mux();

    mux.handle("/api/", ApiHandler {}).await;

    #[handler_func]
    async fn root(w: &mut dyn ResponseWriter, r: Request) {
        // The "/" pattern matches everything, so we need to check
        // that we're at the root here.
        if r.url.path != "/" {
            http::not_found(w, r).await;
            return;
        }

        let _ = w.write(b"Welcome to the home page!").await;
    }
    mux.handle_func("/", root).await;

    if let Err(err) = http::listen_and_serve(":8080", mux).await {
        panic!("fail to listen and serve: {}", err);
    }
}
