use url::URL;

fn main() {
    let mut u = URL {
        host: "example.com".to_string(),
        path: "foo".to_string(),
        ..Default::default()
    };

    assert!(!u.is_abs());

    u.scheme = "http".to_string();

    assert!(u.is_abs());
}
