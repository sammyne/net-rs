fn main() {
    let u = url::parse("https://example.org:8000/path").unwrap();
    assert_eq!("example.org", u.hostname());

    let u = url::parse("https://[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:17000").unwrap();
    assert_eq!("2001:0db8:85a3:0000:0000:8a2e:0370:7334", u.hostname());
}
