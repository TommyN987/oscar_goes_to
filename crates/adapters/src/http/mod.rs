//! The HTTP driver: the JSON API under `/api`, and the built SPA everywhere else.

mod error;
mod health;
mod request_telemetry;
mod spa;

use axum::Router;
use axum::middleware;
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub use spa::Spa;

use crate::telemetry::Telemetry;

#[derive(OpenApi)]
#[openapi(info(
    title = "The Gold Ballot API",
    description = "The JSON API of The Gold Ballot, an Oscar prediction game."
))]
struct ApiDoc;

/// The API operations, each registered once for both routing and the OpenAPI document.
fn api() -> OpenApiRouter<PgPool> {
    let operations = OpenApiRouter::new()
        .routes(routes!(health::liveness))
        .routes(routes!(health::readiness))
        .fallback(error::unknown_operation);
    OpenApiRouter::with_openapi(ApiDoc::openapi()).nest("/api", operations)
}

/// The OpenAPI document that `openapi.json` is generated from.
pub fn openapi() -> utoipa::openapi::OpenApi {
    api().into_openapi()
}

/// The application's router.
///
/// Unknown `/api/*` paths answer with a JSON 404.
/// With a `spa`, every other path serves a file from it, or its shell (spec rule 2.4.2).
/// Every request is recorded through `telemetry`.
pub fn router(spa: Option<Spa>, database: PgPool, telemetry: Telemetry) -> Router {
    let (api, _) = api().split_for_parts();
    let api = api.with_state(database);
    let routes = match spa {
        Some(spa) => api.merge(spa.into_router()),
        None => api,
    };
    routes.layer(middleware::from_fn_with_state(
        telemetry,
        request_telemetry::record_request,
    ))
}
