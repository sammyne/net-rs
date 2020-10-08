use std::fmt;

use serde::{Deserialize, Serialize};

use super::errors::{self, Error};
use super::internal::{self, Encoding};
use super::{UserInfo, Values};

/// A URL represents a parsed URL (technically, a URI reference).
///
/// The general form represented is:
///
///	[scheme:][//[userinfo@]host][/]path[?query][#fragment]
///
/// URLs that do not start with a slash after the scheme are interpreted as:
///
///	scheme:opaque[?query][#fragment]
///
/// Note that the [`path`] field is stored in decoded form: `/%47%6f%2f` becomes `/Go/`.
/// A consequence is that it is impossible to tell which slashes in the [`path`] were
/// slashes in the raw URL and which were %2f. This distinction is rarely important,
/// but when it is, the code should use [`raw_path`], an optional field which only gets
/// set if the default encoding is different from [`path`].
///
/// URL's String method uses the [`escaped_path`] method to obtain the path. See the
/// [`escaped_path`] method for more details.
///
/// [`escaped_path`]: #method.escaped_path
/// [`path`]: #structfield.path
/// [`raw_path`]: #structfield.raw_path
///
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct URL {
    /// append a query ('?') even if `raw_query` is empty
    pub force_query: bool,
    /// fragment for references, without '#'
    pub fragment: String,
    /// host or host:port
    pub host: String,
    /// path (relative paths may omit leading slash)
    pub path: String,
    /// encoded opaque data
    pub opaque: String,
    /// encoded fragment hint (see [escaped_fragment](#method.escaped_fragment) method)
    pub raw_fragment: String,
    /// encoded path hint (see [escaped_path](#method.escaped_path) method)
    pub raw_path: String,
    /// encoded query values, without '?'
    pub raw_query: String,
    pub scheme: String,
    /// username and password information
    pub user: Option<UserInfo>,
}

impl URL {
    /// `escaped_fragment` returns the escaped form of `self.fragment`.
    /// In general there are multiple possible escaped forms of any fragment.
    /// `escaped_fragment` returns `self.raw_fragment` when it is a valid escaping of `self.fragment`.
    /// Otherwise `escaped_fragment` ignores `self.raw_fragment` and computes an escaped
    /// form on its own.
    /// The String method uses `escaped_fragment` to construct its result.
    /// In general, code should call `escaped_fragment` instead of
    /// reading `self.raw_fragment` directly.
    pub fn escaped_fragment(&self) -> String {
        if self.raw_fragment != "" && valid_encoded(&self.raw_fragment, Encoding::Fragment) {
            match internal::unescape(&self.raw_fragment, Encoding::Fragment) {
                Ok(v) if v == self.fragment => return self.raw_fragment.clone(),
                _ => {}
            }
        }

        internal::escape(&self.fragment, Encoding::Fragment)
    }

    /// escaped_path returns the escaped form of self.path.
    /// In general there are multiple possible escaped forms of any path.
    /// escaped_path returns self.raw_path when it is a valid escaping of self.path.
    /// Otherwise escaped_path ignores self.raw_path and computes an escaped
    /// form on its own.
    /// The Display::fmt and [request_uri](#method.request_uri) methods use escaped_path to
    /// construct their results.
    /// In general, code should call escaped_path instead of
    /// reading self.raw_path directly.
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

    /// hostname returns self.host, stripping any valid port number if present.
    ///
    /// If the result is enclosed in square brackets, as literal IPv6 addresses are,
    /// the square brackets are removed from the result.
    pub fn hostname(&self) -> &str {
        let (host, _) = split_host_port(&self.host);
        host
    }

    /// is_abs reports whether the URL is absolute.
    /// Absolute means that it has a non-empty scheme.
    pub fn is_abs(&self) -> bool {
        self.scheme != ""
    }

    /// parse parses a URL in the context of the receiver. The provided URL
    /// may be relative or absolute. Parse returns nil, err on parse
    /// failure, otherwise its return value is the same as
    /// [resolve_reference](#method.resolve_reference).
    pub fn parse(&self, reference: &str) -> Result<Self, Error> {
        let refurl = parse(reference)?;
        Ok(self.resolve_reference(&refurl))
    }

    /// port returns the port part of self.host, without the leading colon.
    ///
    /// If self.host doesn't contain a valid numeric port, port returns an empty string.
    pub fn port(&self) -> &str {
        let (_, port) = split_host_port(&self.host);
        port
    }

    /// query parses raw_query and returns the corresponding values.
    /// It silently discards malformed value pairs.
    /// To check errors use ParseQuery.
    pub fn query(&self) -> Values {
        match super::parse_query(&self.raw_query) {
            Ok(v) => v,
            Err((v, _)) => v,
        }
    }

    /// redacted is like to_string() but replaces any password with "xxxxx".
    /// Only the password in self.user is redacted.
    pub fn redacted(&self) -> String {
        let mut redacted = self.clone();
        if let Some(v) = &redacted.user {
            if v.password.is_some() {
                redacted.user = Some(super::user_password(&v.name, "xxxxx"));
            }
        }

        redacted.to_string()
    }

    /// request_uri returns the encoded path?query or opaque?query
    /// string that would be used in an HTTP request for self.
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

    /// resolve_reference resolves a URI reference to an absolute URI from
    /// an absolute base URI u, per RFC 3986 Section 5.2. The URI reference
    /// may be relative or absolute. resolve_reference always returns a new
    /// URL instance, even if the returned URL is identical to either the
    /// base or reference. If ref is an absolute URL, then resolve_reference
    /// ignores base and returns a copy of ref.
    pub fn resolve_reference(&self, r: &Self) -> Self {
        let mut url = r.clone();

        if r.scheme == "" {
            url.scheme = self.scheme.clone();
        }

        if r.scheme != "" || r.host != "" || r.user.is_some() {
            // The "absoluteURI" or "net_path" cases.
            // We can ignore the error from setPath since we know we provided a
            // validly-escaped path.
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

        // The "abs_path" or "rel_path" cases.
        url.host = self.host.clone();
        url.user = self.user.clone();
        let _ = url.set_path(&resolve_path(self.escaped_path(), r.escaped_path()));

        url
    }

    // set_fragment is like set_path but for fragment/raw_fragment.
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

    /// set_path sets the `path` and `raw_path` fields of the URL based on the provided
    /// escaped path p. It maintains the invariant that `raw_path` is only specified
    /// when it differs from the default encoding of the path.
    /// For example:
    /// - set_path("/foo/bar")   will set path="/foo/bar" and raw_path=""
    /// - set_path("/foo%2fbar") will set path="/foo/bar" and raw_path="/foo%2fbar"
    /// set_path will return an error only if the provided path contains an invalid
    /// escaping.
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
    /// `fmt` reassembles the URL into a valid URL string.
    /// The general form of the result is one of:
    ///
    ///	scheme:opaque?query#fragment
    ///	scheme://userinfo@host/path?query#fragment
    ///
    /// If `self.opaque` is non-empty, `fmt` uses the first form;
    /// otherwise it uses the second form.
    /// Any non-ASCII characters in host are escaped.
    /// To obtain the path, String uses `self.escaped_path()`.
    ///
    /// In the second form, the following rules apply:
    ///	- if `self.scheme` is empty, scheme: is omitted.
    ///	- if `self.user` is None, userinfo@ is omitted.
    ///	- if `self.host` is empty, host/ is omitted.
    ///	- if `self.scheme` and `self.host` are empty and `self.user` is None,
    ///	   the entire scheme://userinfo@host/ is omitted.
    ///	- if `self.host` is non-empty and u.Path begins with a /,
    ///	   the form host/path does not add its own /.
    ///	- if `self.raw_query` is empty, ?query is omitted.
    ///	- if `self.fragment` is empty, #fragment is omitted.
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

/// parse parses rawurl into a URL structure.
///
/// The rawurl may be relative (a path, without a host) or absolute
/// (starting with a scheme). Trying to parse a hostname and path
/// without a scheme is invalid but may not necessarily return an
/// error, due to parsing ambiguities.
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

/// parse_request_uri parses rawurl into a URL structure. It assumes that
/// rawurl was received in an HTTP request, so the rawurl is interpreted
/// only as an absolute URI or an absolute path.
/// The string rawurl is assumed not to have a #fragment suffix.
/// (Web browsers strip #fragment before sending the URL to a web server.)
pub fn parse_request_uri(rawurl: &str) -> Result<URL, Error> {
    do_parse(rawurl, true).map_err(|err| errors::wrap("parse", rawurl, err))
}

/// do_parse parses a URL from a string in one of two contexts. If
/// viaRequest is true, the URL is assumed to have arrived via an HTTP request,
/// in which case only absolute URLs or path-absolute relative URLs are allowed.
/// If viaRequest is false, all forms of relative URLs are allowed.
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

    // Split off possible leading "http:", "mailto:", etc.
    // Cannot contain escaped characters.
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
            // We consider rootless paths per RFC 3986 as opaque.
            out.opaque = rest.to_string();
            return Ok(out);
        }

        if via_request {
            return Err(errors::new_misc("invalid URI for request"));
        }

        // Avoid confusion with malformed schemes, like cache_object:foo/bar.
        // See golang.org/issue/16822.
        //
        // RFC 3986, §3.3:
        // In addition, a URI reference (Section 4.1) may be a relative-path reference,
        // in which case the first path segment cannot contain a colon (":") character.
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

    // Set `path` and, optionally, `raw_path`.
    // `raw_path` is a hint of the encoding of `path`. We don't want to set it if
    // the default escaping of Path is equivalent, to help make sure that people
    // don't rely on it in general.
    out.set_path(rest)?;

    Ok(out)
}

/// Maybe rawurl is of the form scheme:path.
/// (Scheme must be [a-zA-Z][a-zA-Z0-9+-.]*)
/// If so, return scheme, path; else return "", rawurl.
fn getscheme(rawurl: &str) -> Result<(String, &str), Error> {
    for (i, c) in rawurl.chars().enumerate() {
        match c {
            'a'..='z' | 'A'..='Z' => {} // do nothing
            '0'..='9' | '+' | '-' | '.' if i == 0 => break,
            ':' if i == 0 => return Err(errors::new_misc("missing protocol scheme")),
            ':' => {
                let scheme = rawurl.get(..i).unwrap_or_default().to_string();
                let path = rawurl.get((i + 1)..).unwrap_or_default();
                return Ok((scheme, path));
            }
            _ => {
                // we have encountered an invalid character,
                // so there is no valid scheme
                break;
            }
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

/// parse_host parses host as an authority without user
/// information. That is, as host[:port].
fn parse_host(host: &str) -> Result<String, Error> {
    if host.starts_with('[') {
        // Parse an IP-Literal in RFC 3986 and RFC 6874.
        // E.g., "[fe80::1]", "[fe80::1%25en0]", "[fe80::1]:80".
        let i = match host.rfind(']') {
            None => return Err(errors::new_misc("missing ']' in host")),
            Some(v) => v,
        };

        let colon_port = host.get((i + 1)..).unwrap_or_default();
        if !valid_optional_port(colon_port) {
            let err = format!("invalid port {} after host", colon_port);
            return Err(errors::new_misc(err));
        }

        // RFC 6874 defines that %25 (%-encoded percent) introduces
        // the zone identifier, and the zone identifier can use basically
        // any %-encoding it likes. That's different from the host, which
        // can only %-encode non-ASCII bytes.
        // We do impose some restrictions on the zone, to avoid stupidity
        // like newlines.
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

/// resolve_path applies special path segments from refs and applies
/// them to base, per RFC 3986.
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
            "." => {} // drop
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
        Some(&v) if v == "." || v == ".." => dst.push(""), // Add final slash to the joined path.
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

/// split slices s into two substrings separated by the first occurrence of
/// sep. If cutc is true then sep is excluded from the second substring.
/// If sep does not occur in s then s and the empty string is returned.
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

/// string_contains_ctl_byte reports whether s contains any ASCII control character.
fn string_contains_ctl_byte(s: &str) -> bool {
    const WHITESPACE: u8 = ' ' as u8;
    s.as_bytes().iter().any(|&c| c < WHITESPACE || c == 0x7f)
}

/// split_host_port separates host and port. If the port is not valid, it returns
/// the entire input as host, and it doesn't check the validity of the host.
/// Unlike net.SplitHostPort (@TODO: update), but per RFC 3986, it requires ports to be numeric.
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

/// valid_encoded reports whether s is a valid encoded path or fragment,
/// according to mode.
/// It must not contain any bytes that require escaping during encoding.
fn valid_encoded(s: &str, mode: Encoding) -> bool {
    for &c in s.as_bytes() {
        let cc = c as char;
        // RFC 3986, Appendix A.
        // pchar = unreserved / pct-encoded / sub-delims / ":" / "@".
        // should_escape is not quite compliant with the RFC,
        // so we check the sub-delims ourselves and let
        // should_escape handle the others.
        match cc {
            '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | '|' | ';' | '=' | ':' | '@' => {} // ok
            '[' | ']' => {} //ok - not specified in RFC 3986 but left alone by modern browsers
            '%' => {}       // ok - percent encoded, will decode
            _ if internal::should_escape(c, mode) => return false,
            _ => {}
        }
    }

    true
}

/// valid_optional_port reports whether port is either an empty string
/// or matches /^:\d*$/
fn valid_optional_port(port: &str) -> bool {
    (port == "") || (port.starts_with(':') && port.chars().skip(1).all(|c| c >= '0' && c <= '9'))
}

#[cfg(test)]
mod tests;
