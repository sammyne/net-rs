use std::fmt;

use serde::{Deserialize, Serialize};

use super::internal::{self, Encoding};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub password: Option<String>,
}

impl fmt::Display for UserInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = internal::escape(&self.name, Encoding::UserPassword);
        if let Some(v) = &self.password {
            s += format!(":{}", internal::escape(&v, Encoding::UserPassword)).as_str();
        }

        write!(f, "{}", s)
    }
}

pub fn user<T>(name: T) -> UserInfo
where
    T: ToString,
{
    UserInfo {
        name: name.to_string(),
        password: None,
    }
}

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
