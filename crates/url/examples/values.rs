use url::Values;

fn main() {
    let mut v = Values::default();

    v.set("name".to_string(), "Ava");
    v.add("friend", "Jess");
    v.add("friend", "Sarah");
    v.add("friend", "Zoe");

    // v.Encode() == "name=Ava&friend=Jess&friend=Sarah&friend=Zoe"
    assert_eq!(v.get("name").unwrap(), "Ava");
    assert_eq!(v.get("friend").unwrap(), "Jess");

    let friends: Vec<String> = vec!["Jess", "Sarah", "Zoe"]
        .iter()
        .map(|v| v.to_string())
        .collect();
    assert_eq!(v.0["friend"], friends);
}
