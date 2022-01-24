use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() {
    println!("world");
    let mut reply = http::get("http://www.google.com/robots.txt")
        .await
        .expect("get");

    let mut body = String::new();
    reply
        .body
        .read_to_string(&mut body)
        .await
        .expect("read body");

    let status = reply.status.as_u16();
    if status > 299 {
        panic!(
            "Response failed with status code: {} and\nbody: {}\n",
            status, body
        );
    }

    println!("{}", body);
}
