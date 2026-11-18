//! The Postgres adapter: the connection pool, the schema migrations, and the reachability check.

use std::time::Duration;

use sqlx::PgPool;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::PgPoolOptions;

use crate::config::DatabaseUrl;

/// One Cloud Run instance on Neon's free tier needs no more (spec rule 2.5.4).
pub const MAX_CONNECTIONS: u32 = 5;

/// How long a request waits for a connection before it fails.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

static MIGRATOR: Migrator = sqlx::migrate!();

/// Opens the connection pool. Fails if the database cannot be reached, so the server never starts without it.
pub async fn connect(database_url: &DatabaseUrl) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect(database_url.expose())
        .await
}

/// Applies every pending migration, each exactly once (spec rule 2.4.6).
/// Migrations are forward-only; afterwards the schema is the one this build was compiled against.
#[tracing::instrument(name = "postgres.migrate", skip_all, fields(otel.kind = "client", db.system.name = "postgresql"))]
pub async fn migrate(database: &PgPool) -> Result<(), MigrateError> {
    MIGRATOR.run(database).await
}

/// Whether a query currently makes the round trip to the database.
#[tracing::instrument(name = "postgres.ping", skip_all, fields(otel.kind = "client", db.system.name = "postgresql"))]
pub async fn is_reachable(database: &PgPool) -> bool {
    sqlx::query("SELECT 1").execute(database).await.is_ok()
}
