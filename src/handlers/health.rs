//! Health check handler.

use super::HandlerResponse;
use hyper::body::Bytes;
use hyper::Response;

/// GET /health - Health check endpoint.
pub fn handle() -> HandlerResponse {
    (
        Response::builder().header("Content-Type", "text/plain"),
        Bytes::from("ok"),
    )
}
