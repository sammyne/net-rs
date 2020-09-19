use std::fmt;

use serde::{Deserialize, Serialize};

use super::errors::{self, Error};
use super::internal::{self, Encoding};
use super::{UserInfo, Values};

///
/// @todo marshal_binary and unmarshal_binary
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct URL {
    pub force_query: bool,
    pub fragment: String,
    pub host: String,
    pub path: String,
    pub opaque: String,
    pub raw_fragment: String,
    pub raw_path: String,
    pub raw_query: String,
    pub scheme: String,
    pub user: Option<UserInfo>,
}

impl URL {
    pub fn escaped_fragment(&self) -> String {
        if self.raw_fragment != "" && valid_encoded(&self.raw_fragment, Encoding::Fragment) {
            match internal::unescape(&self.raw_fragment, Encoding::Fragment) {
                Ok(v) if v == self.fragment => return self.raw_fragment.clone(),
                _ => {}
            }
        }

        internal::escape(&self.fragment, Encoding::Fragment)
    }

    pub fn escaped_path(&self) -> String {
        if self.raw_path != "" && valid_encoded(&self.raw_path, Encoding::Path) {
            match internal::unescape(&self.raw_path, Encoding::Path) {
                Ok(v) if v == self.path => return self.raw_path.clone(),
                _ => {}
            }
        }

        if self.path == "*" {
            return "*".to_string();
        }

        internal::escape(&self.path, Encoding::Path)
    }

    pub fn hostname(&self) -> &str {
        let (host, _) = split_host_port(&self.host);
        host
    }

    pub fn is_abs(&self) -> bool {
        self.scheme != ""
    }

    pub fn parse(&self, reference: &str) -> Result<Self, Error> {
        let refurl = parse(reference)?;
        Ok(self.resolve_reference(&refurl))
    }

    pub fn port(&self) -> &str {
        let (_, port) = split_host_port(&self.host);
        port
    }

    pub fn query(&self) -> Values {
        match super::parse_query(&self.raw_query) {
            Ok(v) => v,
            Err((v, _)) => v,
        }
    }

    pub fn redacted(&self) -> String {
        let mut redacted = self.clone();
        if let Some(v) = &redacted.user {
            if v.password.is_some() {
                redacted.user = Some(super::user_password(&v.name, "xxxxx"));
            }
        }

        redacted.to_string()
    }

    pub fn request_uri(&self) -> String {
        let mut out = self.opaque.clone();
        if out == "" {
            out = self.escaped_path();
            if out == "" {
                out = "/".to_string();
            }
        } else if out.starts_with("//") {
            out = format!("{}:{}", self.scheme, out);
        }

        if self.force_query || self.raw_query != "" {
            out.push('?');
            out += &self.raw_query;
        }

        out
    }

    pub fn resolve_reference(&self, r: &Self) -> Self {
        let mut url = r.clone();

        if r.scheme == "" {
            url.scheme = self.scheme.clone();
        }

        if r.scheme != "" || r.host != "" || r.user.is_some() {
            let _ = url.set_path(&resolve_path(r.escaped_path(), ""));
            return url;
        }

        if r.opaque != "" {
            url.user = None;
            url.host.clear();
            url.path.clear();
            return url;
        }

        if r.path == "" && r.raw_query == "" {
            url.raw_query = self.raw_query.clone();
            if r.fragment == "" {
                url.fragment = self.fragment.clone();
                url.raw_fragment = self.raw_fragment.clone();
            }
        }

        url.host = self.host.clone();
        url.user = self.user.clone();
        let _ = url.set_path(&resolve_path(self.escaped_path(), r.escaped_path()));

        url
    }

    fn set_fragment(&mut self, fragment: &str) -> Result<(), Error> {
        self.fragment = internal::unescape(fragment, Encoding::Fragment)?;

        let escaped = internal::escape(&self.fragment, Encoding::Fragment);
        self.raw_fragment = if escaped == fragment {
            "".to_string()
        } else {
            fragment.to_string()
        };

        Ok(())
    }

    fn set_path(&mut self, path: &str) -> Result<(), Error> {
        self.path = internal::unescape(path, Encoding::Path)?;

        let escaped = internal::escape(&self.path, Encoding::Path);
        self.raw_path = if escaped == path {
            "".to_string()
        } else {
            path.to_string()
        };

        Ok(())
    }
}

impl fmt::Display for URL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut written = false;
        if self.scheme != "" {
            write!(f, "{}:", self.scheme)?;
            written = true;
        }

        if self.opaque != "" {
            write!(f, "{}", self.opaque)?;
        } else {
            if self.scheme != "" || self.host != "" || self.user.is_some() {
                if self.host != "" || self.path != "" || self.user.is_some() {
                    write!(f, "//")?;
                    written = true;
                }

                if let Some(u) = &self.user {
                    write!(f, "{}@", u)?;
                    written = true;
                }

                if self.host != "" {
                    write!(f, "{}", internal::escape(&self.host, Encoding::Host))?;
                    written = true;
                }
            }

            let path = self.escaped_path();
            if path != "" && !path.starts_with('/') && self.host != "" {
                write!(f, "/")?;
                written = true;
            }

            if !written
                && std::matches!(path.find(':'), Some(i) if !path.get(..i).unwrap().contains('/'))
            {
                write!(f, "./")?;
            }
            write!(f, "{}", path)?;
        }

        if self.force_query || self.raw_query != "" {
            write!(f, "?{}", self.raw_query)?;
        }

        if self.fragment != "" {
            write!(f, "#{}", self.escaped_fragment())?;
        }

        Ok(())
    }
}

pub fn parse(rawurl: &str) -> Result<URL, Error> {
    let (u, frag) = split(rawurl, '#', true);
    let mut url = do_parse(u, false).map_err(|err| errors::wrap("parse", u, err))?;

    if frag == "" {
        return Ok(url);
    }

    url.set_fragment(frag)
        .map_err(|err| errors::wrap("parse", rawurl, err))?;

    Ok(url)
}

pub fn parse_request_uri(rawurl: &str) -> Result<URL, Error> {
    do_parse(rawurl, true).map_err(|err| errors::wrap("parse", rawurl, err))
}

fn do_parse(rawurl: &str, via_request: bool) -> Result<URL, Error> {
    if string_contains_ctl_byte(rawurl) {
        let err = "net/url: invalid control character in URL";
        return Err(errors::new_misc(err));
    }

    if rawurl == "" && via_request {
        return Err(errors::new_misc("empty url"));
    }

    let mut out = URL::default();

    if rawurl == "*" {
        out.path = "*".to_string();
        return Ok(out);
    }

    let (scheme, rest) = getscheme(rawurl)?;
    out.scheme = scheme.to_ascii_lowercase();

    let mut rest = if rest.ends_with('?') && (rest.chars().filter(|&v| v == '?').count() == 1) {
        out.force_query = true;
        rest.trim_end_matches('?')
    } else {
        let (r, raw_query) = split(rest, '?', true);
        out.raw_query = raw_query.to_string();
        r
    };

    if !rest.starts_with('/') {
        if out.scheme != "" {
            out.opaque = rest.to_string();
            return Ok(out);
        }

        if via_request {
            return Err(errors::new_misc("invalid URI for request"));
        }

        if let Some(colon) = rest.find(":") {
            let slash = rest.find('/');
            if slash.is_none() || colon < slash.unwrap() {
                let err = "first path segment in URL cannot contain colon";
                return Err(errors::new_misc(err));
            }
        }
    }

    if (out.scheme != "" || (!via_request && !rest.starts_with("///"))) && rest.starts_with("//") {
        let (authority, r) = split(rest.get(2..).unwrap(), '/', false);
        let (user, host) = parse_authority(authority)?;
        out.user = user;
        out.host = host;
        rest = r;
    }

    out.set_path(rest)?;

    Ok(out)
}

fn getscheme(rawurl: &str) -> Result<(String, &str), Error> {
    for (i, c) in rawurl.chars().enumerate() {
        match c {
            'a'..='z' | 'A'..='Z' => {}
            '0'..='9' | '+' | '-' | '.' if i == 0 => break,
            ':' if i == 0 => return Err(errors::new_misc("missing protocol scheme")),
            ':' => {
                let scheme = rawurl.get(..i).unwrap_or_default().to_string();
                let path = rawurl.get((i + 1)..).unwrap_or_default();
                return Ok((scheme, path));
            }
            _ => break,
        }
    }

    Ok(("".to_string(), rawurl))
}

fn parse_authority(authority: &str) -> Result<(Option<UserInfo>, String), Error> {
    let i = authority.rfind('@');
    let host = match i {
        None => parse_host(authority)?,
        Some(v) => parse_host(authority.get((v + 1)..).unwrap_or_default())?,
    };

    if i.is_none() {
        return Ok((None, host));
    }

    let userinfo = authority.get(..i.unwrap()).unwrap();
    if !super::valid_userinfo(userinfo) {
        return Err(errors::new_misc("net/url: invalid userinfo"));
    }

    let user = if !userinfo.contains(':') {
        let v = internal::unescape(userinfo, Encoding::UserPassword)?;
        super::user(v)
    } else {
        let (username, password) = split(userinfo, ':', true);
        let username = internal::unescape(username, Encoding::UserPassword)?;
        let password = internal::unescape(password, Encoding::UserPassword)?;
        super::user_password(username, password)
    };

    Ok((Some(user), host))
}

fn parse_host(host: &str) -> Result<String, Error> {
    if host.starts_with('[') {
        let i = match host.rfind(']') {
            None => return Err(errors::new_misc("missing ']' in host")),
            Some(v) => v,
        };

        let colon_port = host.get((i + 1)..).unwrap_or_default();
        if !valid_optional_port(colon_port) {
            let err = format!("invalid port {} after host", colon_port);
            return Err(errors::new_misc(err));
        }

        let zone = host.get(..i).unwrap().find("%25");
        if let Some(zone) = zone {
            let host1 = internal::unescape(host.get(..zone).unwrap(), Encoding::Host)?;
            let host2 = internal::unescape(host.get(zone..i).unwrap(), Encoding::Zone)?;
            let host3 = internal::unescape(host.get(i..).unwrap_or_default(), Encoding::Host)?;

            return Ok(host1 + &host2 + &host3);
        }
    } else if let Some(i) = host.rfind(':') {
        let colon_port = host.get(i..).unwrap();
        if !valid_optional_port(colon_port) {
            let err = format!("invalid port {} after host", colon_port);
            return Err(errors::new_misc(err));
        }
    }

    internal::unescape(host, Encoding::Host)
}

fn resolve_path<S, T>(base: S, reference: T) -> String
where
    S: AsRef<str>,
    T: AsRef<str>,
{
    let base = base.as_ref();
    let reference = reference.as_ref();

    let full = if reference == "" {
        base.to_string()
    } else if reference.chars().nth(0) != Some('/') {
        match base.rfind('/') {
            Some(i) => base.get(..(i + 1)).unwrap().to_string() + reference,
            None => reference.to_string(),
        }
    } else {
        reference.to_string()
    };

    if full == "" {
        return full;
    }

    let src = full.split('/').collect::<Vec<_>>();
    let mut dst = Vec::with_capacity(src.len());
    for &v in &src {
        match v {
            "." => {}
            ".." => {
                if dst.len() > 0 {
                    dst.pop();
                }
            }
            _ => {
                dst.push(v);
            }
        }
    }

    match src.last() {
        Some(&v) if v == "." || v == ".." => dst.push(""),
        Some(_) => {}
        _ => panic!("missing the last component"),
    }

    let d = dst.join("/");
    let v = if d.starts_with('/') {
        d.get(1..).unwrap()
    } else {
        d.as_str()
    };

    "/".to_string() + v
}

fn split(s: &str, sep: char, cutc: bool) -> (&str, &str) {
    let i = match s.find(sep) {
        Some(v) => v,
        None => return (s, ""),
    };

    let first = s.get(..i).unwrap_or_default();
    let second = if cutc {
        s.get((i + 1)..).unwrap_or_default()
    } else {
        s.get(i..).unwrap_or_default()
    };

    (first, second)
}

fn string_contains_ctl_byte(s: &str) -> bool {
    const WHITESPACE: u8 = ' ' as u8;
    s.as_bytes().iter().any(|&c| c < WHITESPACE || c == 0x7f)
}

fn split_host_port(hostport: &str) -> (&str, &str) {
    let (host, port) = match hostport.rfind(':') {
        Some(i) if valid_optional_port(&hostport[i..]) => (
            hostport.get(..i).unwrap(),
            hostport.get((i + 1)..).unwrap_or_default(),
        ),
        _ => (hostport, ""),
    };

    if host.starts_with('[') && host.ends_with(']') {
        (host.get(1..(host.len() - 1)).unwrap(), port)
    } else {
        (host, port)
    }
}

fn valid_encoded(s: &str, mode: Encoding) -> bool {
    for &c in s.as_bytes() {
        let cc = c as char;
        match cc {
            '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | '|' | ';' | '=' | ':' | '@' => {}
            '[' | ']' => {}
            '%' => {}
            _ if internal::should_escape(c, mode) => return false,
            _ => {}
        }
    }

    true
}

fn valid_optional_port(port: &str) -> bool {
    (port == "") || (port.starts_with(':') && port.chars().skip(1).all(|c| c >= '0' && c <= '9'))
}

#[cfg(test)]
mod tests;
