use super::errors::Error;

use super::internal::{self, Encoding};

/// path_escape escapes the string so it can be safely placed inside a URL path segment,
/// replacing special characters (including /) with %XX sequences as needed.
pub fn path_escape(s: &str) -> String {
    internal::escape(s, Encoding::PathSegment)
}

/// path_unescape does the inverse transformation of path_escape,
/// converting each 3-byte encoded substring of the form "%AB" into the
/// hex-decoded byte 0xAB. It returns an error if any % is not followed
/// by two hexadecimal digits.
///
/// path_unescape is identical to query_unescape except that it does not
/// unescape '+' to ' ' (space).
pub fn path_unescape(s: &str) -> Result<String, Error> {
    internal::unescape(s, Encoding::PathSegment)
}
