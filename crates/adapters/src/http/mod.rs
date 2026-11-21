//! The HTTP driver: the JSON API under `/api`, and the built SPA everywhere else.

mod body_limit;
mod error;
#[cfg(feature = "fake-identity")]
mod fake_identity;
mod health;
mod request_telemetry;
mod security_headers;
mod session;
mod spa;

use std::sync::Arc;

use application::admin_emails::AdminEmails;
use application::clock::Clock;
use axum::Router;
use axum::extract::FromRef;
use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use axum::middleware;
use sqlx::PgPool;
use tower_http::set_header::SetResponseHeaderLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub use spa::Spa;

/// The largest request body the server accepts, in bytes (spec rule 6.5.2).
/// Every body the API takes is a few fields of JSON.
pub const MAX_REQUEST_BODY_BYTES: usize = 16 * 1024;

use crate::postgres::PostgresStorage;
use crate::telemetry::Telemetry;

/// An API answer may depend on the session cookie, or set it, so no cache between server and browser keeps one.
const NEVER_STORE: HeaderValue = HeaderValue::from_static("no-store");

/// What the API operations work with: storage, the clock, and who the configuration names an admin.
#[derive(Clone)]
pub struct ApiState {
    database: PgPool,
    storage: PostgresStorage,
    clock: Arc<dyn Clock>,
    admin_emails: Arc<AdminEmails>,
}

impl ApiState {
    pub fn new(database: PgPool, clock: Arc<dyn Clock>, admin_emails: AdminEmails) -> Self {
        Self {
            storage: PostgresStorage::new(database.clone()),
            database,
            clock,
            admin_emails: Arc::new(admin_emails),
        }
    }
}

/// The health operations ask the pool itself whether the database answers.
impl FromRef<ApiState> for PgPool {
    fn from_ref(state: &ApiState) -> Self {
        state.database.clone()
    }
}

#[derive(OpenApi)]
#[openapi(info(
    title = "The Gold Ballot API",
    description = "The JSON API of The Gold Ballot, an Oscar prediction game."
))]
struct ApiDoc;

/// The API operations, each registered once for both routing and the OpenAPI document.
fn api() -> OpenApiRouter<ApiState> {
    let operations = OpenApiRouter::new()
        .routes(routes!(health::liveness))
        .routes(routes!(health::readiness))
        .routes(routes!(session::who_am_i))
        .routes(routes!(session::sign_out))
        .fallback(error::unknown_operation);
    // A plain route, not an operation: it stays out of the OpenAPI document (spec rule 4.6.1).
    #[cfg(feature = "fake-identity")]
    let operations = operations.route(fake_identity::PATH, axum::routing::post(fake_identity::sign_in));
    OpenApiRouter::with_openapi(ApiDoc::openapi()).nest("/api", operations)
}

/// The OpenAPI document that `openapi.json` is generated from.
pub fn openapi() -> utoipa::openapi::OpenApi {
    api().into_openapi()
}

/// The application's router.
///
/// Unknown `/api/*` paths answer with a JSON 404, and no cache may store any `/api` answer.
/// With a `spa`, every other path serves a file from it, or its shell (spec rule 2.4.2).
/// A request with an oversize body is refused (spec rule 6.5.2), every request is recorded through
/// `telemetry`, and every response carries the security headers (spec rule 6.5.1).
pub fn router(spa: Option<Spa>, state: ApiState, telemetry: Telemetry) -> Router {
    let (api, _) = api().split_for_parts();
    let api = api
        .with_state(state)
        .layer(SetResponseHeaderLayer::overriding(CACHE_CONTROL, NEVER_STORE));
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
