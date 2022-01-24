mod client;
mod header;
mod method;
mod proto;
mod request;
mod response;
mod server;
mod status;

pub mod errors;

pub use http_proc_macro::handler_func;

pub use client::*;
pub use header::*;
pub use method::*;
pub use proto::*;
pub use request::*;
pub use response::*;
pub use server::*;
pub use status::*;

#[handler_func]
async fn hello_handler(w: &mut dyn ResponseWriter, _r: Request) {
    let _ = w.write(b"Hello, world!\n").await;
}
