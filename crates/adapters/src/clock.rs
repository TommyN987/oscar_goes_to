//! The clock adapter: the machine's own time.

use std::time::{SystemTime, UNIX_EPOCH};

use application::clock::Clock;
use domain::time::UtcInstant;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> UtcInstant {
        UtcInstant::from_millis_since_epoch(millis_since_epoch(SystemTime::now()))
    }
}

/// The milliseconds from 1970 to `time`, negative for a time before it.
/// A time too far away to count reads as the farthest one that can be counted.
fn millis_since_epoch(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_millis()).unwrap_or(i64::MAX),
        Err(before) => i64::try_from(before.duration().as_millis()).map_or(i64::MIN, |millis| -millis),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn an_instant_counts_milliseconds_from_1970() {
        assert_eq!(millis_since_epoch(UNIX_EPOCH), 0);
        assert_eq!(millis_since_epoch(UNIX_EPOCH + Duration::from_millis(1500)), 1500);
    }

    /// A machine whose clock is wrong still gets an instant, and a session then looks as old as it is.
    #[test]
    fn a_clock_set_before_1970_reads_as_a_negative_instant() {
        assert_eq!(millis_since_epoch(UNIX_EPOCH - Duration::from_millis(1500)), -1500);
    }
}
