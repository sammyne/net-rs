use super::errors::Error;
use super::{Values, URL};

#[test]
fn parse_query() {
    use std::collections::HashMap;

    let m = super::parse_query("x=1&y=2&y=3;z").unwrap();

    let expected = {
        let mut v = HashMap::new();
        v.insert("x".to_string(), vec!["1".to_string()]);
        v.insert("y".to_string(), vec!["2".to_string(), "3".to_string()]);
        v.insert("z".to_string(), vec!["".to_string()]);
        v
    };

    assert_eq!(expected, m.0);
}

#[test]
fn url() {
    let mut u = super::parse("http://bing.com/search?q=dotnet").unwrap();

    u.scheme = "https".to_string();
    u.host = "google.com".to_string();

    let mut q = u.query();
    q.set("q", "golang");

    u.raw_query = q.encode();

    const EXPECTED: &str = "https://google.com/search?q=golang";
    assert_eq!(EXPECTED, u.to_string());
}

#[test]
fn url_escaped_fragment() {
    let u = super::parse("http://example.com/#x/y%2Fz").unwrap();

    assert_eq!("x/y/z", u.fragment, "invalid fragment");
    assert_eq!("x/y%2Fz", u.raw_fragment, "invalid raw fragment");
    assert_eq!("x/y%2Fz", u.escaped_fragment(), "invalid escaped fragment");
}

#[test]
fn url_escaped_path() {
    let u = super::parse("http://example.com/x/y%2Fz").unwrap();

    assert_eq!("/x/y/z", u.path, "invalid path");
    assert_eq!("/x/y%2Fz", u.raw_path, "invalid raw path");
    assert_eq!("/x/y%2Fz", u.escaped_path(), "invalid escaped path");
}

#[test]
fn url_hostname() {
    let u = super::parse("https://example.org:8000/path").unwrap();
    assert_eq!("example.org", u.hostname());

    let u = super::parse("https://[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:17000").unwrap();
    assert_eq!("2001:0db8:85a3:0000:0000:8a2e:0370:7334", u.hostname());
}

#[test]
fn url_is_abs() {
    let mut u = URL {
        host: "example.com".to_string(),
        path: "foo".to_string(),
        ..Default::default()
    };

    assert!(!u.is_abs());

    u.scheme = "http".to_string();

    assert!(u.is_abs());
}

#[test]
fn url_parse() {
    let u = super::parse("https://example.org").unwrap();
    let rel = u.parse("/foo").unwrap();
    assert_eq!("https://example.org/foo", rel.to_string());

    match u.parse(":foo") {
        Err(Error::Wrapped { .. }) => {}
        _ => panic!("should has a wrapped error"),
    }
}

#[test]
fn url_port() {
    let u = super::parse("https://example.org").unwrap();
    assert_eq!("", u.port());

    let u = super::parse("https://example.org:8080").unwrap();
    assert_eq!("8080", u.port());
}

#[test]
fn url_query() {
    let q = super::parse("https://example.org/?a=1&a=2&b=&=3&&&&")
        .unwrap()
        .query();

    assert_eq!(vec!["1".to_string(), "2".to_string()], q.0["a"]);
    assert_eq!("", q.get("b").unwrap());
    assert_eq!("3", q.get("").unwrap());
}

#[test]
fn url_redacted() {
    let mut u = URL {
        scheme: "https".to_string(),
        user: Some(super::user_password("user", "password")),
        host: "example.com".to_string(),
        path: "foo/bar".to_string(),
        ..Default::default()
    };

    assert_eq!("https://user:xxxxx@example.com/foo/bar", u.redacted());

    u.user = Some(super::user_password("me", "new_password"));

    assert_eq!("https://me:xxxxx@example.com/foo/bar", u.redacted());
}

#[test]
fn url_request_uri() {
    let u = super::parse("https://example.org/path?foo=bar").unwrap();
    assert_eq!("/path?foo=bar", u.request_uri());
}

#[test]
fn url_resolve_reference() {
    let u = super::parse("../../..//search?q=dotnet").unwrap();
    let base = super::parse("http://example.com/directory/").unwrap();

    let got = base.resolve_reference(&u);
    const EXPECTED: &str = "http://example.com/search?q=dotnet";
    assert_eq!(EXPECTED, got.to_string());
}

#[test]
fn url_roundtrip() {
    let u = super::parse("https://example.com/foo%2fbar").unwrap();

    assert_eq!("/foo/bar", u.path, "invalid path");
    assert_eq!("/foo%2fbar", u.raw_path, "invalid raw_path");
    assert_eq!(
        "https://example.com/foo%2fbar",
        u.to_string(),
        "invalid to_string() output"
    );
}

#[test]
fn url_string() {
    let mut u = URL {
        scheme: "https".to_string(),
        user: Some(super::user_password("me", "pass")),
        host: "example.com".to_string(),
        path: "foo/bar".to_string(),
        raw_query: "x=1&y=2".to_string(),
        fragment: "anchor".to_string(),
        ..Default::default()
    };

    assert_eq!(
        "https://me:pass@example.com/foo/bar?x=1&y=2#anchor",
        u.to_string()
    );

    u.opaque = "opaque".to_string();
    assert_eq!("https:opaque?x=1&y=2#anchor", u.to_string());
}

#[test]
fn values() {
    let mut v = Values::default();

    v.set("name".to_string(), "Ava");
    v.add("friend", "Jess");
    v.add("friend", "Sarah");
    v.add("friend", "Zoe");

    // v.Encode() == "name=Ava&friend=Jess&friend=Sarah&friend=Zoe"
    assert_eq!(v.get("name").unwrap(), "Ava");
    assert_eq!(v.get("friend").unwrap(), "Jess");

    let friends: Vec<String> = vec!["Jess", "Sarah", "Zoe"]
        .iter()
        .map(|v| v.to_string())
        .collect();
    assert_eq!(v.0["friend"], friends);
}
