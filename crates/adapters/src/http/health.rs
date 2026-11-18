use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

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
