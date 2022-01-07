fn main() {
    let u = url::parse("../../..//search?q=dotnet").unwrap();
    let base = url::parse("http://example.com/directory/").unwrap();

    let got = base.resolve_reference(&u);
    const EXPECTED: &str = "http://example.com/search?q=dotnet";
    assert_eq!(EXPECTED, got.to_string());
}
