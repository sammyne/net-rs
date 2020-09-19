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

                    if mode == Encoding::Host
                        && unhex(s[i + 1]) < 8
                        && (&s[i..=(i + 2)] != PERCENT_IPV6)
                    {
                        let err = unsafe { String::from_utf8_unchecked(s[i..=(i + 2)].to_vec()) };
                        return Err(Error::Escape(err));
                    }

                    if mode == Encoding::Zone {
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

pub(crate) fn should_escape(c: u8, mode: Encoding) -> bool {
    let c = c as char;
    if std::matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9') {
        return false;
    }

    if mode == Encoding::Host || mode == Encoding::Zone {
        match c {
            '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '=' | ':' | '[' | ']'
            | '<' | '>' | '"' => return false,
            _ => {}
        }
    }

    match c {
        '-' | '_' | '.' | '~' => return false,
        '$' | '&' | '+' | ',' | '/' | ':' | ';' | '=' | '?' | '@' => match mode {
            Encoding::Path => return c == '?',
            Encoding::PathSegment => return std::matches!(c, '/' | ';' | ',' | '?'),
            Encoding::UserPassword => return std::matches!(c, '@' | '/' | '?' | ':'),
            Encoding::QueryComponent => return true,
            Encoding::Fragment => return false,
            _ => {}
        },
        _ => {}
    }

    if mode == Encoding::Fragment && std::matches!(c, '!' | '(' | ')' | '*') {
        return false;
    }

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
