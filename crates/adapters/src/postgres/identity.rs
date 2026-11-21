//! Players and their sessions in Postgres.

use std::error::Error;

use application::session_token::SessionTokenHash;
use application::storage::{PlayerStore, SessionStore, StorageError};
use domain::player::{DisplayName, Player, PlayerId, PlayerName, VerifiedIdentity};
use domain::time::UtcInstant;
use sqlx::types::time::OffsetDateTime;

use super::PostgresStorage;
use crate::telemetry;

/// Turns a failure into the port's error. It logs the failure, once, and marks the span of the storage call
/// as failed, so a trace shows which statement it was. Call it inside that span.
fn storage_error(cause: impl Into<Box<dyn Error + Send + Sync>>) -> StorageError {
    let error = StorageError::from(cause.into());
    tracing::error!(error = &error as &dyn Error, "storage failed");
    telemetry::mark_current_span_failed(&error.to_string());
    error
}

/// Postgres keeps microseconds, so every instant the application writes comes back unchanged.
fn timestamp(instant: UtcInstant) -> Result<OffsetDateTime, StorageError> {
    let nanos = i128::from(instant.millis_since_epoch()) * 1_000_000;
    OffsetDateTime::from_unix_timestamp_nanos(nanos).map_err(storage_error)
}

impl PlayerStore for PostgresStorage {
    #[tracing::instrument(
        name = "postgres.player.add_or_refresh",
        skip_all,
        fields(otel.kind = "client", db.system.name = "postgresql")
    )]
    async fn add_or_refresh_player(
        &self,
        identity: &VerifiedIdentity,
        now: UtcInstant,
    ) -> Result<PlayerId, StorageError> {
        sqlx::query_scalar!(
            "INSERT INTO players (google_subject, email, suggested_name, created_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT ON CONSTRAINT players_google_subject_unique DO UPDATE SET email = EXCLUDED.email
             RETURNING id",
            identity.subject(),
            identity.email(),
            identity.name(),
            timestamp(now)?,
        )
        .fetch_one(&self.database)
        .await
        .map(PlayerId::new)
        .map_err(storage_error)
    }
}

impl SessionStore for PostgresStorage {
    #[tracing::instrument(
        name = "postgres.session.start",
        skip_all,
        fields(otel.kind = "client", db.system.name = "postgresql")
    )]
    async fn start_session(
        &self,
        token_hash: &SessionTokenHash,
        player: PlayerId,
        now: UtcInstant,
    ) -> Result<(), StorageError> {
        sqlx::query!(
            "INSERT INTO sessions (token_hash, player_id, last_used_at) VALUES ($1, $2, $3)",
            token_hash.as_bytes().as_slice(),
            player.value(),
            timestamp(now)?,
        )
        .execute(&self.database)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    #[tracing::instrument(
        name = "postgres.session.use",
        skip_all,
        fields(otel.kind = "client", db.system.name = "postgresql")
    )]
    async fn use_session(
        &self,
        token_hash: &SessionTokenHash,
        now: UtcInstant,
        live_if_used_after: UtcInstant,
    ) -> Result<Option<Player>, StorageError> {
        let Some(row) = sqlx::query!(
            "UPDATE sessions SET last_used_at = $2
             FROM players
             WHERE sessions.token_hash = $1 AND sessions.last_used_at > $3 AND players.id = sessions.player_id
             RETURNING players.id, players.email, players.display_name, players.suggested_name",
            token_hash.as_bytes().as_slice(),
            timestamp(now)?,
            timestamp(live_if_used_after)?,
        )
        .fetch_optional(&self.database)
        .await
        .map_err(storage_error)?
        else {
            return Ok(None);
        };

        let name = match row.display_name {
            Some(chosen) => PlayerName::Chosen(DisplayName::parse(&chosen).map_err(storage_error)?),
            None => PlayerName::NotChosen {
                suggested: row.suggested_name,
            },
        };
        Ok(Some(Player {
            id: PlayerId::new(row.id),
            email: row.email,
            name,
        }))
    }

    #[tracing::instrument(
        name = "postgres.session.end",
        skip_all,
        fields(otel.kind = "client", db.system.name = "postgresql")
    )]
    async fn end_session(&self, token_hash: &SessionTokenHash) -> Result<(), StorageError> {
        sqlx::query!(
            "DELETE FROM sessions WHERE token_hash = $1",
            token_hash.as_bytes().as_slice()
        )
        .execute(&self.database)
        .await
        .map_err(storage_error)?;
        Ok(())
    }

    #[tracing::instrument(
        name = "postgres.session.end_all_of_player",
        skip_all,
        fields(otel.kind = "client", db.system.name = "postgresql")
    )]
    async fn end_sessions_of(&self, player: PlayerId) -> Result<(), StorageError> {
        sqlx::query!("DELETE FROM sessions WHERE player_id = $1", player.value())
            .execute(&self.database)
            .await
            .map_err(storage_error)?;
        Ok(())
    }
}
