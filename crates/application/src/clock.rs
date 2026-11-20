//! The clock port: use cases ask it for the time, so a test can set the time.

use domain::time::UtcInstant;

pub trait Clock: Send + Sync {
    fn now(&self) -> UtcInstant;
}
