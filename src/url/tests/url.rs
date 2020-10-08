use super::super::errors::Error;
use super::super::{UserInfo, URL};

const PATH_THAT_LOOKS_SCHEME_RELATIVE: &str = "//not.a.user@not.a.host/just/a/path";

#[test]
fn hostname_and_port() {
    struct Case {
        raw_host: &'static str,
        host: &'static str,
        port: &'static str,
    }

    let new_case = |raw_host, host, port| -> Case {
        Case {
            raw_host,
            host,
            port,
        }
    };

    let test_vector = vec![
        new_case("foo.com:80", "foo.com", "80"),
        new_case("foo.com", "foo.com", ""),
        new_case("foo.com:", "foo.com", ""),
        new_case("FOO.COM", "FOO.COM", ""), // no canonicalization
        new_case("1.2.3.4", "1.2.3.4", ""),
        new_case("1.2.3.4:80", "1.2.3.4", "80"),
        new_case("[1:2:3:4]", "1:2:3:4", ""),
        new_case("[1:2:3:4]:80", "1:2:3:4", "80"),
        new_case("[::1]:80", "::1", "80"),
        new_case("[::1]", "::1", ""),
        new_case("[::1]:", "::1", ""),
        new_case("localhost", "localhost", ""),
        new_case("localhost:443", "localhost", "443"),
        new_case(
            "some.super.long.domain.example.org:8080",
            "some.super.long.domain.example.org",
            "8080",
        ),
        new_case(
            "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:17000",
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
            "17000",
        ),
        new_case(
            "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]",
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
            "",
        ),
        // Ensure that even when not valid, Host is one of "Hostname",
        // "Hostname:Port", "[Hostname]" or "[Hostname]:Port".
        // See https://golang.org/issue/29098.
        new_case("[google.com]:80", "google.com", "80"),
        new_case("google.com]:80", "google.com]", "80"),
        new_case(
            "google.com:80_invalid_port",
            "google.com:80_invalid_port",
            "",
        ),
        new_case("[::1]extra]:80", "::1]extra", "80"),
        new_case("google.com]extra:extra", "google.com]extra:extra", ""),
    ];

    for c in test_vector {
        let url = URL {
            host: c.raw_host.to_string(),
            ..Default::default()
        };

        assert_eq!(c.host, url.hostname(), "invalid hostname");
        assert_eq!(c.port, url.port(), "invalid port");
    }
}

#[test]
fn nil_user() {
    let url = super::super::parse("http://foo.com/").unwrap();
    assert!(url.user.is_none());
}

#[test]
fn json() {
    let url = super::super::parse("https://www.google.com/x?y=z").unwrap();

    let json = serde_json::to_vec(&url).unwrap();

    let recovered: URL = serde_json::from_slice(json.as_slice()).unwrap();

    assert_eq!(
        url.to_string(),
        recovered.to_string(),
        "json decoding failed"
    );
}

#[test]
fn parse() {
    let test_vector = url_test_vector();

    for c in test_vector {
        let got = super::super::parse(&c.rawurl)
            .map_err(|err| format!("parse({}) returned error: {:?}", c.rawurl, err))
            .unwrap();

        assert_eq!(
            c.out, got,
            "parse({}):\n\tgot  {}\n\twant {}\n",
            c.rawurl, got, c.out
        );
    }
}

#[test]
fn parse_errors() {
    struct Case {
        rawurl: String,
        want_err: bool,
    }

    let new_case = |rawurl: &str, want_err: bool| -> Case {
        Case {
            rawurl: rawurl.to_string(),
            want_err,
        }
    };

    let test_vector = vec![
        new_case("http://[::1]", false),
        new_case("http://[::1]:80", false),
        new_case("http://[::1]:namedport", true), // rfc3986 3.2.3
        new_case("http://x:namedport", true),     // rfc3986 3.2.3
        new_case("http://[::1]/", false),
        new_case("http://[::1]a", true),
        new_case("http://[::1]%23", true),
        new_case("http://[::1%25en0]", false), // valid zone id
        new_case("http://[::1]:", false),      // colon, but no port OK
        new_case("http://x:", false),          // colon, but no port OK
        new_case("http://[::1]:%38%30", true), // not allowed: % encoding only for non-ASCII
        new_case("http://[::1%25%41]", false), // RFC 6874 allows over-escaping in zone
        new_case("http://[%10::1]", true),     // no %xx escapes in IP address
        new_case("http://[::1]/%48", false),   // %xx in path is fine
        new_case("http://%41:8080/", true),    // not allowed: % encoding only for non-ASCII
        new_case("mysql://x@y(z:123)/foo", true), // not well-formed per RFC 3986, golang.org/issue/33646
        new_case("mysql://x@y(1.2.3.4:123)/foo", true),
        new_case(" http://foo.com", true), // invalid character in schema
        new_case("ht tp://foo.com", true), // invalid character in schema
        new_case("ahttp://foo.com", false), // valid schema characters
        new_case("1http://foo.com", true), // invalid character in schema
        new_case(
            "http://[]%20%48%54%54%50%2f%31%2e%31%0a%4d%79%48%65%61%64%65%72%3a%20%31%32%33%0a%0a/",
            true,
        ), // golang.org/issue/11208
        new_case("http://a b.com/", true), // no space in host name please
        new_case("cache_object://foo", true), // scheme cannot have _, relative path cannot have : in first segment
        new_case("cache_object:foo", true),
        new_case("cache_object:foo/bar", true),
        new_case("cache_object/:foo/bar", false),
    ];

    for (i, c) in test_vector.iter().enumerate() {
        let got = super::super::parse(&c.rawurl);
        if c.want_err {
            assert!(got.is_err(), "#{}: Parse({}) wants an error", i, c.rawurl);
            continue;
        }

        assert!(got.is_ok(), "#{}: Parse({}) wants no error", i, c.rawurl);
    }
}

#[test]
fn parse_failure() {
    const URL: &str = "%gh&%ij";
    let err = super::super::parse_query(URL)
        .expect_err("should error out")
        .1
        .to_string();

    assert!(
        err.contains("%gh"),
        "parse_query({}) returned error '{}', want something containing '%gh'",
        URL,
        err
    );
}

#[test]
fn parse_invalid_user_password() {
    const RAWURL: &str = "http://user^:passwo^rd@foo.com/";
    match super::super::parse(RAWURL) {
        Ok(_) => panic!(),
        Err(err) if err.to_string().contains("net/url: invalid userinfo") => {}
        Err(err) => panic!("{:?}", err),
    }
}

#[test]
fn parse_reject_control_characters() {
    let test_vector = vec![
        "http://foo.com/?foo\nbar",
        "http\r://foo.com/",
        "http://foo\x7f.com/",
    ];

    let expect_err = "net/url: invalid control character in URL";
    for (i, c) in test_vector.iter().enumerate() {
        match super::super::parse(c) {
            Ok(_) => panic!("#{} expect errors", i),
            Err(Error::Wrapped { err, .. }) if err.to_string().contains(expect_err) => {}
            Err(err) => panic!("#{} unexpected error: {:?}", i, err),
        }
    }

    // @TODO: this case needs further checking on if \x80 is effective
    if let Err(err) = super::super::parse(r#"http://foo.com/ctl\x80"#) {
        panic!("error parsing URL with non-ASCII control byte: {:?}", err);
    }
}

#[test]
fn parse_request_uri() {
    struct Case {
        url: String,
        expected_valid: bool,
    }

    let new_case = |url: &str, expected_valid: bool| -> Case {
        Case {
            url: url.to_string(),
            expected_valid,
        }
    };

    let test_vector = vec![
        new_case("http://foo.com", true),
        new_case("http://foo.com/", true),
        new_case("http://foo.com/path", true),
        new_case("/", true),
        new_case(PATH_THAT_LOOKS_SCHEME_RELATIVE, true),
        new_case("//not.a.user@%66%6f%6f.com/just/a/path/also", true),
        new_case("*", true),
        new_case("http://192.168.0.1/", true),
        new_case("http://192.168.0.1:8080/", true),
        new_case("http://[fe80::1]/", true),
        new_case("http://[fe80::1]:8080/", true),
        // Tests exercising RFC 6874 compliance:
        new_case("http://[fe80::1%25en0]/", true), // with alphanum zone identifier
        new_case("http://[fe80::1%25en0]:8080/", true), // with alphanum zone identifier
        new_case("http://[fe80::1%25%65%6e%301-._~]/", true), // with percent-encoded+unreserved zone identifier
        new_case("http://[fe80::1%25%65%6e%301-._~]:8080/", true), // with percent-encoded+unreserved zone identifier
        new_case("foo.html", false),
        new_case("../dir/", false),
        new_case(" http://foo.com", false),
        new_case("http://192.168.0.%31/", false),
        new_case("http://192.168.0.%31:8080/", false),
        new_case("http://[fe80::%31]/", false),
        new_case("http://[fe80::%31]:8080/", false),
        new_case("http://[fe80::%31%25en0]/", false),
        new_case("http://[fe80::%31%25en0]:8080/", false),
        // These two cases are valid as textual representations as
        // described in RFC 4007, but are not valid as address
        // literals with IPv6 zone identifiers in URIs as described in
        // RFC 6874.
        new_case("http://[fe80::1%en0]/", false),
        new_case("http://[fe80::1%en0]:8080/", false),
    ];

    for c in test_vector {
        match super::super::parse_request_uri(&c.url) {
            Err(err) if c.expected_valid => panic!(
                "parse_request_uri({}) gave err {}; want no error",
                c.url, err
            ),
            Ok(_) if !c.expected_valid => panic!(
                "parse_request_uri({}) gave nil error; want some error",
                c.url
            ),
            _ => {}
        }
    }

    let url = super::super::parse_request_uri(PATH_THAT_LOOKS_SCHEME_RELATIVE).unwrap();
    assert_eq!(
        url.path, PATH_THAT_LOOKS_SCHEME_RELATIVE,
        "parse_request_uri path:\ngot  {}\nwant {}",
        url.path, PATH_THAT_LOOKS_SCHEME_RELATIVE
    );
}

#[test]
fn query_values() {
    let mut v = super::super::parse("http://x.com?foo=bar&bar=1&bar=2")
        .unwrap()
        .query();

    assert_eq!(
        2,
        v.0.len(),
        "got mismatched #(keys) in Query values, want 2"
    );

    let got = v.get("foo").unwrap_or_default();
    assert_eq!("bar", got, "get('foo') failed");

    // Case sensitive:
    assert!(v.get("Foo").is_none(), "get('Foo') should return no values");

    assert_eq!(Some("1"), v.get("bar"), "get('bar') failed");

    assert!(v.get("baz").is_none(), "get('baz') should return no values");

    v.del("bar");
    assert!(
        v.get("bar").is_none(),
        "get('bar') should return no values after delete"
    );
}

#[test]
fn request_uri() {
    struct Case {
        url: URL,
        out: String,
    }

    let new_case = |url, out: &str| -> Case {
        Case {
            url,
            out: out.to_string(),
        }
    };

    let test_vector = vec![
        new_case(
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                path: "".to_string(),
                ..Default::default()
            },
            "/",
        ),
        new_case(
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                path: "/a b".to_string(),
                ..Default::default()
            },
            "/a%20b",
        ),
        // golang.org/issue/4860 variant 1
        new_case(
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                opaque: "/%2F/%2F/".to_string(),
                ..Default::default()
            },
            "/%2F/%2F/",
        ),
        // golang.org/issue/4860 variant 2
        new_case(
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                opaque: "//other.example.com/%2F/%2F/".to_string(),
                ..Default::default()
            },
            "http://other.example.com/%2F/%2F/",
        ),
        // better fix for issue 4860
        new_case(
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                path: "/////".to_string(),
                raw_path: "/%2F/%2F/".to_string(),
                ..Default::default()
            },
            "/%2F/%2F/",
        ),
        new_case(
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                path: "/////".to_string(),
                raw_path: "/WRONG/".to_string(), // ignored because doesn't match Path
                ..Default::default()
            },
            "/////",
        ),
        new_case(
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                path: "/a b".to_string(),
                raw_query: "q=go+language".to_string(),
                ..Default::default()
            },
            "/a%20b?q=go+language",
        ),
        new_case(
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                path: "/a b".to_string(),
                raw_path: "/a b".to_string(), // ignored because invalid
                raw_query: "q=go+language".to_string(),
                ..Default::default()
            },
            "/a%20b?q=go+language",
        ),
        new_case(
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                path: "/a?b".to_string(),
                raw_path: "/a?b".to_string(), // ignored because invalid
                raw_query: "q=go+language".to_string(),
                ..Default::default()
            },
            "/a%3Fb?q=go+language",
        ),
        new_case(
            URL {
                scheme: "myschema".to_string(),
                opaque: "opaque".to_string(),
                ..Default::default()
            },
            "opaque",
        ),
        new_case(
            URL {
                scheme: "myschema".to_string(),
                opaque: "opaque".to_string(),
                raw_query: "q=go+language".to_string(),
                ..Default::default()
            },
            "opaque?q=go+language",
        ),
        new_case(
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                path: "//foo".to_string(),
                ..Default::default()
            },
            "//foo",
        ),
        new_case(
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                path: "/foo".to_string(),
                force_query: true,
                ..Default::default()
            },
            "/foo?",
        ),
    ];

    for c in test_vector {
        let got = c.url.request_uri();
        assert_eq!(c.out, got, "{}.request_uri() failed", c.url);
    }
}

#[test]
fn resolve_reference() {
    struct Case {
        base: &'static str,
        rel: &'static str,
        expected: &'static str,
    }

    let new_case = |base, rel, expected| -> Case {
        Case {
            base,
            rel,
            expected,
        }
    };

    let test_vector = vec![
        // Absolute URL references
        new_case("http://foo.com?a=b", "https://bar.com/", "https://bar.com/"),
        new_case(
            "http://foo.com/",
            "https://bar.com/?a=b",
            "https://bar.com/?a=b",
        ),
        new_case("http://foo.com/", "https://bar.com/?", "https://bar.com/?"),
        new_case(
            "http://foo.com/bar",
            "mailto:foo@example.com",
            "mailto:foo@example.com",
        ),
        // Path-absolute references
        new_case("http://foo.com/bar", "/baz", "http://foo.com/baz"),
        new_case("http://foo.com/bar?a=b#f", "/baz", "http://foo.com/baz"),
        new_case("http://foo.com/bar?a=b", "/baz?", "http://foo.com/baz?"),
        new_case(
            "http://foo.com/bar?a=b",
            "/baz?c=d",
            "http://foo.com/baz?c=d",
        ),
        // Multiple slashes
        new_case(
            "http://foo.com/bar",
            "http://foo.com//baz",
            "http://foo.com//baz",
        ),
        new_case(
            "http://foo.com/bar",
            "http://foo.com///baz/quux",
            "http://foo.com///baz/quux",
        ),
        // Scheme-relative
        new_case(
            "https://foo.com/bar?a=b",
            "//bar.com/quux",
            "https://bar.com/quux",
        ),
        // Path-relative references:

        // ... current directory
        new_case("http://foo.com", ".", "http://foo.com/"),
        new_case("http://foo.com/bar", ".", "http://foo.com/"),
        new_case("http://foo.com/bar/", ".", "http://foo.com/bar/"),
        // ... going down
        new_case("http://foo.com", "bar", "http://foo.com/bar"),
        new_case("http://foo.com/", "bar", "http://foo.com/bar"),
        new_case("http://foo.com/bar/baz", "quux", "http://foo.com/bar/quux"),
        // ... going up
        new_case("http://foo.com/bar/baz", "../quux", "http://foo.com/quux"),
        new_case(
            "http://foo.com/bar/baz",
            "../../../../../quux",
            "http://foo.com/quux",
        ),
        new_case("http://foo.com/bar", "..", "http://foo.com/"),
        new_case("http://foo.com/bar/baz", "./..", "http://foo.com/"),
        // ".." in the middle (issue 3560)
        new_case(
            "http://foo.com/bar/baz",
            "quux/dotdot/../tail",
            "http://foo.com/bar/quux/tail",
        ),
        new_case(
            "http://foo.com/bar/baz",
            "quux/./dotdot/../tail",
            "http://foo.com/bar/quux/tail",
        ),
        new_case(
            "http://foo.com/bar/baz",
            "quux/./dotdot/.././tail",
            "http://foo.com/bar/quux/tail",
        ),
        new_case(
            "http://foo.com/bar/baz",
            "quux/./dotdot/./../tail",
            "http://foo.com/bar/quux/tail",
        ),
        new_case(
            "http://foo.com/bar/baz",
            "quux/./dotdot/dotdot/././../../tail",
            "http://foo.com/bar/quux/tail",
        ),
        new_case(
            "http://foo.com/bar/baz",
            "quux/./dotdot/dotdot/./.././../tail",
            "http://foo.com/bar/quux/tail",
        ),
        new_case(
            "http://foo.com/bar/baz",
            "quux/./dotdot/dotdot/dotdot/./../../.././././tail",
            "http://foo.com/bar/quux/tail",
        ),
        new_case(
            "http://foo.com/bar/baz",
            "quux/./dotdot/../dotdot/../dot/./tail/..",
            "http://foo.com/bar/quux/dot/",
        ),
        // Remove any dot-segments prior to forming the target URI.
        // http://tools.ietf.org/html/rfc3986#section-5.2.4
        new_case(
            "http://foo.com/dot/./dotdot/../foo/bar",
            "../baz",
            "http://foo.com/dot/baz",
        ),
        // Triple dot isn't special
        new_case("http://foo.com/bar", "...", "http://foo.com/..."),
        // Fragment
        new_case("http://foo.com/bar", ".#frag", "http://foo.com/#frag"),
        new_case(
            "http://example.org/",
            "#!$&%27()*+,;=",
            "http://example.org/#!$&%27()*+,;=",
        ),
        // Paths with escaping (issue 16947).
        new_case("http://foo.com/foo%2fbar/", "../baz", "http://foo.com/baz"),
        new_case(
            "http://foo.com/1/2%2f/3%2f4/5",
            "../../a/b/c",
            "http://foo.com/1/a/b/c",
        ),
        new_case(
            "http://foo.com/1/2/3",
            "./a%2f../../b/..%2fc",
            "http://foo.com/1/2/b/..%2fc",
        ),
        new_case(
            "http://foo.com/1/2%2f/3%2f4/5",
            "./a%2f../b/../c",
            "http://foo.com/1/2%2f/3%2f4/a%2f../c",
        ),
        new_case("http://foo.com/foo%20bar/", "../baz", "http://foo.com/baz"),
        new_case(
            "http://foo.com/foo",
            "../bar%2fbaz",
            "http://foo.com/bar%2fbaz",
        ),
        new_case(
            "http://foo.com/foo%2dbar/",
            "./baz-quux",
            "http://foo.com/foo%2dbar/baz-quux",
        ),
        // RFC 3986: Normal Examples
        // http://tools.ietf.org/html/rfc3986#section-5.4.1
        new_case("http://a/b/c/d;p?q", "g:h", "g:h"),
        new_case("http://a/b/c/d;p?q", "g", "http://a/b/c/g"),
        new_case("http://a/b/c/d;p?q", "./g", "http://a/b/c/g"),
        new_case("http://a/b/c/d;p?q", "g/", "http://a/b/c/g/"),
        new_case("http://a/b/c/d;p?q", "/g", "http://a/g"),
        new_case("http://a/b/c/d;p?q", "//g", "http://g"),
        new_case("http://a/b/c/d;p?q", "?y", "http://a/b/c/d;p?y"),
        new_case("http://a/b/c/d;p?q", "g?y", "http://a/b/c/g?y"),
        new_case("http://a/b/c/d;p?q", "#s", "http://a/b/c/d;p?q#s"),
        new_case("http://a/b/c/d;p?q", "g#s", "http://a/b/c/g#s"),
        new_case("http://a/b/c/d;p?q", "g?y#s", "http://a/b/c/g?y#s"),
        new_case("http://a/b/c/d;p?q", ";x", "http://a/b/c/;x"),
        new_case("http://a/b/c/d;p?q", "g;x", "http://a/b/c/g;x"),
        new_case("http://a/b/c/d;p?q", "g;x?y#s", "http://a/b/c/g;x?y#s"),
        new_case("http://a/b/c/d;p?q", "", "http://a/b/c/d;p?q"),
        new_case("http://a/b/c/d;p?q", ".", "http://a/b/c/"),
        new_case("http://a/b/c/d;p?q", "./", "http://a/b/c/"),
        new_case("http://a/b/c/d;p?q", "..", "http://a/b/"),
        new_case("http://a/b/c/d;p?q", "../", "http://a/b/"),
        new_case("http://a/b/c/d;p?q", "../g", "http://a/b/g"),
        new_case("http://a/b/c/d;p?q", "../..", "http://a/"),
        new_case("http://a/b/c/d;p?q", "../../", "http://a/"),
        new_case("http://a/b/c/d;p?q", "../../g", "http://a/g"),
        // RFC 3986: Abnormal Examples
        // http://tools.ietf.org/html/rfc3986#section-5.4.2
        new_case("http://a/b/c/d;p?q", "../../../g", "http://a/g"),
        new_case("http://a/b/c/d;p?q", "../../../../g", "http://a/g"),
        new_case("http://a/b/c/d;p?q", "/./g", "http://a/g"),
        new_case("http://a/b/c/d;p?q", "/../g", "http://a/g"),
        new_case("http://a/b/c/d;p?q", "g.", "http://a/b/c/g."),
        new_case("http://a/b/c/d;p?q", ".g", "http://a/b/c/.g"),
        new_case("http://a/b/c/d;p?q", "g..", "http://a/b/c/g.."),
        new_case("http://a/b/c/d;p?q", "..g", "http://a/b/c/..g"),
        new_case("http://a/b/c/d;p?q", "./../g", "http://a/b/g"),
        new_case("http://a/b/c/d;p?q", "./g/.", "http://a/b/c/g/"),
        new_case("http://a/b/c/d;p?q", "g/./h", "http://a/b/c/g/h"),
        new_case("http://a/b/c/d;p?q", "g/../h", "http://a/b/c/h"),
        new_case("http://a/b/c/d;p?q", "g;x=1/./y", "http://a/b/c/g;x=1/y"),
        new_case("http://a/b/c/d;p?q", "g;x=1/../y", "http://a/b/c/y"),
        new_case("http://a/b/c/d;p?q", "g?y/./x", "http://a/b/c/g?y/./x"),
        new_case("http://a/b/c/d;p?q", "g?y/../x", "http://a/b/c/g?y/../x"),
        new_case("http://a/b/c/d;p?q", "g#s/./x", "http://a/b/c/g#s/./x"),
        new_case("http://a/b/c/d;p?q", "g#s/../x", "http://a/b/c/g#s/../x"),
        // Extras.
        new_case("https://a/b/c/d;p?q", "//g?q", "https://g?q"),
        new_case("https://a/b/c/d;p?q", "//g#s", "https://g#s"),
        new_case(
            "https://a/b/c/d;p?q",
            "//g/d/e/f?y#s",
            "https://g/d/e/f?y#s",
        ),
        new_case("https://a/b/c/d;p#s", "?y", "https://a/b/c/d;p?y"),
        new_case("https://a/b/c/d;p?q#s", "?y", "https://a/b/c/d;p?y"),
    ];

    let opaque = URL {
        scheme: "scheme".to_string(),
        opaque: "opaque".to_string(),
        ..Default::default()
    };
    for c in test_vector {
        let base = super::super::parse(&c.base).unwrap();
        let rel = super::super::parse(&c.rel).unwrap();

        let url = base.resolve_reference(&rel);
        assert_eq!(
            c.expected,
            url.to_string(),
            "URL({}).resolve_reference({}) failed",
            c.base,
            c.rel
        );

        let got = base
            .parse(&c.rel)
            .map(|v| v.to_string())
            .map_err(|err| format!("URL({}).parse({}) failed: {}", c.base, c.rel, err))
            .unwrap();
        assert_eq!(c.expected, got, "URL({}).parse({}) failed", c.base, c.rel);

        let url = base.resolve_reference(&opaque);
        assert_eq!(
            opaque,
            url,
            "resolve_reference failed to resolve opaque URL: {}",
            base.to_string()
        );

        let url = base
            .parse("scheme:opaque")
            .map_err(|err| format!(r#"URL({}).parse("scheme:opaque") failed: {}"#, c.base, err))
            .unwrap();
        assert_eq!(url, opaque, "URL.parse() failed to resolve opaque URL");
    }
}

#[test]
fn star_request() {
    match super::super::parse("*") {
        Err(err) => panic!("unexpected error: {:?}", err),
        Ok(v) => assert_eq!("*", v.request_uri(), "invalid request URI"),
    }
}

#[test]
fn url_string() {
    let test_vector = url_test_vector();
    for c in test_vector {
        let u = match super::super::parse(&c.rawurl) {
            Err(err) => {
                println!("parse({}) returned error: {}", c.rawurl, err);
                continue;
            }
            Ok(v) => v,
        };

        let expected = if c.roundtrip == "" {
            &c.rawurl
        } else {
            &c.roundtrip
        };

        assert_eq!(
            expected,
            &u.to_string(),
            "parse({}).to_string() invalid",
            c.rawurl
        );
    }

    // more
    struct Case {
        url: URL,
        want: &'static str,
    }

    let test_vector = vec![
        Case {
            url: URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "search".to_string(),
                ..Default::default()
            },
            want: "http://www.google.com/search",
        },
        // Relative path with first element containing ":" should be prepended with "./", golang.org/issue/17184
        Case {
            url: URL {
                path: "this:that".to_string(),
                ..Default::default()
            },
            want: "./this:that",
        },
        // Relative path with second element containing ":" should not be prepended with "./"
        Case {
            url: URL {
                path: "here/this:that".to_string(),
                ..Default::default()
            },
            want: "here/this:that",
        },
        // Non-relative path with first element containing ":" should not be prepended with "./"
        Case {
            url: URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "this:that".to_string(),
                ..Default::default()
            },
            want: "http://www.google.com/this:that",
        },
    ];

    for c in test_vector {
        let got = c.url.to_string();
        assert_eq!(c.want, got, "URL.to_string({}) invalid", c.url);
    }
}

struct URLTest {
    rawurl: String,
    out: URL,          // expected parse
    roundtrip: String, // expected result of reserializing the URL; empty means same as "rawurl".
}

fn url_test_vector() -> Vec<URLTest> {
    let new_user = |name: &str| -> Option<UserInfo> { Some(super::super::user(name)) };
    let new_user_password = |name: &str, password: &str| -> Option<UserInfo> {
        Some(super::super::user_password(name, password))
    };

    let new_case = |rawurl: &str, out: URL, roundtrip: &str| -> URLTest {
        URLTest {
            rawurl: rawurl.to_string(),
            out,
            roundtrip: roundtrip.to_string(),
        }
    };

    vec![
        // no path
        new_case(
            "http://www.google.com",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                ..Default::default()
            },
            "",
        ),
        // path
        new_case(
            "http://www.google.com/",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "/".to_string(),
                ..Default::default()
            },
            "",
        ),
        // path with hex escaping
        new_case(
            "http://www.google.com/file%20one%26two",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "/file one&two".to_string(),
                raw_path: "/file%20one%26two".to_string(),
                ..Default::default()
            },
            "",
        ),
        // fragment with hex escaping
        new_case(
            "http://www.google.com/#file%20one%26two",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "/".to_string(),
                fragment: "file one&two".to_string(),
                raw_fragment: "file%20one%26two".to_string(),
                ..Default::default()
            },
            "",
        ),
        // user
        new_case(
            "ftp://webmaster@www.google.com/",
            URL {
                scheme: "ftp".to_string(),
                user: new_user("webmaster"),
                host: "www.google.com".to_string(),
                path: "/".to_string(),
                ..Default::default()
            },
            "",
        ),
        // escape sequence in username
        new_case(
            "ftp://john%20doe@www.google.com/",
            URL {
                scheme: "ftp".to_string(),
                user: new_user("john doe"),
                host: "www.google.com".to_string(),
                path: "/".to_string(),
                ..Default::default()
            },
            "ftp://john%20doe@www.google.com/",
        ),
        // empty query
        new_case(
            "http://www.google.com/?",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "/".to_string(),
                force_query: true,
                ..Default::default()
            },
            "",
        ),
        // query ending in question mark (Issue 14573)
        new_case(
            "http://www.google.com/?foo=bar?",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "/".to_string(),
                raw_query: "foo=bar?".to_string(),
                ..Default::default()
            },
            "",
        ),
        // query
        new_case(
            "http://www.google.com/?q=go+language",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "/".to_string(),
                raw_query: "q=go+language".to_string(),
                ..Default::default()
            },
            "",
        ),
        // query with hex escaping: NOT parsed
        new_case(
            "http://www.google.com/?q=go%20language",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "/".to_string(),
                raw_query: "q=go%20language".to_string(),
                ..Default::default()
            },
            "",
        ),
        // %20 outside query
        new_case(
            "http://www.google.com/a%20b?q=c+d",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "/a b".to_string(),
                raw_query: "q=c+d".to_string(),
                ..Default::default()
            },
            "",
        ),
        // path without leading /, so no parsing
        new_case(
            "http:www.google.com/?q=go+language",
            URL {
                scheme: "http".to_string(),
                opaque: "www.google.com/".to_string(),
                raw_query: "q=go+language".to_string(),
                ..Default::default()
            },
            "http:www.google.com/?q=go+language",
        ),
        // path without leading /, so no parsing
        new_case(
            "http:%2f%2fwww.google.com/?q=go+language",
            URL {
                scheme: "http".to_string(),
                opaque: "%2f%2fwww.google.com/".to_string(),
                raw_query: "q=go+language".to_string(),
                ..Default::default()
            },
            "http:%2f%2fwww.google.com/?q=go+language",
        ),
        // non-authority with path
        new_case(
            "mailto:/webmaster@golang.org",
            URL {
                scheme: "mailto".to_string(),
                path: "/webmaster@golang.org".to_string(),
                ..Default::default()
            },
            "mailto:///webmaster@golang.org", // unfortunate compromise
        ),
        // non-authority
        new_case(
            "mailto:webmaster@golang.org",
            URL {
                scheme: "mailto".to_string(),
                opaque: "webmaster@golang.org".to_string(),
                ..Default::default()
            },
            "",
        ),
        // unescaped :// in query should not create a scheme
        new_case(
            "/foo?query=http://bad",
            URL {
                path: "/foo".to_string(),
                raw_query: "query=http://bad".to_string(),
                ..Default::default()
            },
            "",
        ),
        // leading // without scheme should create an authority
        new_case(
            "//foo",
            URL {
                host: "foo".to_string(),
                ..Default::default()
            },
            "",
        ),
        // leading // without scheme, with userinfo, path, and query
        new_case(
            "//user@foo/path?a=b",
            URL {
                user: new_user("user"),
                host: "foo".to_string(),
                path: "/path".to_string(),
                raw_query: "a=b".to_string(),
                ..Default::default()
            },
            "",
        ),
        // Three leading slashes isn't an authority, but doesn't return an error.
        // (We can't return an error, as this code is also used via
        // ServeHTTP -> ReadRequest -> Parse, which is arguably a
        // different URL parsing context, but currently shares the
        // same codepath)
        new_case(
            "///threeslashes",
            URL {
                path: "///threeslashes".to_string(),
                ..Default::default()
            },
            "",
        ),
        new_case(
            "http://user:password@google.com",
            URL {
                scheme: "http".to_string(),
                user: new_user_password("user", "password"),
                host: "google.com".to_string(),
                ..Default::default()
            },
            "http://user:password@google.com",
        ),
        // unescaped @ in username should not confuse host
        new_case(
            "http://j@ne:password@google.com",
            URL {
                scheme: "http".to_string(),
                user: new_user_password("j@ne", "password"),
                host: "google.com".to_string(),
                ..Default::default()
            },
            "http://j%40ne:password@google.com",
        ),
        // unescaped @ in password should not confuse host
        new_case(
            "http://jane:p@ssword@google.com",
            URL {
                scheme: "http".to_string(),
                user: new_user_password("jane", "p@ssword"),
                host: "google.com".to_string(),
                ..Default::default()
            },
            "http://jane:p%40ssword@google.com",
        ),
        new_case(
            "http://j@ne:password@google.com/p@th?q=@go",
            URL {
                scheme: "http".to_string(),
                user: new_user_password("j@ne", "password"),
                host: "google.com".to_string(),
                path: "/p@th".to_string(),
                raw_query: "q=@go".to_string(),
                ..Default::default()
            },
            "http://j%40ne:password@google.com/p@th?q=@go",
        ),
        new_case(
            "http://www.google.com/?q=go+language#foo",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "/".to_string(),
                raw_query: "q=go+language".to_string(),
                fragment: "foo".to_string(),
                ..Default::default()
            },
            "",
        ),
        new_case(
            "http://www.google.com/?q=go+language#foo&bar",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "/".to_string(),
                raw_query: "q=go+language".to_string(),
                fragment: "foo&bar".to_string(),
                ..Default::default()
            },
            "http://www.google.com/?q=go+language#foo&bar",
        ),
        new_case(
            "http://www.google.com/?q=go+language#foo%26bar",
            URL {
                scheme: "http".to_string(),
                host: "www.google.com".to_string(),
                path: "/".to_string(),
                raw_query: "q=go+language".to_string(),
                fragment: "foo&bar".to_string(),
                raw_fragment: "foo%26bar".to_string(),
                ..Default::default()
            },
            "http://www.google.com/?q=go+language#foo%26bar",
        ),
        new_case(
            "file:///home/adg/rabbits",
            URL {
                scheme: "file".to_string(),
                host: "".to_string(),
                path: "/home/adg/rabbits".to_string(),
                ..Default::default()
            },
            "file:///home/adg/rabbits",
        ),
        // "Windows" paths are no exception to the rule.
        // See golang.org/issue/6027, especially comment #9.
        new_case(
            "file:///C:/FooBar/Baz.txt",
            URL {
                scheme: "file".to_string(),
                host: "".to_string(),
                path: "/C:/FooBar/Baz.txt".to_string(),
                ..Default::default()
            },
            "file:///C:/FooBar/Baz.txt",
        ),
        // case-insensitive scheme
        new_case(
            "MaIlTo:webmaster@golang.org",
            URL {
                scheme: "mailto".to_string(),
                opaque: "webmaster@golang.org".to_string(),
                ..Default::default()
            },
            "mailto:webmaster@golang.org",
        ),
        // Relative path
        new_case(
            "a/b/c",
            URL {
                path: "a/b/c".to_string(),
                ..Default::default()
            },
            "a/b/c",
        ),
        // escaped '?' in username and password
        new_case(
            "http://%3Fam:pa%3Fsword@google.com",
            URL {
                scheme: "http".to_string(),
                user: new_user_password("?am", "pa?sword"),
                host: "google.com".to_string(),
                ..Default::default()
            },
            "",
        ),
        // host subcomponent; IPv4 address in RFC 3986
        new_case(
            "http://192.168.0.1/",
            URL {
                scheme: "http".to_string(),
                host: "192.168.0.1".to_string(),
                path: "/".to_string(),
                ..Default::default()
            },
            "",
        ),
        // host and port subcomponents; IPv4 address in RFC 3986
        new_case(
            "http://192.168.0.1:8080/",
            URL {
                scheme: "http".to_string(),
                host: "192.168.0.1:8080".to_string(),
                path: "/".to_string(),
                ..Default::default()
            },
            "",
        ),
        // host subcomponent; IPv6 address in RFC 3986
        new_case(
            "http://[fe80::1]/",
            URL {
                scheme: "http".to_string(),
                host: "[fe80::1]".to_string(),
                path: "/".to_string(),
                ..Default::default()
            },
            "",
        ),
        // host and port subcomponents; IPv6 address in RFC 3986
        new_case(
            "http://[fe80::1]:8080/",
            URL {
                scheme: "http".to_string(),
                host: "[fe80::1]:8080".to_string(),
                path: "/".to_string(),
                ..Default::default()
            },
            "",
        ),
        // host subcomponent; IPv6 address with zone identifier in RFC 6874
        new_case(
            "http://[fe80::1%25en0]/", // alphanum zone identifier
            URL {
                scheme: "http".to_string(),
                host: "[fe80::1%en0]".to_string(),
                path: "/".to_string(),
                ..Default::default()
            },
            "",
        ),
        // host and port subcomponents; IPv6 address with zone identifier in RFC 6874
        new_case(
            "http://[fe80::1%25en0]:8080/", // alphanum zone identifier
            URL {
                scheme: "http".to_string(),
                host: "[fe80::1%en0]:8080".to_string(),
                path: "/".to_string(),
                ..Default::default()
            },
            "",
        ),
        // host subcomponent; IPv6 address with zone identifier in RFC 6874
        new_case(
            "http://[fe80::1%25%65%6e%301-._~]/", // percent-encoded+unreserved zone identifier
            URL {
                scheme: "http".to_string(),
                host: "[fe80::1%en01-._~]".to_string(),
                path: "/".to_string(),
                ..Default::default()
            },
            "http://[fe80::1%25en01-._~]/",
        ),
        // host and port subcomponents; IPv6 address with zone identifier in RFC 6874
        new_case(
            "http://[fe80::1%25%65%6e%301-._~]:8080/", // percent-encoded+unreserved zone identifier
            URL {
                scheme: "http".to_string(),
                host: "[fe80::1%en01-._~]:8080".to_string(),
                path: "/".to_string(),
                ..Default::default()
            },
            "http://[fe80::1%25en01-._~]:8080/",
        ),
        // alternate escapings of path survive round trip
        new_case(
            "http://rest.rsc.io/foo%2fbar/baz%2Fquux?alt=media",
            URL {
                scheme: "http".to_string(),
                host: "rest.rsc.io".to_string(),
                path: "/foo/bar/baz/quux".to_string(),
                raw_path: "/foo%2fbar/baz%2Fquux".to_string(),
                raw_query: "alt=media".to_string(),
                ..Default::default()
            },
            "",
        ),
        // issue 12036
        new_case(
            "mysql://a,b,c/bar",
            URL {
                scheme: "mysql".to_string(),
                host: "a,b,c".to_string(),
                path: "/bar".to_string(),
                ..Default::default()
            },
            "",
        ),
        // worst case host, still round trips
        new_case(
            "scheme://!$&'()*+,;=hello!:1/path",
            URL {
                scheme: "scheme".to_string(),
                host: "!$&'()*+,;=hello!:1".to_string(),
                path: "/path".to_string(),
                ..Default::default()
            },
            "",
        ),
        // worst case path, still round trips
        new_case(
            "http://host/!$&'()*+,;=:@[hello]",
            URL {
                scheme: "http".to_string(),
                host: "host".to_string(),
                path: "/!$&'()*+,;=:@[hello]".to_string(),
                raw_path: "/!$&'()*+,;=:@[hello]".to_string(),
                ..Default::default()
            },
            "",
        ),
        // golang.org/issue/5684
        new_case(
            "http://example.com/oid/[order_id]",
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                path: "/oid/[order_id]".to_string(),
                raw_path: "/oid/[order_id]".to_string(),
                ..Default::default()
            },
            "",
        ),
        // golang.org/issue/12200 (colon with empty port)
        new_case(
            "http://192.168.0.2:8080/foo",
            URL {
                scheme: "http".to_string(),
                host: "192.168.0.2:8080".to_string(),
                path: "/foo".to_string(),
                ..Default::default()
            },
            "",
        ),
        new_case(
            "http://192.168.0.2:/foo",
            URL {
                scheme: "http".to_string(),
                host: "192.168.0.2:".to_string(),
                path: "/foo".to_string(),
                ..Default::default()
            },
            "",
        ),
        new_case(
            // Malformed IPv6 but still accepted.
            "http://2b01:e34:ef40:7730:8e70:5aff:fefe:edac:8080/foo",
            URL {
                scheme: "http".to_string(),
                host: "2b01:e34:ef40:7730:8e70:5aff:fefe:edac:8080".to_string(),
                path: "/foo".to_string(),
                ..Default::default()
            },
            "",
        ),
        new_case(
            // Malformed IPv6 but still accepted.
            "http://2b01:e34:ef40:7730:8e70:5aff:fefe:edac:/foo",
            URL {
                scheme: "http".to_string(),
                host: "2b01:e34:ef40:7730:8e70:5aff:fefe:edac:".to_string(),
                path: "/foo".to_string(),
                ..Default::default()
            },
            "",
        ),
        new_case(
            "http://[2b01:e34:ef40:7730:8e70:5aff:fefe:edac]:8080/foo",
            URL {
                scheme: "http".to_string(),
                host: "[2b01:e34:ef40:7730:8e70:5aff:fefe:edac]:8080".to_string(),
                path: "/foo".to_string(),
                ..Default::default()
            },
            "",
        ),
        new_case(
            "http://[2b01:e34:ef40:7730:8e70:5aff:fefe:edac]:/foo",
            URL {
                scheme: "http".to_string(),
                host: "[2b01:e34:ef40:7730:8e70:5aff:fefe:edac]:".to_string(),
                path: "/foo".to_string(),
                ..Default::default()
            },
            "",
        ),
        // golang.org/issue/7991 and golang.org/issue/12719 (non-ascii %-encoded in host)
        new_case(
            "http://hello.世界.com/foo",
            URL {
                scheme: "http".to_string(),
                host: "hello.世界.com".to_string(),
                path: "/foo".to_string(),
                ..Default::default()
            },
            "http://hello.%E4%B8%96%E7%95%8C.com/foo",
        ),
        new_case(
            "http://hello.%e4%b8%96%e7%95%8c.com/foo",
            URL {
                scheme: "http".to_string(),
                host: "hello.世界.com".to_string(),
                path: "/foo".to_string(),
                ..Default::default()
            },
            "http://hello.%E4%B8%96%E7%95%8C.com/foo",
        ),
        new_case(
            "http://hello.%E4%B8%96%E7%95%8C.com/foo",
            URL {
                scheme: "http".to_string(),
                host: "hello.世界.com".to_string(),
                path: "/foo".to_string(),
                ..Default::default()
            },
            "",
        ),
        // golang.org/issue/10433 (path beginning with //)
        new_case(
            "http://example.com//foo",
            URL {
                scheme: "http".to_string(),
                host: "example.com".to_string(),
                path: "//foo".to_string(),
                ..Default::default()
            },
            "",
        ),
        // test that we can reparse the host names we accept.
        new_case(
            "myscheme://authority<\"hi\">/foo",
            URL {
                scheme: "myscheme".to_string(),
                host: "authority<\"hi\">".to_string(),
                path: "/foo".to_string(),
                ..Default::default()
            },
            "",
        ),
        // spaces in hosts are disallowed but escaped spaces in IPv6 scope IDs are grudgingly OK.
        // This happens on Windows.
        // golang.org/issue/14002
        new_case(
            "tcp://[2020::2020:20:2020:2020%25Windows%20Loves%20Spaces]:2020",
            URL {
                scheme: "tcp".to_string(),
                host: "[2020::2020:20:2020:2020%Windows Loves Spaces]:2020".to_string(),
                ..Default::default()
            },
            "",
        ),
        // test we can roundtrip magnet url
        // fix issue https://golang.org/issue/20054
        new_case(
            "magnet:?xt=urn:btih:c12fe1c06bba254a9dc9f519b335aa7c1367a88a&dn",
            URL {
                scheme: "magnet".to_string(),
                host: "".to_string(),
                path: "".to_string(),
                raw_query: "xt=urn:btih:c12fe1c06bba254a9dc9f519b335aa7c1367a88a&dn".to_string(),
                ..Default::default()
            },
            "magnet:?xt=urn:btih:c12fe1c06bba254a9dc9f519b335aa7c1367a88a&dn",
        ),
        new_case(
            "mailto:?subject=hi",
            URL {
                scheme: "mailto".to_string(),
                host: "".to_string(),
                path: "".to_string(),
                raw_query: "subject=hi".to_string(),
                ..Default::default()
            },
            "mailto:?subject=hi",
        ),
    ]
}
