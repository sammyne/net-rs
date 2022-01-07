fn main() {
    let u = url::parse("https://example.org").unwrap();
    assert_eq!("", u.port());

    let u = url::parse("https://example.org:8080").unwrap();
    assert_eq!("8080", u.port());
}
