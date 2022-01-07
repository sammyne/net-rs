fn main() {
    let u = url::parse("https://example.org/path?foo=bar").unwrap();
    assert_eq!("/path?foo=bar", u.request_uri());
}
