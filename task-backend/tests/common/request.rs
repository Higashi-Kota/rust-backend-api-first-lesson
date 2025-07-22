// tests/common/request.rs
use axum::{
    body::Body,
    http::{header, Method, Request},
};
use serde::Serialize;

/// 認証付きのHTTPリクエストを作成
pub fn create_request<T: Serialize>(
    method: &str,
    uri: &str,
    token: &str,
    body: &T,
) -> Request<Body> {
    let method = Method::from_bytes(method.as_bytes()).unwrap();
    let body_json = serde_json::to_string(body).unwrap();

    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(body_json))
        .unwrap()
}
