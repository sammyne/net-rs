use url::URL;

fn main() {
    let mut u = URL {
        scheme: "https".to_string(),
        user: Some(url::user_password("user", "password")),
        host: "example.com".to_string(),
        path: "foo/bar".to_string(),
        ..Default::default()
    };

    assert_eq!("https://user:xxxxx@example.com/foo/bar", u.redacted());

    u.user = Some(url::user_password("me", "new_password"));

    assert_eq!("https://me:xxxxx@example.com/foo/bar", u.redacted());
}
