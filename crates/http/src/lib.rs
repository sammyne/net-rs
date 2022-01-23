mod header;
mod method;
mod proto;
mod request;
mod server;
mod status;

pub mod errors;

pub use http_proc_macro::handler_func;

pub use header::*;
pub use method::*;
pub use proto::*;
pub use request::*;
pub use server::*;
pub use status::*;

#[handler_func]
async fn hello_handler(w: &mut dyn ResponseWriter, _r: Request) {
    let _ = w.write(b"Hello, world!\n").await;
}
