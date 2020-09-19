use super::errors::*;

#[test]
fn unescape() {
    let test_vector = unescape_test_vector();

    let is_equal = |a: &Result<String, Error>, b: &Result<String, Error>| -> bool {
        match a {
            Ok(v) if b.is_ok() => b.as_ref().unwrap() == v,
            Err(err) if b.is_err() => {
                let x = format!("{:?}", b.as_ref().unwrap_err());
                let y = format!("{:?}", err);
                x == y
            }
            _ => false,
        }
    };

    for c in test_vector {
        let got = super::query_unescape(c.rawurl);
        assert!(
            is_equal(&c.out, &got),
            "query_unescape({}) failed",
            c.rawurl
        );

        let mut rawurl = c.rawurl.to_string();
        let expected = if c.rawurl.contains('+') {
            rawurl = c.rawurl.replace('+', "%20");

            let got = super::path_unescape(&rawurl);
            assert!(is_equal(&c.out, &got), "path_unescape({}) failed", rawurl);

            let mut expected = c.out;
            if expected.is_ok() {
                let s = match super::query_unescape(&c.rawurl.replace('+', "XXX")) {
                    Err(_) => continue,
                    Ok(v) => v,
                };

                rawurl = c.rawurl.to_string();
                expected = expected.map(|_| s.replace("XXX", "+"));
            }

            expected
        } else {
            c.out.map(|v| v.to_string())
        };

        let got = super::path_unescape(&rawurl);
        assert!(
            is_equal(&expected, &got),
            "path_unescape({}) failed",
            rawurl
        );
    }
}

struct EscapeTest {
    rawurl: &'static str,
    out: Result<String, Error>,
}

fn unescape_test_vector() -> Vec<EscapeTest> {
    let new_case = |rawurl, out: Result<&'static str, Error>| -> EscapeTest {
        EscapeTest {
            rawurl,
            out: out.map(|v| v.to_string()),
        }
    };

    vec![
        new_case("", Ok("")),
        new_case("abc", Ok("abc")),
        new_case("1%41", Ok("1A")),
        new_case("1%41%42%43", Ok("1ABC")),
        new_case("%4a", Ok("J")),
        new_case("%6F", Ok("o")),
        new_case("%", Err(Error::Escape("%".to_string()))), // not enough characters after %
        new_case("%a", Err(Error::Escape("%a".to_string()))), // not enough characters after %
        new_case("%1", Err(Error::Escape("%1".to_string()))), // not enough characters after %
        new_case("123%45%6", Err(Error::Escape("%6".to_string()))), // invalid hex digits
        new_case("%zzzzz", Err(Error::Escape("%zz".to_string()))),
        new_case("a+b", Ok("a b")),
        new_case("a%20b", Ok("a b")),
    ]
}

mod internal;
mod path;
mod query;
mod url;
mod values;
