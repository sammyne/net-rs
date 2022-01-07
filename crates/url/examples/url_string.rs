use url::URL;

fn main() {
    let mut u = URL {
        scheme: "https".to_string(),
        user: Some(url::user_password("me", "pass")),
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
