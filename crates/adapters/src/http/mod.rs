//! The HTTP driver: the JSON API under `/api`, and the built SPA everywhere else.

mod body_limit;
mod error;
mod health;
mod request_telemetry;
mod security_headers;
mod spa;

use axum::Router;
use axum::middleware;
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub use spa::Spa;

/// The largest request body the server accepts, in bytes (spec rule 6.5.2).
/// Every body the API takes is a few fields of JSON.
pub const MAX_REQUEST_BODY_BYTES: usize = 16 * 1024;

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
/// A request with an oversize body is refused (spec rule 6.5.2), every request is recorded through
/// `telemetry`, and every response carries the security headers (spec rule 6.5.1).
pub fn router(spa: Option<Spa>, database: PgPool, telemetry: Telemetry) -> Router {
    let (api, _) = api().split_for_parts();
    let api = api.with_state(database);
    let routes = match spa {
        Some(spa) => api.merge(spa.into_router()),
        None => api,
    };
    // The last layer is the outermost: it sees the request first and the response last.
    routes
        .layer(middleware::from_fn(body_limit::refuse_oversize_body))
        .layer(middleware::from_fn_with_state(
            telemetry,
            request_telemetry::record_request,
        ))
        .layer(middleware::from_fn(security_headers::set_security_headers))
}
