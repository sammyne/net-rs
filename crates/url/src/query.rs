use super::errors::Error;

use super::internal::{self, Encoding};

/// query_escape escapes the string so it can be safely placed
/// inside a URL query.
pub fn query_escape(s: &str) -> String {
    internal::escape(s, Encoding::QueryComponent)
}

/// query_unescape does the inverse transformation of query_escape,
/// converting each 3-byte encoded substring of the form "%AB" into the
/// hex-decoded byte 0xAB.
/// It returns an error if any % is not followed by two hexadecimal
/// digits.
pub fn query_unescape(s: &str) -> Result<String, Error> {
    internal::unescape(s, Encoding::QueryComponent)
}
