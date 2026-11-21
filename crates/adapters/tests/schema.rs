//! The schema the migrations build, probed with direct SQL.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails by panicking, in its helper functions too"
)]

use std::fmt::Debug;
use std::fs;

use adapters::postgres;
use sqlx::PgPool;

/// The version of every migration the database records as applied, with the time it was applied.
async fn applied_migrations(database: &PgPool) -> Vec<(i64, String)> {
    sqlx::query_as("SELECT version, installed_on::text FROM _sqlx_migrations WHERE success ORDER BY version")
        .fetch_all(database)
        .await
        .unwrap()
}

/// The versions of the migration files, which are named `<version>_<description>.sql`.
fn migration_file_versions() -> Vec<i64> {
    let mut versions: Vec<i64> = fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/migrations"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|file_name| file_name.ends_with(".sql"))
        .map(|file_name| file_name.split_once('_').unwrap().0.parse().unwrap())
        .collect();
    versions.sort_unstable();
    versions
}

#[sqlx::test(migrations = false)]
async fn rule_2_4_6_each_pending_migration_is_applied_exactly_once(database: PgPool) {
    postgres::migrate(&database).await.unwrap();
    let after_first_run = applied_migrations(&database).await;
    postgres::migrate(&database).await.unwrap();
    let after_second_run = applied_migrations(&database).await;

    let applied_versions: Vec<i64> = after_first_run.iter().map(|(version, _)| *version).collect();
    assert_eq!(applied_versions, migration_file_versions());
    assert_eq!(after_second_run, after_first_run);
}

/// Every column of the schema: `table.column type`, then `not null` where it applies.
async fn columns(database: &PgPool) -> Vec<String> {
    sqlx::query_scalar!(
        r#"
        SELECT table_name || '.' || column_name || ' ' || data_type
            || CASE is_nullable WHEN 'NO' THEN ' not null' ELSE '' END AS "column!"
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name <> '_sqlx_migrations'
        ORDER BY table_name, ordinal_position
        "#
    )
    .fetch_all(database)
    .await
    .unwrap()
}

/// A new column fails this test, so it is weighed against the rule before it lands.
/// No table names a season either: an account belongs to no season and outlives each one (spec rule 1.6.5).
#[sqlx::test]
async fn rule_3_5_1_the_schema_holds_these_columns_and_no_other(database: PgPool) {
    assert_eq!(
        columns(&database).await,
        [
            "blocked_accounts.google_subject text not null",
            "blocked_accounts.email text not null",
            "blocked_accounts.removed_at timestamp with time zone not null",
            "players.id bigint not null",
            "players.google_subject text not null",
            "players.email text not null",
            "players.display_name text",
            "players.display_name_key text",
            "players.suggested_name text",
            "players.created_at timestamp with time zone not null",
            "sessions.token_hash bytea not null",
            "sessions.player_id bigint not null",
            "sessions.last_used_at timestamp with time zone not null",
        ]
    );
}

/// What a test chooses about a row of `players`; the email and the creation time are fixed.
#[derive(Default)]
struct NewPlayer<'a> {
    google_subject: &'a str,
    display_name: Option<&'a str>,
    display_name_key: Option<&'a str>,
    suggested_name: Option<&'a str>,
}

/// Inserts a player and returns their ID.
async fn insert_player(database: &PgPool, player: NewPlayer<'_>) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO players (google_subject, email, display_name, display_name_key, suggested_name, created_at)
         VALUES ($1, 'player@example.com', $2, $3, $4, TIMESTAMPTZ '2027-01-01 00:00:00+00')
         RETURNING id",
    )
    .bind(player.google_subject)
    .bind(player.display_name)
    .bind(player.display_name_key)
    .bind(player.suggested_name)
    .fetch_one(database)
    .await
}

/// Inserts a player who has not chosen a name yet, and returns their ID.
async fn insert_nameless_player(database: &PgPool, google_subject: &str) -> i64 {
    let player = NewPlayer {
        google_subject,
        ..NewPlayer::default()
    };
    insert_player(database, player).await.unwrap()
}

async fn insert_session(database: &PgPool, token_hash: &[u8], player_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO sessions (token_hash, player_id, last_used_at)
         VALUES ($1, $2, TIMESTAMPTZ '2027-01-01 00:00:00+00')",
    )
    .bind(token_hash)
    .bind(player_id)
    .execute(database)
    .await
    .map(|_| ())
}

/// The name of the constraint that refused a statement.
fn violated_constraint<T: Debug>(outcome: Result<T, sqlx::Error>) -> String {
    let error = outcome.expect_err("the statement must be refused");
    let Some(constraint) = error.as_database_error().and_then(|error| error.constraint()) else {
        panic!("the statement must be refused by a constraint, not with: {error}");
    };
    constraint.to_owned()
}

#[sqlx::test]
async fn a_player_has_a_display_name_and_its_key_or_neither(database: PgPool) {
    let name_without_key = NewPlayer {
        google_subject: "subject-1",
        display_name: Some("Pat"),
        ..NewPlayer::default()
    };
    let key_without_name = NewPlayer {
        google_subject: "subject-2",
        display_name_key: Some("pat"),
        ..NewPlayer::default()
    };

    assert_eq!(
        violated_constraint(insert_player(&database, name_without_key).await),
        "players_name_and_key_together"
    );
    assert_eq!(
        violated_constraint(insert_player(&database, key_without_name).await),
        "players_name_and_key_together"
    );
}

/// The storage half of spec rule 1.6.2; plan item P3.5.2 closes the rule.
#[sqlx::test]
async fn two_players_cannot_hold_one_display_name_key(database: PgPool) {
    let first = NewPlayer {
        google_subject: "subject-1",
        display_name: Some("Pat"),
        display_name_key: Some("pat"),
        ..NewPlayer::default()
    };
    let same_name_in_another_case = NewPlayer {
        google_subject: "subject-2",
        display_name: Some("PAT"),
        display_name_key: Some("pat"),
        ..NewPlayer::default()
    };
    insert_player(&database, first).await.unwrap();

    assert_eq!(
        violated_constraint(insert_player(&database, same_name_in_another_case).await),
        "players_display_name_key_unique"
    );
}

#[sqlx::test]
async fn any_number_of_players_can_be_nameless(database: PgPool) {
    let first = insert_nameless_player(&database, "subject-1").await;
    let second = insert_nameless_player(&database, "subject-2").await;

    assert_ne!(first, second);
}

#[sqlx::test]
async fn rule_3_5_1_a_suggested_name_does_not_outlive_the_choice_of_a_display_name(database: PgPool) {
    let named_and_still_suggested = NewPlayer {
        google_subject: "subject-1",
        display_name: Some("Pat"),
        display_name_key: Some("pat"),
        suggested_name: Some("Patricia Player"),
    };

    assert_eq!(
        violated_constraint(insert_player(&database, named_and_still_suggested).await),
        "players_suggestion_only_while_nameless"
    );
}

/// A player is keyed by Google's subject ID (spec rule 1.6.1).
#[sqlx::test]
async fn two_players_cannot_share_a_google_subject(database: PgPool) {
    insert_nameless_player(&database, "subject-1").await;
    let same_subject = NewPlayer {
        google_subject: "subject-1",
        ..NewPlayer::default()
    };

    assert_eq!(
        violated_constraint(insert_player(&database, same_subject).await),
        "players_google_subject_unique"
    );
}

#[sqlx::test]
async fn rule_3_2_1_a_token_hash_is_32_bytes(database: PgPool) {
    let player = insert_nameless_player(&database, "subject-1").await;

    assert_eq!(
        violated_constraint(insert_session(&database, &[7; 31], player).await),
        "sessions_token_hash_is_32_bytes"
    );
    insert_session(&database, &[7; 32], player).await.unwrap();
}

#[sqlx::test]
async fn a_session_belongs_to_a_player(database: PgPool) {
    let no_such_player = 404;

    assert_eq!(
        violated_constraint(insert_session(&database, &[7; 32], no_such_player).await),
        "sessions_player_id_fkey"
    );
}

/// What the schema gives spec rule 3.2.5; plan item P3.8 closes the rule, once an old cookie
/// is refused after deletion and after removal.
#[sqlx::test]
async fn rule_3_2_5_deleting_a_player_deletes_all_their_sessions(database: PgPool) {
    let player = insert_nameless_player(&database, "subject-1").await;
    // One session per device (spec rule 3.2.3).
    insert_session(&database, &[7; 32], player).await.unwrap();
    insert_session(&database, &[8; 32], player).await.unwrap();

    sqlx::query("DELETE FROM players WHERE id = $1")
        .bind(player)
        .execute(&database)
        .await
        .unwrap();

    let sessions_left: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions")
        .fetch_one(&database)
        .await
        .unwrap();
    assert_eq!(sessions_left, 0);
}
