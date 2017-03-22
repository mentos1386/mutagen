//! Provides a custom path join function for sync algorithms.

/// Performs a path join. This method is used inside the `scan` module to
/// compute root-relative paths. It is required because we need joins that are
/// consistent across all platforms (`Path::join` will use `\` to join on
/// Windows), generate owned UTF-8 strings (`Path::join` will generate a
/// `PathBuf` that requires conversion/validation), and give better performance
/// (this function assumes both components are `/`-stripped).
pub fn join(first: &str, second: &str) -> String {
    if first.len() == 0 {
        second.to_owned()
    } else {
        format!("{}/{}", first, second)
    }
}

#[cfg(test)]
mod tests {
    use super::join;

    /// Generates tests of `join`.
    macro_rules! join_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (first, second, expected) = $value;
                assert_eq!(join(first, second), expected);
            }
        )*
        }
    }

    join_tests! {
        join_first_empty: ("", "something", "something"),
        join_first_empty_second_path: ("", "some/path", "some/path"),
        join_both_present: ("some", "path", "some/path"),
        join_second_path: ("some", "path/other", "some/path/other"),
        join_first_path: ("some/path", "other", "some/path/other"),
        join_both_path: ("some/path", "other/path", "some/path/other/path"),
    }
}
