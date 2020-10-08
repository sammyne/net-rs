use super::errors::Error;

const WHITESPACE: u8 = ' ' as u8;
const PERCENT: u8 = '%' as u8;
const PLUS: u8 = '+' as u8;

const UPPERHEX: &'static str = "0123456789ABCDEF";
const PERCENT_IPV6: &[u8] = "%25".as_bytes();

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Encoding {
    Path,
    PathSegment,
    Host,
    Zone,
    UserPassword,
    QueryComponent,
    Fragment,
}

pub fn escape(s: &str, mode: Encoding) -> String {
    let (space_count, hex_count) = {
        let (mut space_count, mut hex_count) = (0, 0);

        for &c in s.as_bytes() {
            if !should_escape(c, mode) {
                continue;
            }

            match mode {
                Encoding::QueryComponent if c == WHITESPACE => space_count += 1,
                _ => hex_count += 1,
            }
        }

        (space_count, hex_count)
    };

    if space_count == 0 && hex_count == 0 {
        return s.to_string();
    }

    if hex_count == 0 {
        return s.replace(" ", "+");
    }

    let upperhex = UPPERHEX.as_bytes();

    let mut t = String::with_capacity(s.len() + 2 * hex_count);
    for &c in s.as_bytes() {
        if c == WHITESPACE && mode == Encoding::QueryComponent {
            t.push('+');
        } else if should_escape(c, mode) {
            t.push('%');
            t.push(upperhex[(c >> 4) as usize] as char);
            t.push(upperhex[(c & 0x0f) as usize] as char);
        } else {
            t.push(c as char);
        }
    }

    t
}

/// unescape unescapes a string; the mode specifies
/// which section of the URL string is being unescaped.
pub fn unescape(s: &str, mode: Encoding) -> Result<String, Error> {
    // Count %, check that they're well-formed.
    let (n, has_plus) = {
        let mut n = 0;
        let mut has_plus = false;

        let mut i = 0;
        let s = s.as_bytes();
        while i < s.len() {
            let c = s[i];
            match c {
                PERCENT => {
                    n += 1;
                    if i + 2 >= s.len() || !is_hex(s[i + 1]) || !is_hex(s[i + 2]) {
                        let err = if s[i..].len() < 3 {
                            s[i..].to_vec()
                        } else {
                            s[i..=(i + 2)].to_vec()
                        };

                        return Err(Error::Escape(unsafe { String::from_utf8_unchecked(err) }));
                    }

                    // Per https://tools.ietf.org/html/rfc3986#page-21
                    // in the host component %-encoding can only be used
                    // for non-ASCII bytes.
                    // But https://tools.ietf.org/html/rfc6874#section-2
                    // introduces %25 being allowed to escape a percent sign
                    // in IPv6 scoped-address literals. Yay.
                    if mode == Encoding::Host
                        && unhex(s[i + 1]) < 8
                        && (&s[i..=(i + 2)] != PERCENT_IPV6)
                    {
                        let err = unsafe { String::from_utf8_unchecked(s[i..=(i + 2)].to_vec()) };
                        return Err(Error::Escape(err));
                    }

                    if mode == Encoding::Zone {
                        // RFC 6874 says basically "anything goes" for zone identifiers
                        // and that even non-ASCII can be redundantly escaped,
                        // but it seems prudent to restrict %-escaped bytes here to those
                        // that are valid host name bytes in their unescaped form.
                        // That is, you can use escaping in the zone identifier but not
                        // to introduce bytes you couldn't just write directly.
                        // But Windows puts spaces here! Yay.
                        let v = unhex(s[i + 1]) << 4 | unhex(s[i + 2]);
                        if (&s[i..=(i + 2)] != PERCENT_IPV6)
                            && v != WHITESPACE
                            && should_escape(v, Encoding::Host)
                        {
                            let err =
                                unsafe { String::from_utf8_unchecked(s[i..=(i + 2)].to_vec()) };
                            return Err(Error::Escape(err));
                        }
                    }

                    i += 3;
                }
                PLUS => {
                    has_plus = mode == Encoding::QueryComponent;
                    i += 1;
                }
                _ => {
                    if (mode == Encoding::Host || mode == Encoding::Zone)
                        && (c < 0x80)
                        && should_escape(c, mode)
                    {
                        return Err(Error::InvalidHost((c as char).to_string()));
                    }

                    i += 1;
                }
            }
        }

        (n, has_plus)
    };

    if n == 0 && !has_plus {
        return Ok(s.to_string());
    }

    let s = s.as_bytes();

    // String isn't ok in case of non-utf8 char
    let mut t = Vec::with_capacity(s.len() - 2 * n);
    let mut i = 0;
    while i < s.len() {
        match s[i] {
            PERCENT => {
                let c = (unhex(s[i + 1]) << 4) | unhex(s[i + 2]);
                t.push(c);
                i += 2;
            }
            PLUS if mode == Encoding::QueryComponent => t.push(' ' as u8),
            PLUS => t.push('+' as u8),
            _ => t.push(s[i]),
        }

        i += 1;
    }

    let t = unsafe { String::from_utf8_unchecked(t) };

    Ok(t)
}

/// Return true if the specified character should be escaped when
/// appearing in a URL string, according to RFC 3986.
///
/// Please be informed that for now should_escape does not check all
/// reserved characters correctly. See golang.org/issue/5684.
pub(crate) fn should_escape(c: u8, mode: Encoding) -> bool {
    let c = c as char;
    // §2.3 Unreserved characters (alphanum)
    if std::matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9') {
        return false;
    }

    if mode == Encoding::Host || mode == Encoding::Zone {
        // §3.2.2 Host allows
        //	sub-delims = "!" / "$" / "&" / "'" / "(" / ")" / "*" / "+" / "," / ";" / "="
        // as part of reg-name.
        // We add : because we include :port as part of host.
        // We add [ ] because we include [ipv6]:port as part of host.
        // We add < > because they're the only characters left that
        // we could possibly allow, and Parse will reject them if we
        // escape them (because hosts can't use %-encoding for
        // ASCII bytes).
        match c {
            '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '=' | ':' | '[' | ']'
            | '<' | '>' | '"' => return false,
            _ => {}
        }
    }

    match c {
        '-' | '_' | '.' | '~' => return false, // §2.3 Unreserved characters (mark)
        '$' | '&' | '+' | ',' | '/' | ':' | ';' | '=' | '?' | '@' => {
            // §2.2 Reserved characters (reserved)
            match mode {
                Encoding::Path => {
                    // §3.3
                    // The RFC allows : @ & = + $ but saves / ; , for assigning
                    // meaning to individual path segments. This package
                    // only manipulates the path as a whole, so we allow th
                    return c == '?';
                }
                Encoding::PathSegment => {
                    // §3.3
                    // The RFC allows : @ & = + $ but saves / ; , for assigning
                    // meaning to individual path segments.
                    return std::matches!(c, '/' | ';' | ',' | '?');
                }
                Encoding::UserPassword => {
                    // §3.2.1
                    // The RFC allows ';', ':', '&', '=', '+', '$', and ',' in
                    // userinfo, so we must escape only '@', '/', and '?'.
                    // The parsing of userinfo treats ':' as special so we must escape
                    // that too.
                    return std::matches!(c, '@' | '/' | '?' | ':');
                }
                Encoding::QueryComponent => {
                    // §3.4
                    // The RFC reserves (so we must escape) everything.
                    return true;
                }
                Encoding::Fragment => {
                    // §4.1
                    // The RFC text is silent but the grammar allows
                    // everything, so escape nothing.
                    return false;
                }
                _ => {}
            }
        }
        _ => {}
    }

    // RFC 3986 §2.2 allows not escaping sub-delims. A subset of sub-delims are
    // included in reserved from RFC 2396 §2.2. The remaining sub-delims do not
    // need to be escaped. To minimize potential breakage, we apply two restrictions:
    // (1) we always escape sub-delims outside of the fragment, and (2) we always
    // escape single quote to avoid breaking callers that had previously assumed that
    // single quotes would be escaped. See issue #19917.
    if mode == Encoding::Fragment && std::matches!(c, '!' | '(' | ')' | '*') {
        return false;
    }

    // Everything else must be escaped.
    true
}

fn is_hex(c: u8) -> bool {
    let c = c as char;
    std::matches!(c, '0'..='9'|'a'..='f'|'A'..='f')
}

fn unhex(c: u8) -> u8 {
    let c = c as char;
    match c {
        '0'..='9' => (c as u8) - ('0' as u8),
        'a'..='f' => (c as u8) - ('a' as u8) + 10,
        'A'..='F' => (c as u8) - ('A' as u8) + 10,
        _ => 0,
    }
}
