//! Time as the rules need it: instants that can be compared.

/// A point in time, in whole milliseconds since the Unix epoch, UTC.
///
/// The rules only compare instants, so this crate needs no calendar and no time library.
/// Adapters convert to and from their own time types at the edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcInstant {
    millis_since_epoch: i64,
}

impl UtcInstant {
    pub const fn from_millis_since_epoch(millis_since_epoch: i64) -> Self {
        Self { millis_since_epoch }
    }

    pub const fn millis_since_epoch(self) -> i64 {
        self.millis_since_epoch
    }
}
