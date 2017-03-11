//! Provides tests for the time module.

use std::time::{UNIX_EPOCH, SystemTime, Duration};
use protobuf::Message;
use super::{
    TIMESTAMP_MINIMUM_SECONDS,
    TIMESTAMP_MAXIMUM_SECONDS,
    Timestamp,
    AsTimestamp
};

fn verify_serialize_roundtrip(timestamp: Timestamp) {
    let mut decoded = Timestamp::new();
    decoded.merge_from_bytes(
        timestamp.write_to_bytes().unwrap().as_slice()
    ).unwrap();
    assert_eq!(decoded.seconds, timestamp.seconds);
    assert_eq!(decoded.nanos, timestamp.nanos);
}

#[test]
fn unix_epoch() {
    let timestamp = UNIX_EPOCH.as_timestamp().unwrap();
    assert_eq!(timestamp.seconds, 0);
    assert_eq!(timestamp.nanos, 0);
    verify_serialize_roundtrip(timestamp);
}

#[test]
fn now() {
    verify_serialize_roundtrip(SystemTime::now().as_timestamp().unwrap());
}

#[test]
fn too_low() {
    let time = UNIX_EPOCH
                - Duration::new(((-TIMESTAMP_MINIMUM_SECONDS) as u64) + 1, 0);
    assert!(time.as_timestamp().is_err());
}

#[test]
fn too_low_by_nanosecond() {
    let time = UNIX_EPOCH
                - Duration::new((-TIMESTAMP_MINIMUM_SECONDS) as u64, 1);
    assert!(time.as_timestamp().is_err());
}

#[test]
fn lowest() {
    let time = UNIX_EPOCH
                - Duration::new((-TIMESTAMP_MINIMUM_SECONDS) as u64, 0);
    verify_serialize_roundtrip(time.as_timestamp().unwrap());
}

#[test]
fn highest() {
    let time = UNIX_EPOCH
                + Duration::new(
                    (TIMESTAMP_MAXIMUM_SECONDS - 1) as u64,
                    999_999_999
                );
    verify_serialize_roundtrip(time.as_timestamp().unwrap());
}

#[test]
fn too_high() {
    let time = UNIX_EPOCH
                + Duration::new(TIMESTAMP_MAXIMUM_SECONDS as u64, 0);
    assert!(time.as_timestamp().is_err());
}

// TODO: Add tests for Order.
