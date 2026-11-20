//! The storage ports, one per aggregate.

use std::error::Error;
use std::future::Future;

use domain::player::{Player, PlayerId, VerifiedIdentity};
use domain::time::UtcInstant;

use crate::session_token::SessionTokenHash;

/// Storage could not answer. The cause is for the log; no use case can act on it.
/// The message names the cause, because a log line prints an error's message and not its source.
#[derive(Debug, thiserror::Error)]
#[error("storage failed: {0}")]
pub struct StorageError(#[from] Box<dyn Error + Send + Sync>);

pub trait PlayerStore: Sync {
    /// Finds the player keyed by the identity's subject, or adds them without a display name, at `now`.
    ///
    /// Afterwards the stored email is the identity's. A display name already chosen is left alone,
    /// and the provider's name is kept as a suggestion only when the player is added.
    fn record_sign_in(
        &self,
        identity: &VerifiedIdentity,
        now: UtcInstant,
    ) -> impl Future<Output = Result<PlayerId, StorageError>> + Send;
}

pub trait SessionStore: Sync {
    /// Stores a session of `player`, last used at `now`.
    fn start_session(
        &self,
        token_hash: &SessionTokenHash,
        player: PlayerId,
        now: UtcInstant,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    /// Marks the session as used at `now` and returns its player,
    /// if the session exists and was last used after `live_if_used_after`.
    fn use_session(
        &self,
        token_hash: &SessionTokenHash,
        now: UtcInstant,
        live_if_used_after: UtcInstant,
    ) -> impl Future<Output = Result<Option<Player>, StorageError>> + Send;

    /// Ends one session. Ending a session that does not exist is not an error.
    fn end_session(&self, token_hash: &SessionTokenHash) -> impl Future<Output = Result<(), StorageError>> + Send;

    /// Ends every session of `player`.
    fn end_sessions_of(&self, player: PlayerId) -> impl Future<Output = Result<(), StorageError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The log prints an error's message, not its source chain, so the message carries the cause.
    #[test]
    fn a_storage_error_says_what_caused_it() {
        let error = StorageError::from(Box::<dyn Error + Send + Sync>::from("the pool timed out"));

        assert_eq!(error.to_string(), "storage failed: the pool timed out");
    }
}
