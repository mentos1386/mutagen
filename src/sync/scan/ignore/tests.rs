//! Provides tests for the ignore module. We don't go overboard with the testing
//! the pattern matching, because that's well-tested in the `glob` crate. All we
//! really need to verify is that the negation syntax is working correctly.

use super::Ignorer;

/// Generates tests of `Ignorer`.
macro_rules! ignorer_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (ignores, tests): (Vec<&str>, Vec<(&str, bool)>) = $value;
            let ignorer = Ignorer::new(ignores).unwrap();
            for t in tests.iter() {
                assert_eq!(ignorer.ignored(t.0), t.1);
            }
        }
    )*
    }
}

ignorer_tests! {
    null: (
        vec![],
        vec![
            ("something", false),
            ("some/path", false)
        ]
    ),
    empty: (
        // In practice, this ignore specification wouldn't make any sense,
        // because it would be equivalent to ignoring the root. A scan would
        // never check the root to see whether or not it's ignored. But it's
        // still best to make sure it parses and behaves as expected.
        vec![""],
        vec![
            ("", true),
            ("something", false),
            ("some/path", false)
        ]
    ),
    basic: (
        vec!["something", "otherthing", "!something"],
        vec![
            ("", false),
            ("something", false),
            ("something/other", false),
            ("otherthing", true),
            ("some/path", false)
        ]
    ),
    negate_ordering: (
        vec!["!something", "otherthing", "something"],
        vec![
            ("", false),
            ("something", true),
            ("something/other", false),
            ("otherthing", true),
            ("some/path", false)
        ]
    ),
    wildcard: (
        vec!["some*", "!someone"],
        vec![
            ("", false),
            ("som", false),
            ("some", true),
            ("something", true),
            ("someone", false),
            ("some/path", true)
        ]
    ),
    path_wildcard: (
        vec!["some/*", "!some/other"],
        vec![
            ("", false),
            ("something", false),
            ("some", false),
            ("some/path", true),
            ("some/other", false),
            ("some/other/path", true)
        ]
    ),
}
