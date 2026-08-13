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
) -> impl Fn(State<ApiState>, Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone
{
    move |_state: State<ApiState>, _req: Request| {
        let local_path = local_path;
        Box::pin(async move {
            match tokio::fs::read_to_string(local_path).await {
                Ok(contents) => Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
                    .body(Body::from(contents))
                    .unwrap(),
                Err(err) => {
                    tracing::error!(error = %err, path = %local_path, "failed to read static override file");
                    proxy_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to read static override file")
                }
            }
        })
    }
}
