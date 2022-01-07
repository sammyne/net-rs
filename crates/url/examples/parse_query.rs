use std::collections::HashMap;

fn main() {
    let m = url::parse_query("x=1&y=2&y=3;z").unwrap();

    let expected = {
        let mut v = HashMap::new();
        v.insert("x".to_string(), vec!["1".to_string()]);
        v.insert("y".to_string(), vec!["2".to_string(), "3".to_string()]);
        v.insert("z".to_string(), vec!["".to_string()]);
        v
    };

    assert_eq!(expected, m.0);
}
