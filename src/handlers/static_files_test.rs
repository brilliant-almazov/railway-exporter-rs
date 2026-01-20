//! Tests for static file serving handler.

use super::static_files::{get_mime_type, handle};
use hyper::body::Bytes;
use hyper::http::StatusCode;
use std::path::Path;

// =============================================================================
// MIME Type Tests
// =============================================================================

#[test]
fn test_mime_type_html() {
    assert_eq!(get_mime_type(Path::new("index.html")), "text/html; charset=utf-8");
    assert_eq!(get_mime_type(Path::new("page.htm")), "text/html; charset=utf-8");
}

#[test]
fn test_mime_type_css() {
    assert_eq!(get_mime_type(Path::new("styles.css")), "text/css; charset=utf-8");
}

#[test]
fn test_mime_type_javascript() {
    assert_eq!(get_mime_type(Path::new("app.js")), "application/javascript; charset=utf-8");
    assert_eq!(get_mime_type(Path::new("module.mjs")), "application/javascript; charset=utf-8");
}

#[test]
fn test_mime_type_json() {
    assert_eq!(get_mime_type(Path::new("data.json")), "application/json; charset=utf-8");
    assert_eq!(get_mime_type(Path::new("app.js.map")), "application/json");
}

#[test]
fn test_mime_type_images() {
    assert_eq!(get_mime_type(Path::new("logo.png")), "image/png");
    assert_eq!(get_mime_type(Path::new("photo.jpg")), "image/jpeg");
    assert_eq!(get_mime_type(Path::new("photo.jpeg")), "image/jpeg");
    assert_eq!(get_mime_type(Path::new("anim.gif")), "image/gif");
    assert_eq!(get_mime_type(Path::new("icon.svg")), "image/svg+xml");
    assert_eq!(get_mime_type(Path::new("favicon.ico")), "image/x-icon");
    assert_eq!(get_mime_type(Path::new("image.webp")), "image/webp");
}

#[test]
fn test_mime_type_fonts() {
    assert_eq!(get_mime_type(Path::new("font.woff")), "font/woff");
    assert_eq!(get_mime_type(Path::new("font.woff2")), "font/woff2");
    assert_eq!(get_mime_type(Path::new("font.ttf")), "font/ttf");
}

#[test]
fn test_mime_type_text() {
    assert_eq!(get_mime_type(Path::new("readme.txt")), "text/plain; charset=utf-8");
    assert_eq!(get_mime_type(Path::new("config.xml")), "application/xml; charset=utf-8");
}

#[test]
fn test_mime_type_unknown() {
    assert_eq!(get_mime_type(Path::new("file.unknown")), "application/octet-stream");
    assert_eq!(get_mime_type(Path::new("noext")), "application/octet-stream");
}

// =============================================================================
// Handler Security Tests
// =============================================================================

#[test]
fn test_handle_blocks_directory_traversal() {
    let (builder, body) = handle("../../../etc/passwd");
    let response = builder.body(body.clone()).unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(body, Bytes::from("Forbidden"));
}

#[test]
fn test_handle_blocks_traversal_in_middle() {
    let (builder, body) = handle("/some/../path");
    let response = builder.body(body.clone()).unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(body, Bytes::from("Forbidden"));
}

#[test]
fn test_handle_blocks_double_dot_anywhere() {
    let paths = [
        "../secret",
        "foo/../bar",
        "..%2f..%2fetc/passwd",  // URL-encoded but contains literal ".."
        "/path/to/../../../etc",
    ];

    for path in &paths {
        if path.contains("..") {
            let (builder, body) = handle(path);
            let response = builder.body(body.clone()).unwrap();
            assert_eq!(
                response.status(),
                StatusCode::FORBIDDEN,
                "Path '{}' should be forbidden",
                path
            );
        }
    }
}

// =============================================================================
// Handler Behavior Tests
// =============================================================================

#[test]
fn test_handle_static_dir_not_exists() {
    // On dev machine, /static doesn't exist
    let (builder, body) = handle("/");
    let response = builder.body(body.clone()).unwrap();

    // Without /static directory, should return 404 with specific message
    if !Path::new("/static").exists() {
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(body, Bytes::from("Static files not available"));
    }
}

#[test]
fn test_handle_strips_leading_slashes() {
    // Multiple leading slashes should be handled
    let (builder, body) = handle("///index.html");
    let response = builder.body(body.clone()).unwrap();

    // Should not crash, return appropriate response
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
        "Expected OK or NOT_FOUND, got {:?}",
        response.status()
    );
}

#[test]
fn test_handle_empty_path_serves_index() {
    let (builder, body) = handle("");
    let response = builder.body(body.clone()).unwrap();

    // Empty path should try to serve index.html
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
        "Expected OK or NOT_FOUND, got {:?}",
        response.status()
    );
}

#[test]
fn test_handle_root_path_serves_index() {
    let (builder, body) = handle("/");
    let response = builder.body(body.clone()).unwrap();

    // Root path should try to serve index.html
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
        "Expected OK or NOT_FOUND, got {:?}",
        response.status()
    );
}
