use crate::internal::Encoding;

#[test]
fn should_escape() {
    struct Case {
        c: u8,
        mode: Encoding,
        escape: bool,
    }

    let new_case = |c: char, mode, escape| -> Case {
        Case {
            c: c as u8,
            mode,
            escape,
        }
    };

    let test_vector = vec![
        // Unreserved characters (§2.3)
        new_case('a', Encoding::Path, false),
        new_case('a', Encoding::UserPassword, false),
        new_case('a', Encoding::QueryComponent, false),
        new_case('a', Encoding::Fragment, false),
        new_case('a', Encoding::Host, false),
        new_case('z', Encoding::Path, false),
        new_case('A', Encoding::Path, false),
        new_case('Z', Encoding::Path, false),
        new_case('0', Encoding::Path, false),
        new_case('9', Encoding::Path, false),
        new_case('-', Encoding::Path, false),
        new_case('-', Encoding::UserPassword, false),
        new_case('-', Encoding::QueryComponent, false),
        new_case('-', Encoding::Fragment, false),
        new_case('.', Encoding::Path, false),
        new_case('_', Encoding::Path, false),
        new_case('~', Encoding::Path, false),
        // User information (§3.2.1)
        new_case(':', Encoding::UserPassword, true),
        new_case('/', Encoding::UserPassword, true),
        new_case('?', Encoding::UserPassword, true),
        new_case('@', Encoding::UserPassword, true),
        new_case('$', Encoding::UserPassword, false),
        new_case('&', Encoding::UserPassword, false),
        new_case('+', Encoding::UserPassword, false),
        new_case(',', Encoding::UserPassword, false),
        new_case(';', Encoding::UserPassword, false),
        new_case('=', Encoding::UserPassword, false),
        // Host (IP address, IPv6 address, registered name, port suffix; §3.2.2)
        new_case('!', Encoding::Host, false),
        new_case('$', Encoding::Host, false),
        new_case('&', Encoding::Host, false),
        new_case('\'', Encoding::Host, false),
        new_case('(', Encoding::Host, false),
        new_case(')', Encoding::Host, false),
        new_case('*', Encoding::Host, false),
        new_case('+', Encoding::Host, false),
        new_case(',', Encoding::Host, false),
        new_case(';', Encoding::Host, false),
        new_case('=', Encoding::Host, false),
        new_case(':', Encoding::Host, false),
        new_case('[', Encoding::Host, false),
        new_case(']', Encoding::Host, false),
        new_case('0', Encoding::Host, false),
        new_case('9', Encoding::Host, false),
        new_case('A', Encoding::Host, false),
        new_case('z', Encoding::Host, false),
        new_case('_', Encoding::Host, false),
        new_case('-', Encoding::Host, false),
        new_case('.', Encoding::Host, false),
    ];

    for c in test_vector {
        let got = super::super::internal::should_escape(c.c, c.mode);
        assert_eq!(c.escape, got, "should_escape({}, {:?}) failed", c.c, c.mode);
    }
}
