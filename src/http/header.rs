//use std::collections::HashSet;
//use std::io;

#[derive(Clone)]
pub struct Header(hyper::HeaderMap);

impl Header {
    pub fn add<T, S>(&mut self, _key: T, _value: S)
    where
        T: ToString,
        S: ToString,
    {
        todo!();
    }

    pub fn del(&mut self, _key: &str) {
        todo!();
    }

    pub fn get(&self, _key: &str) -> &str {
        todo!();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn set<T, S>(&mut self, _key: T, _value: S)
    where
        T: ToString,
        S: ToString,
    {
        todo!();
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
