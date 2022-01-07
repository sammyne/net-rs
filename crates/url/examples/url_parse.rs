use url::errors::Error;

fn main() {
    let u = url::parse("https://example.org").unwrap();
    let rel = u.parse("/foo").unwrap();
    assert_eq!("https://example.org/foo", rel.to_string());

    match u.parse(":foo") {
        Err(Error::Wrapped { .. }) => {}
        _ => panic!("should has a wrapped error"),
    }
}
