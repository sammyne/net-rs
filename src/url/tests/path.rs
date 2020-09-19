#[test]
fn path_escape() {
    struct Case {
        s: String,
        expect: String,
    }

    let new_case = |s: &str, expect: &str| -> Case {
        Case {
            s: s.to_string(),
            expect: expect.to_string(),
        }
    };

    let test_vector = vec![
    new_case("", ""),
    new_case("abc", "abc"),
    new_case("abc+def", "abc+def"),
    new_case("a/b", "a%2Fb"),
    new_case("one two", "one%20two"),
    new_case("10%", "10%25"),
    new_case(
      " ?&=#+%!<>#\"{}|\\^[]`☺\t:/@$'()*,;",
      "%20%3F&=%23+%25%21%3C%3E%23%22%7B%7D%7C%5C%5E%5B%5D%60%E2%98%BA%09:%2F@$%27%28%29%2A%2C%3B",
    ),
  ];

    for c in test_vector {
        let got = super::super::path_escape(c.s.as_str());
        assert_eq!(
            c.expect, got,
            "path_escape({})={}, want {}",
            c.s, got, c.expect
        );

        let roundtrip = super::super::path_unescape(got.as_str()).unwrap();
        assert_eq!(
            c.s, roundtrip,
            "path_unescape({})={}, expect {}",
            got, roundtrip, c.s
        );
    }
}
