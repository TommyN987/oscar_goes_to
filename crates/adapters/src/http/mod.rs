//! The HTTP driver: the JSON API under `/api`, and the built SPA everywhere else.

mod error;
mod health;
mod spa;

use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub use spa::Spa;

#[derive(OpenApi)]
#[openapi(info(
    title = "The Gold Ballot API",
    description = "The JSON API of The Gold Ballot, an Oscar prediction game."
))]
struct ApiDoc;

/// The API operations, each registered once for both routing and the OpenAPI document.
fn api() -> OpenApiRouter {
    let operations = OpenApiRouter::new()
        .routes(routes!(health::liveness))
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
pub fn router(spa: Option<Spa>) -> Router {
    let (api, _) = api().split_for_parts();
    match spa {
        Some(spa) => api.merge(spa.into_router()),
        None => api,
    }
}
