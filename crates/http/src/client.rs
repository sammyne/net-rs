use hyper::client;
use tokio::io::AsyncRead;

use crate::{Method, Request, Response};

lazy_static::lazy_static! {
  pub static ref DEFAULT_CLIENT: Client = Client::new();
}

#[derive(Default)]
pub struct Client {}

impl Client {
    pub async fn do_request(&self, r: Request) -> Result<Response, String> {
        let r = r
            .to_hyper()
            .map_err(|err| format!("build request: {}", err))?;
        let c = client::Client::new();
        c.request(r)
            .await
            .map(Response::from)
            .map_err(|err| format!("do request: {}", err))
    }

    pub async fn get(&self, url: &str) -> Result<Response, String> {
        let req = Request::new(Method::GET, url.to_string(), None)
            .map_err(|err| format!("build request: {}", err))?;
        self.do_request(req).await
    }

    pub fn head(&self, _url: &str) -> Result<Response, String> {
        todo!();
    }

    pub fn new() -> Self {
        Self {}
    }

    pub fn post<B>(&self, _url: &str, _content_type: &str, _body: B) -> Result<Response, String>
    where
        B: AsyncRead + Unpin + Send + 'static,
    {
        todo!();
    }

    pub fn post_form(&self, _url: &str, _data: url::Values) -> Result<Response, String> {
        todo!();
    }
}

pub async fn get(url: &str) -> Result<Response, String> {
    DEFAULT_CLIENT.get(url).await
}
