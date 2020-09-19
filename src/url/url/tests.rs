#[test]
fn resolve_path() {
    struct Case {
        base: &'static str,
        reference: &'static str,
        expected: &'static str,
    }

    let new_case = |base, reference, expected| -> Case {
        Case {
            base,
            reference,
            expected,
        }
    };

    let test_vector = vec![
        new_case("a/b", ".", "/a/"),
        new_case("a/b", "c", "/a/c"),
        new_case("a/b", "..", "/"),
        new_case("a/", "..", "/"),
        new_case("a/", "../..", "/"),
        new_case("a/b/c", "..", "/a/"),
        new_case("a/b/c", "../d", "/a/d"),
        new_case("a/b/c", ".././d", "/a/d"),
        new_case("a/b", "./..", "/"),
        new_case("a/./b", ".", "/a/"),
        new_case("a/../", ".", "/"),
        new_case("a/.././b", "c", "/c"),
    ];

    for c in test_vector {
        let got = super::resolve_path(c.base, c.reference);
        assert_eq!(c.expected, got);
    }
}
