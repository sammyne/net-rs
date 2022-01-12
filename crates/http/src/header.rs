//use std::collections::HashSet;
//use std::io;

use std::str::FromStr;

use hyper::header::{HeaderName, HeaderValue};

#[derive(Clone, Default, Debug)]
pub struct Header(pub(crate) hyper::HeaderMap);

impl Header {
    pub fn add<T, S>(&mut self, key: T, value: S)
    where
        T: AsRef<str>,
        S: AsRef<str>,
    {
        let k = HeaderName::from_str(key.as_ref()).expect("bad key");
        let v = HeaderValue::from_str(value.as_ref()).expect("bad value");
        self.0.append(k, v);
    }

    pub fn del(&mut self, key: &str) {
        self.0.remove(key);
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|v| v.to_str().expect("value as str"))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let (k, v) = (must_canonicalize_key(key), must_canonicalize_value(value));
        self.0.insert(k, v);
    }

    // TODO(sammyne)
    //pub fn values(&self, _key: &str) -> Option<&str> {
    //    todo!();
    //}

    // TODO(sammyne)
    //pub fn write<W>(_w: &mut W) -> io::Result<()>
    //where
    //    W: io::Write,
    //{
    //    todo!();
    //}

    // TODO(sammyne)
    //pub fn write_subset<W, K>(_w: &mut W, _exclude: HashSet<K>) -> io::Result<()>
    //where
    //    W: io::Write,
    //    K: ToString,
    //{
    //    todo!();
    //}

    pub fn new() -> Self {
        Self(hyper::HeaderMap::new())
    }
}

impl Header {
    pub(crate) fn to_hyper(&self) -> hyper::HeaderMap {
        self.0.clone()
    }
}

fn must_canonicalize_key<K>(k: K) -> HeaderName
where
    K: AsRef<str>,
{
    HeaderName::from_str(k.as_ref()).expect("bad key")
}

fn must_canonicalize_value<V>(v: V) -> HeaderValue
where
    V: AsRef<str>,
{
    HeaderValue::from_str(v.as_ref()).expect("bad value")
}
