use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid URL escape `{0}`")]
    Escape(String),
    #[error("invalid character `{0}` in host name")]
    InvalidHost(String),
    #[error("{0}")]
    Misc(String),
    #[error("{op} {url}: {err}")]
    Wrapped {
        op: String,
        url: String,
        err: Box<dyn std::error::Error>,
    },
}

pub fn new_misc<T>(desc: T) -> Error
where
    T: ToString,
{
    Error::Misc(desc.to_string())
}

pub fn wrap(op: &str, url: &str, err: Error) -> Error {
    Error::Wrapped {
        op: op.to_string(),
        url: url.to_string(),
        err: Box::new(err),
    }
}
