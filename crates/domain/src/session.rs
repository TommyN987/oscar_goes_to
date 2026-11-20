//! Sessions: how long one lasts.

use crate::time::UtcInstant;

/// A session ends this long after its last use (spec rule 3.2.4).
pub const LIFETIME_MILLIS: i64 = 90 * 24 * 60 * 60 * 1000;

/// The instant a session must have been last used after, to be live at `now`.
///
/// A session used exactly at this instant has ended. Storage compares against it,
/// so the 90 days are stated once, here.
pub fn live_if_used_after(now: UtcInstant) -> UtcInstant {
    now.earlier_by_millis(LIFETIME_MILLIS)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY_MILLIS: i64 = 24 * 60 * 60 * 1000;

    #[test]
    fn rule_3_2_4_a_session_must_have_been_used_in_the_last_90_days() {
        let day_1090 = UtcInstant::from_millis_since_epoch(1090 * DAY_MILLIS);
        let day_1000 = UtcInstant::from_millis_since_epoch(1000 * DAY_MILLIS);

        assert_eq!(live_if_used_after(day_1090), day_1000);
    }
}
