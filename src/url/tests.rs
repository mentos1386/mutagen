//! Provides tests for the url module.

use protobuf::Message;
use super::{parse, Protocol, URL};

/// Generates tests of the parse function.
macro_rules! parse_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (raw, succeed, proto, user, host, port, path) = $value;
            match parse(raw) {
                Ok(url) => {
                    // Verify components.
                    assert_eq!(url.protocol, proto);
                    assert_eq!(url.username, user);
                    assert_eq!(url.hostname, host);
                    assert_eq!(url.port, port);
                    assert_eq!(url.path, path);

                    // Perform a serialization/deserialization cycle and
                    // reverify components.
                    let mut decoded = URL::new();
                    decoded.merge_from_bytes(
                        url.write_to_bytes().unwrap().as_slice()
                    ).unwrap();
                    assert_eq!(decoded.protocol, proto);
                    assert_eq!(decoded.username, user);
                    assert_eq!(decoded.hostname, host);
                    assert_eq!(decoded.port, port);
                    assert_eq!(decoded.path, path);
                },
                Err(_) => {
                    // Ensure that parsing was supposed to fail.
                    assert!(!succeed);
                }
            }
        }
    )*
    }
}

parse_tests! {
    parse_empty_invalid: (
        "",
        false,
        Protocol::Local,
        "",
        "",
        0,
        ""
    ),
    parse_empty_hostname_and_path_invalid: (
        ":",
        false,
        Protocol::Local,
        "",
        "",
        0,
        ""
    ),
    parse_empty_hostname_invalid: (
        ":path",
        false,
        Protocol::Local,
        "",
        "",
        0,
        ""
    ),
    parse_username_empty_hostname_invalid: (
        "user@:path",
        false,
        Protocol::Local,
        "",
        "",
        0,
        ""
    ),
    parse_path: (
        "/this/is/a:path",
        true,
        Protocol::Local,
        "",
        "",
        0,
        "/this/is/a:path"
    ),
    parse_username_hostname_is_local: (
        "user@host",
        true,
        Protocol::Local,
        "",
        "",
        0,
        "user@host"
    ),
    parse_hostname_empty_path: (
        "host:",
        true,
        Protocol::SSH,
        "",
        "host",
        0,
        ""
    ),
    parse_hostname_path: (
        "host:path",
        true,
        Protocol::SSH,
        "",
        "host",
        0,
        "path"
    ),
    parse_username_hostname_path: (
        "user@host:path",
        true,
        Protocol::SSH,
        "user",
        "host",
        0,
        "path"
    ),
    parse_username_hostname_port_path: (
        "user@host:65535:path",
        true,
        Protocol::SSH,
        "user",
        "host",
        65535,
        "path"
    ),
    parse_username_hostname_zero_port_path: (
        "user@host:0:path",
        true,
        Protocol::SSH,
        "user",
        "host",
        0,
        "path"
    ),
    parse_username_hostname_double_zero_port_path: (
        "user@host:00:path",
        true,
        Protocol::SSH,
        "user",
        "host",
        0,
        "path"
    ),
    parse_username_hostname_numeric_path: (
        "user@host:65536:path",
        true,
        Protocol::SSH,
        "user",
        "host",
        0,
        "65536:path"
    ),
    parse_username_hostname_hex_numeric_path: (
        "user@host:aaa:path",
        true,
        Protocol::SSH,
        "user",
        "host",
        0,
        "aaa:path"
    ),
    parse_unicode_username_hostname_path: (
        "üsér@høst:пат",
        true,
        Protocol::SSH,
        "üsér",
        "høst",
        0,
        "пат"
    ),
    parse_unicode_username_hostname_port_path: (
        "üsér@høst:23:пат",
        true,
        Protocol::SSH,
        "üsér",
        "høst",
        23,
        "пат"
    ),
}

#[cfg(windows)]
parse_tests! {
    parse_windows_path: (
        r"C:\something",
        true,
        Protocol::Local,
        "",
        "",
        0,
        r"C:\something"
    ),
    parse_windows_path_forward: (
        r"C:/something",
        true,
        Protocol::Local,
        "",
        "",
        0,
        r"C:/something"
    ),
    parse_windows_path_small: (
        r"c:\something",
        true,
        Protocol::Local,
        "",
        "",
        0,
        r"c:\something"
    ),
}

#[cfg(unix)]
parse_tests! {
    parse_windows_path: (
        r"C:\something",
        true,
        Protocol::SSH,
        "",
        "C",
        0,
        r"\something"
    ),
    parse_windows_path_forward: (
        r"C:/something",
        true,
        Protocol::SSH,
        "",
        "C",
        0,
        r"/something"
    ),
    parse_windows_path_small: (
        r"c:\something",
        true,
        Protocol::SSH,
        "",
        "c",
        0,
        r"\something"
    ),
}
