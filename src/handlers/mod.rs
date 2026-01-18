//! HTTP request handlers.
//!
//! Each handler returns `(http::response::Builder, Bytes)` tuple.
//! The server finalizes the response by adding CORS headers (if enabled)
//! and calling `.body()` + `.unwrap()`.

mod health;
mod metrics;
mod status;

pub use health::handle as health;
pub use metrics::handle_json as metrics_json;
pub use metrics::handle_prometheus as metrics_prometheus;
pub use status::handle as status;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::http::response::Builder;
use hyper::Response;
use hyper::StatusCode;

/// Handler response type - builder + body bytes.
pub type HandlerResponse = (Builder, Bytes);

/// Finalize response: add CORS headers if enabled, build response.
pub fn finalize(response: HandlerResponse, cors_enabled: bool) -> Response<Full<Bytes>> {
    let (builder, body) = response;

    let builder = if cors_enabled {
        builder.header("Access-Control-Allow-Origin", "*")
    } else {
        builder
    };

    builder.body(Full::new(body)).unwrap()
}

/// 404 Not Found handler.
pub fn not_found() -> HandlerResponse {
    (
        Response::builder().status(StatusCode::NOT_FOUND),
        Bytes::from("Not Found"),
    )
}
