use super::errors::Error;

use super::internal::{self, Encoding};

// @depend super::internal::escape
pub fn path_escape(s: &str) -> String {
    internal::escape(s, Encoding::PathSegment)
}

pub fn path_unescape(s: &str) -> Result<String, Error> {
    internal::unescape(s, Encoding::PathSegment)
}
