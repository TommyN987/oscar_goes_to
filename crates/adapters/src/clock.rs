//! The clock adapter: the machine's own time.

use std::time::{SystemTime, UNIX_EPOCH};

use application::clock::Clock;
use domain::time::UtcInstant;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> UtcInstant {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is set after 1970");
        let millis =
            i64::try_from(since_epoch.as_millis()).expect("the system clock is set before the year 292 million");
        UtcInstant::from_millis_since_epoch(millis)
    }
}
