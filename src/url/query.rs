use super::errors::Error;

use super::internal::{self, Encoding};

pub fn query_escape(s: &str) -> String {
    internal::escape(s, Encoding::QueryComponent)
}

pub fn query_unescape(s: &str) -> Result<String, Error> {
    internal::unescape(s, Encoding::QueryComponent)
}
