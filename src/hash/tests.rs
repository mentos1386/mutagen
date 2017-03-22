//! Provides tests for the `hash` module.

use std::io;
use super::{Algorithm, Hasher};

/// Generates tests of hash algorithms.
macro_rules! hasher_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let (value, algorithm, expected) = $value;
            let mut reader = value.as_bytes();
            let mut hasher = Hasher::new(&algorithm);
            let copied = io::copy(&mut reader, &mut hasher).unwrap() as usize;
            assert_eq!(copied, value.len());
            let digest = format!("{}", hasher.digest());
            assert_eq!(digest, expected);
        }
    )*
    }
}

hasher_tests! {
    sha1_empty: (
        "",
        Algorithm::Sha1,
        "da39a3ee5e6b4b0d3255bfef95601890afd80709",
    ),
    sha1_data: (
        "The quick brown fox jumps over the lazy dog.",
        Algorithm::Sha1,
        "408d94384216f890ff7a0c3528e8bed1e0b01621",
    ),
    blake2b_16_empty: (
        "",
        Algorithm::Blake2b(16),
        "cae66941d9efbd404e4d88758ea67670"
    ),
    blake2b_16_data: (
        "The quick brown fox jumps over the lazy dog.",
        Algorithm::Blake2b(16),
        "e21b9ba9a54571d92988718c8629e81b"
    ),
    blake2b_32_empty: (
        "",
        Algorithm::Blake2b(32),
        "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8"
    ),
    blake2b_32_data: (
        "The quick brown fox jumps over the lazy dog.",
        Algorithm::Blake2b(32),
        "69d7d3b0afba81826d27024c17f7f183659ed0812cf27b382eaef9fdc29b5712"
    ),
    blake2b_64_empty: (
        "",
        Algorithm::Blake2b(64),
        "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e10\
        31afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce"
    ),
    blake2b_64_data: (
        "The quick brown fox jumps over the lazy dog.",
        Algorithm::Blake2b(64),
        "87af9dc4afe5651b7aa89124b905fd214bf17c79af58610db86a0fb1e0194622a4e9d8\
        e395b352223a8183b0d421c0994b98286cbf8c68a495902e0fe6e2bda2",
    ),
}
