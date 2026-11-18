//! Records every request as a span and as a duration measurement (spec rules 2.7.2 and 2.7.4).

use std::time::Instant;

use axum::extract::{MatchedPath, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use tracing::Instrument;
use tracing::field::Empty;

use crate::telemetry::Telemetry;

/// The route label of a request that matched no API operation: an SPA page, a static file, or an unknown path.
const UNMATCHED_ROUTE: &str = "unmatched";

/// Wraps the handler in a server span and records the request's duration.
///
/// Telemetry carries the route template only, never the raw path or the query string,
/// so no personal data can reach it from a URL (spec rule 2.7.6).
pub async fn record_request(State(telemetry): State<Telemetry>, request: Request, next: Next) -> Response {
    let method = request.method().as_str().to_owned();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or(UNMATCHED_ROUTE, MatchedPath::as_str)
        .to_owned();

    let span = tracing::info_span!(
        "http_request",
        otel.name = format!("{method} {route}"),
        otel.kind = "server",
        http.request.method = method,
        http.route = route,
        http.response.status_code = Empty,
        otel.status_code = Empty,
    );
    let started = Instant::now();
    let response = next.run(request).instrument(span.clone()).await;

    let status = response.status();
    span.record("http.response.status_code", status.as_u16());
    if status.is_server_error() {
        span.record("otel.status_code", "ERROR");
    }
    drop(span);

    telemetry.record_request(&method, &route, status.as_u16(), started.elapsed());
    if telemetry.flushes_each_request() {
        telemetry.flush().await;
    }
    response
}
