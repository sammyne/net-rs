use std::fmt;

use serde::{Deserialize, Serialize};

use crate::internal::{self, Encoding};

/// The Userinfo type is an immutable encapsulation of username and
/// password details for a URL. An existing Userinfo value is guaranteed
/// to have a username set (potentially empty, as allowed by RFC 2396),
/// and optionally a password.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub password: Option<String>,
}

impl fmt::Display for UserInfo {
    // fmt returns the encoded userinfo information in the standard form
    // of "username[:password]".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = internal::escape(&self.name, Encoding::UserPassword);
        if let Some(v) = &self.password {
            s += format!(":{}", internal::escape(&v, Encoding::UserPassword)).as_str();
        }

        write!(f, "{}", s)
    }
}

/// user returns a Userinfo containing the provided name
/// and no password set.
pub fn user<T>(name: T) -> UserInfo
where
    T: ToString,
{
    UserInfo {
        name: name.to_string(),
        password: None,
    }
}

/// user_password returns a Userinfo containing the provided name
/// and password.
///
/// This functionality should only be used with legacy web sites.
/// RFC 2396 warns that interpreting Userinfo this way
/// "is NOT RECOMMENDED, because the passing of authentication
/// information in clear text (such as URI) has proven to be a
/// security risk in almost every case where it has been used."
pub fn user_password<S, T>(name: S, password: T) -> UserInfo
where
    S: ToString,
    T: ToString,
{
    UserInfo {
        name: name.to_string(),
        password: Some(password.to_string()),
    }
}

/// valid_userinfo reports whether s is a valid userinfo string per RFC 3986
/// Section 3.2.1:
///     userinfo    = *( unreserved / pct-encoded / sub-delims / ":" )
///     unreserved  = ALPHA / DIGIT / "-" / "." / "_" / "~"
///     sub-delims  = "!" / "$" / "&" / "'" / "(" / ")"
///                   / "*" / "+" / "," / ";" / "="
///
/// It doesn't validate pct-encoded. The caller does that via func unescape.
pub(crate) fn valid_userinfo(s: &str) -> bool {
    let iter = s
        .chars()
        .filter(|&c| c < 'A' || c > 'Z')
        .filter(|&c| c < 'a' || c > 'z')
        .filter(|&c| c < '0' || c > '9');

    for c in iter {
        match c {
            '-' | '.' | '_' | ':' | '~' | '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | '|'
            | ';' | '=' | '%' | '@' => {}
            _ => return false,
        }
    }

    return true;
}
