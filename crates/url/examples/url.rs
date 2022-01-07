fn main() {
    let mut u = url::parse("http://bing.com/search?q=dotnet").unwrap();

    u.scheme = "https".to_string();
    u.host = "google.com".to_string();

    let mut q = u.query();
    q.set("q", "golang");

    u.raw_query = q.encode();

    const EXPECTED: &str = "https://google.com/search?q=golang";
    assert_eq!(EXPECTED, u.to_string());
}
