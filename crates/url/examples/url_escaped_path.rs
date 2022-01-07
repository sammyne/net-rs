fn main() {
    let u = url::parse("http://example.com/x/y%2Fz").unwrap();

    assert_eq!("/x/y/z", u.path, "invalid path");
    assert_eq!("/x/y%2Fz", u.raw_path, "invalid raw path");
    assert_eq!("/x/y%2Fz", u.escaped_path(), "invalid escaped path");
}
