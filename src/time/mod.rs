//! Provides facilities for converting `SystemTime` objects to Protocol Buffers
//! timestamps. This package will be necessary until the Rust Protocol Buffers
//! implementation supports Well-Known Types.

#[cfg(test)]
mod tests;

use std::time::{SystemTime, UNIX_EPOCH};
use super::errors::Result;
use super::proto::time::Timestamp;

/// Represents the minimum value that a `Timestamp` `seconds` field can take.
/// Protocol Buffers timestamps use an `i64` to represent seconds since the Unix
/// epoch, but all timestamps are artificially restricted to occur between
/// `0001-01-01T00:00:00Z` and `9999-12-31T23:59:59.999999999Z`. The lower time
/// bound corresponds to this seconds value. Since nanoseconds always count
/// forward in Protocol Buffers timestamps, the `seconds` field must always be
/// equal to or greater than this value.
const TIMESTAMP_MINIMUM_SECONDS: i64 = -62135596800;

/// Represents the`seconds` field of a hypothetical `Timestamp` one second after
/// the maximum value that a `Timestamp` can take. Protocol Buffers timestamps
/// use an `i64` to represent seconds since the Unix epoch, but all timestamps
/// are artificially restricted to occur between `0001-01-01T00:00:00Z` and
/// `9999-12-31T23:59:59.999999999Z`. This value corresponds to the `seconds`
/// field for a timestamp at `10000-01-01T00:00:00`. Since nanoseconds always
/// count forward in Protocol Buffers timestamps, the `seconds` must always be
/// strictly less than this value.
///
/// [The actual .proto file](https://github.com/google/protobuf/blob/master/src/google/protobuf/timestamp.proto)
/// where the `Timestamp` message is defined is a bit vague on whether or not
/// nanoseconds are required to be 0 when the `seconds` field takes on this
/// value. It references both `9999-12-31T23:59:59.999999999Z` and
/// `9999-12-31T23:59:59Z` as inclusive upper bounds. But based on the most
/// implementations and a more detailed reading of the comments there, it seems
/// like nanoseconds can be non-0 when the `seconds` field takes on this value.
/// The only restriction is that the timestamp must lie in the "current era" so
/// that it can be converted to an
/// [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) date string, and it seems
/// clear to me that the former time is the correct upper bound.
const TIMESTAMP_MAXIMUM_SECONDS: i64 = 253402300800;

pub trait AsTimestamp {
    fn as_timestamp(&self) -> Result<Timestamp>;
}

impl AsTimestamp for SystemTime {
    fn as_timestamp(&self) -> Result<Timestamp> {
        // Create a blank timestamp.
        let mut timestamp = Timestamp::new();

        // Compute the duration since the Unix epoch. If the time is before the
        // Unix epoch, the duration_since method will return an error that
        // allows us to compute the negative duration. Because the Timestamp
        // message type uses signed values, we have to do some boundary checks.
        match self.duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                // Compute the seconds component and make sure it is below the
                // maximum value. Since the maximum value is typed as an i64, we
                // can also be sure we won't overflow an i64 when converting.
                let seconds = duration.as_secs();
                if seconds >= (TIMESTAMP_MAXIMUM_SECONDS as u64) {
                    bail!("timestamp too far in the future to represent");
                }

                // Compute the nanoseconds component. It's value should be
                // strictly less than 1,000,000,000, so it will definitely fit
                // into an i32.
                let nanos = duration.subsec_nanos();
                assert!(nanos < 1_000_000_000);

                // Update the timestamp.
                timestamp.seconds = seconds as i64;
                timestamp.nanos = nanos as i32;
            },
            Err(negative) => {
                // Extract the negative duration.
                let duration = negative.duration();

                // Extract the seconds component. We need to modify it if there
                // is a non-zero number of nanoseconds, so we leave it mutable
                // and bounds check it post-modification.
                let mut seconds = duration.as_secs();

                // Compute the nanoseconds component. It's value should be
                // strictly less than 1,000,000,000, which will definitely fit
                // into an i32.
                let mut nanos = duration.subsec_nanos();
                assert!(nanos < 1_000_000_000);

                // The Protocol Buffers Timestamp definition states that
                // nanoseconds should always count forward in time, even for
                // timestamps occurring before the Unix epoch, so if there's a
                // non-negative number of nanoseconds, increase the number of
                // seconds (making it more negative once we flip its sign) and
                // invert the number of nanoseconds.
                if nanos > 0 {
                    seconds += 1;
                    nanos = 1_000_000_000 - nanos;
                }

                // Verify that the number of seconds is within the allowed
                // timestamp range.
                // HACK: We're assuming here that TIMESTAMP_MINIMUM_SECONDS is
                // strictly greater than the minimum value that an i64 can take,
                // because the minimum i64 value is greater in magnitude than
                // the maximum i64 value, and thus this sign flip would cause an
                // overflow if our assumption failed.
                if seconds > ((-TIMESTAMP_MINIMUM_SECONDS) as u64) {
                    bail!("timestamp too far in the past to represent");
                }

                // Update the timestamp.
                // HACK: We're also relying on our assumption about the
                // magnitude of TIMESTAMP_MINIMUM_SECONDS here when converting
                // seconds to an i64 before the sign flip.
                timestamp.seconds = -(seconds as i64);
                timestamp.nanos = nanos as i32;
            },
        }

        // Success.
        Ok(timestamp)
    }
}

/// Provides an ordering for timestamps.
#[derive(PartialEq)]
pub enum Order {
    Ascending,
    Equal,
    Descending,
}

impl Order {
    /// Computes a timestamp ordering. Ideally we'd implement `Eq` and `Ord` on
    /// the timestamp message directly, but this isn't possible due to the use
    /// of generated code.
    pub fn compute(first: &Timestamp, second: &Timestamp) -> Order {
        // Check if one component is lower than the other. This strategy relies
        // on two facts: nanoseconds are always positive and count forward and
        // they are restricted to [0, 999,999,999]. Without these restrictions,
        // we'd have to perform some sort of normalization pass on the
        // timestamps first.
        if first.seconds < second.seconds {
            return Order::Ascending;
        } else if second.seconds < first.seconds {
            return Order::Descending;
        } else if first.nanos < second.nanos {
            return Order::Ascending;
        } else if second.nanos < first.nanos {
            return Order::Descending;
        }

        // Both components must be equal.
        Order::Equal
    }
}
