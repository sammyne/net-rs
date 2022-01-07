//! module url parses URLs and implements query escaping.
//!
//! See RFC 3986. This package generally follows RFC 3986, except where
//! it deviates for compatibility reasons. When sending changes, first
//! search old issues for history on decisions. Unit tests should also
//! contain references to issue numbers with details.

mod path;
mod query;
mod url;
mod user_info;
mod values;

mod internal;

pub mod errors;

pub use crate::url::*;
pub use path::*;
pub use query::*;
pub use user_info::*;
pub use values::*;

#[cfg(test)]
mod tests;

//#[cfg(test)]
//mod examples;
