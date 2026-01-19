//! HTTP request handlers.
//!
//! Each handler returns `(http::response::Builder, Bytes)` tuple.
//! The server finalizes the response by adding CORS headers (if enabled),
//! gzip compression (if configured), and calling `.body()` + `.unwrap()`.

mod health;
mod icons;
mod metrics;
mod status;

#[cfg(test)]
mod tests;

pub use health::handle as health;
pub use icons::handle as icons;
pub use metrics::handle_json as metrics_json;
pub use metrics::handle_prometheus as metrics_prometheus;
pub use status::handle as status;

use crate::config::GzipConfig;
use flate2::write::GzEncoder;
use flate2::Compression;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::http::response::Builder;
use hyper::Response;
use hyper::StatusCode;
use std::io::Write;

/// Handler response type - builder + body bytes.
pub type HandlerResponse = (Builder, Bytes);

/// Finalize response: add CORS headers if enabled, gzip if configured, build response.
pub fn finalize(
    response: HandlerResponse,
    cors_enabled: bool,
    gzip_requested: bool,
    gzip_config: &GzipConfig,
) -> Response<Full<Bytes>> {
    let (mut builder, body) = response;

    // Add CORS header if enabled
    if cors_enabled {
        builder = builder.header("Access-Control-Allow-Origin", "*");
    }

    // Gzip compress if enabled, requested, and body exceeds min_size
    let should_compress =
        gzip_config.enabled && gzip_requested && body.len() > gzip_config.min_size;

    let (final_body, is_gzipped) = if should_compress {
        match gzip_compress(&body, gzip_config.level) {
            Ok(compressed) if compressed.len() < body.len() => (Bytes::from(compressed), true),
            _ => (body, false), // Fallback to uncompressed if compression fails or increases size
        }
    } else {
        (body, false)
    };

    // Add Content-Encoding header if compressed
    if is_gzipped {
        builder = builder.header("Content-Encoding", "gzip");
    }

    builder.body(Full::new(final_body)).unwrap()
}

/// Gzip compress bytes with specified compression level.
fn gzip_compress(data: &[u8], level: u32) -> std::io::Result<Vec<u8>> {
    let compression = Compression::new(level);
    let mut encoder = GzEncoder::new(Vec::new(), compression);
    encoder.write_all(data)?;
    encoder.finish()
}

/// 404 Not Found handler.
pub fn not_found() -> HandlerResponse {
    (
        Response::builder().status(StatusCode::NOT_FOUND),
        Bytes::from("Not Found"),
    )
}
