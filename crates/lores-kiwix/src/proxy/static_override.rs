use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    response::Response,
};

use super::proxy_error;
use crate::api::ApiState;

/// Build a handler that returns a local static file instead of proxying the
/// request upstream.
///
/// The upstream path is still accepted so callers can read as:
///
///   "/skin/index.js" => static_override::handler("/skin/index.js", "...")
///
/// Query strings (e.g. `?cacheid=...`) are ignored for content selection; the
/// local file is always served.
pub fn handler(
    local_path: &'static str,
    content_type: &'static str,
) -> impl Fn(State<ApiState>, Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone
{
    move |_state: State<ApiState>, _req: Request| {
        let local_path = local_path;
        let content_type = content_type;
        Box::pin(async move { serve_static_file(local_path, content_type).await })
    }
}

pub async fn serve_static_file(static_path: &str, content_type: &str) -> Response {
    let mut file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/static").to_string();
    file_path.push_str(static_path);

    match tokio::fs::read_to_string(file_path).await {
        Ok(contents) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .body(Body::from(contents))
            .unwrap(),
        Err(err) => {
            tracing::error!(error = %err, path = static_path, "failed to static file");
            proxy_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to read static file")
        }
    }
}
