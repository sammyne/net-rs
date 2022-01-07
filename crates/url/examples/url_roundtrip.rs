fn main() {
    let u = url::parse("https://example.com/foo%2fbar").unwrap();

    assert_eq!("/foo/bar", u.path, "invalid path");
    assert_eq!("/foo%2fbar", u.raw_path, "invalid raw_path");
    assert_eq!(
        "https://example.com/foo%2fbar",
        u.to_string(),
        "invalid to_string() output"
    );
}
