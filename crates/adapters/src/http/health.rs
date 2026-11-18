use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

use super::error::{ApiError, ErrorCode};
use crate::postgres;

#[derive(Serialize, ToSchema)]
pub struct Liveness {
    #[schema(example = "ok")]
    status: &'static str,
}

/// Liveness
///
/// The process is up. Answers without touching the database (spec rule 2.4.5).
#[utoipa::path(
    get,
    path = "/health",
    operation_id = "liveness",
    tag = "health",
    responses((status = OK, description = "The process is up.", body = Liveness))
)]
pub async fn liveness() -> Json<Liveness> {
    Json(Liveness { status: "ok" })
}

/// Readiness
///
/// The process can reach the database, so it can serve every request (spec rule 2.4.5).
#[utoipa::path(
    get,
    path = "/health/ready",
    operation_id = "readiness",
    tag = "health",
    responses(
        (status = NO_CONTENT, description = "The database is reachable."),
        (status = SERVICE_UNAVAILABLE, description = "The database is not reachable.", body = ApiError),
    )
)]
pub async fn readiness(State(database): State<PgPool>) -> Result<StatusCode, ApiError> {
    if postgres::is_reachable(&database).await {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::new(
            ErrorCode::DatabaseUnavailable,
            "The database is not reachable.",
        ))
    }
}
