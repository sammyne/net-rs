use std::collections::HashMap;

use super::super::Values;

#[test]
fn encode_query() {
    struct Case {
        values: Values,
        expect: String,
    }

    let new_case = |values: Vec<(&str, Vec<&str>)>, expect: &str| -> Case {
        let mut vs = HashMap::<String, Vec<String>>::new();

        for v in values {
            let k = v.0.to_string();
            let v = v.1.iter().map(|v| v.to_string()).collect::<Vec<_>>();
            vs.insert(k, v);
        }

        Case {
            values: Values(vs),
            expect: expect.to_string(),
        }
    };

    let test_vector = vec![
        new_case(vec![], ""),
        new_case(
            vec![("q", vec!["puppies"]), ("oe", vec!["utf8"])],
            "oe=utf8&q=puppies",
        ),
        new_case(vec![("q", vec!["dogs", "&", "7"])], "q=dogs&q=%26&q=7"),
        new_case(
            vec![
                ("a", vec!["a1", "a2", "a3"]),
                ("b", vec!["b1", "b2", "b3"]),
                ("c", vec!["c1", "c2", "c3"]),
            ],
            "a=a1&a=a2&a=a3&b=b1&b=b2&b=b3&c=c1&c=c2&c=c3",
        ),
    ];

    for c in test_vector {
        let got = c.values.encode();
        assert_eq!(
            c.expect, got,
            "encode_query()={:?}, want {:?}",
            got, c.expect
        );
    }
}

#[test]
fn parse_query() {
    struct Case {
        s: String,
        expect: HashMap<String, Vec<String>>,
    }

    let new_case = |s: &str, expect: Vec<(&str, Vec<&str>)>| -> Case {
        let mut e = HashMap::<String, Vec<String>>::new();

        for v in expect {
            let k = v.0.to_string();
            let v = v.1.iter().map(|v| v.to_string()).collect::<Vec<_>>();
            e.insert(k, v);
        }

        Case {
            s: s.to_string(),
            expect: e,
        }
    };

    let test_vector = vec![
        new_case("a=1&b=2", vec![("a", vec!["1"]), ("b", vec!["2"])]),
        new_case("a=1&a=2&a=banana", vec![("a", vec!["1", "2", "banana"])]),
        new_case(
            "ascii=%3Ckey%3A+0x90%3E",
            vec![("ascii", vec!["<key: 0x90>"])],
        ),
        new_case("a=1;b=2", vec![("a", vec!["1"]), ("b", vec!["2"])]),
        new_case("a=1&a=2;a=banana", vec![("a", vec!["1", "2", "banana"])]),
    ];

    for c in test_vector {
        let got = super::super::parse_query(c.s.as_str()).unwrap();
        assert_eq!(
            c.expect, got.0,
            "parse_query({})={:?}, want {:?}",
            c.s, got.0, c.expect
        );
    }
}
