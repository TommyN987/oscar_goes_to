//! The security headers every response carries (spec rule 6.5.1).

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::http::header::{CONTENT_SECURITY_POLICY, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS};
use axum::middleware::Next;
use axum::response::Response;

/// No page may frame the app. A `<meta>` policy cannot say this, so it travels as a header;
/// the rest of the policy is in the SPA shell, where SvelteKit writes it with the hash of its inline script
/// (`csp` in `frontend/vite.config.ts`).
const NO_FRAMING: HeaderValue = HeaderValue::from_static("frame-ancestors 'none'");
/// Browsers use HTTPS for this host for a year. Subdomains are left alone: the domain may serve other things.
const HTTPS_ONLY_FOR_A_YEAR: HeaderValue = HeaderValue::from_static("max-age=31536000");
const NO_CONTENT_TYPE_GUESSING: HeaderValue = HeaderValue::from_static("nosniff");

/// Adds the headers to whatever the rest of the router answered.
pub async fn set_security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(CONTENT_SECURITY_POLICY, NO_FRAMING);
    headers.insert(STRICT_TRANSPORT_SECURITY, HTTPS_ONLY_FOR_A_YEAR);
    headers.insert(X_CONTENT_TYPE_OPTIONS, NO_CONTENT_TYPE_GUESSING);
    response
}
