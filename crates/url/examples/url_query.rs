fn main() {
    let q = url::parse("https://example.org/?a=1&a=2&b=&=3&&&&")
        .unwrap()
        .query();

    assert_eq!(vec!["1".to_string(), "2".to_string()], q.0["a"]);
    assert_eq!("", q.get("b").unwrap());
    assert_eq!("3", q.get("").unwrap());
}
