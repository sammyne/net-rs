fn main() {
    let u = url::parse("http://example.com/#x/y%2Fz").unwrap();

    assert_eq!("x/y/z", u.fragment, "invalid fragment");
    assert_eq!("x/y%2Fz", u.raw_fragment, "invalid raw fragment");
    assert_eq!("x/y%2Fz", u.escaped_fragment(), "invalid escaped fragment");
}
