//! Static file serving handler.

use super::HandlerResponse;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use std::path::Path;

/// Default static files directory (inside container).
const STATIC_DIR: &str = "/static";

/// Serve a static file from the static directory.
/// Falls back to index.html for SPA routing if file not found.
pub fn handle(path: &str) -> HandlerResponse {
    // Security: prevent directory traversal
    let clean_path = path.trim_start_matches('/');
    if clean_path.contains("..") {
        return (
            Response::builder().status(StatusCode::FORBIDDEN),
            Bytes::from("Forbidden"),
        );
    }

    // Check if static directory exists
    let static_dir = Path::new(STATIC_DIR);
    if !static_dir.exists() {
        return (
            Response::builder().status(StatusCode::NOT_FOUND),
            Bytes::from("Static files not available"),
        );
    }

    // Build full path
    let file_path = if clean_path.is_empty() {
        static_dir.join("index.html")
    } else {
        static_dir.join(clean_path)
    };

    // Try to read the file
    match std::fs::read(&file_path) {
        Ok(contents) => {
            let content_type = mime_type(&file_path);
            (
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", content_type)
                    .header("Cache-Control", "public, max-age=3600"),
                Bytes::from(contents),
            )
        }
        Err(_) => {
            // SPA fallback: try index.html for non-file paths
            if !clean_path.contains('.') {
                let index_path = static_dir.join("index.html");
                if let Ok(contents) = std::fs::read(&index_path) {
                    return (
                        Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "text/html; charset=utf-8"),
                        Bytes::from(contents),
                    );
                }
            }
            (
                Response::builder().status(StatusCode::NOT_FOUND),
                Bytes::from("Not Found"),
            )
        }
    }
}

/// Determine MIME type from file extension.
fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("webp") => "image/webp",
        Some("map") => "application/json",
        _ => "application/octet-stream",
    }
}

/// Returns MIME type for testing.
#[cfg(test)]
pub(crate) fn get_mime_type(path: &Path) -> &'static str {
    mime_type(path)
}
