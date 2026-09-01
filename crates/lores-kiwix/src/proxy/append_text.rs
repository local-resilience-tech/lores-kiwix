use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    response::Response,
};

use super::proxy_error;
use crate::api::ApiState;

/// Build a handler that proxies `upstream_path` and appends the contents of
/// `append_path` to successful responses.
///
/// The original upstream `Content-Type` header is preserved unchanged.
///
/// The query string (e.g. `?cacheid=ae79e41a`) is passed through unchanged so
/// the upstream kiwix server sees the same cache-busting URL the browser sent.
pub fn handler(
    upstream_path: &'static str,
    append_path: &'static str,
) -> impl Fn(State<ApiState>, Request) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone
{
    move |State(state): State<ApiState>, req: Request| {
        let upstream_path = upstream_path;
        let append_path = append_path;
        Box::pin(async move {
            let query = req.uri().query().map(|q| format!("?{q}")).unwrap_or_default();
            let upstream_url = format!("{}{}{}", state.upstream, upstream_path, query);

            let upstream_resp = match state.client.get(&upstream_url).send().await {
                Ok(resp) => resp,
                Err(err) => {
                    tracing::error!(error = %err, url = %upstream_url, "failed to fetch upstream resource");
                    return proxy_error(StatusCode::BAD_GATEWAY, "failed to fetch upstream resource");
                }
            };

            let status = upstream_resp.status();
            let content_type = upstream_resp.headers().get(header::CONTENT_TYPE).cloned();

            let body = match upstream_resp.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => {
                    tracing::error!(error = %err, "failed to read upstream resource body");
                    return proxy_error(StatusCode::BAD_GATEWAY, "failed to read upstream resource body");
                }
            };

            let body = if status.is_success() {
                append_text(&body, append_path).await
            } else {
                body.to_vec()
            };

            let mut builder = Response::builder().status(status);
            if let Some(content_type) = content_type {
                builder = builder.header(header::CONTENT_TYPE, content_type);
            }
            builder.body(Body::from(body)).unwrap()
        })
    }
}

async fn append_text(upstream: &[u8], append_path: &str) -> Vec<u8> {
    let relative_path = append_path.strip_prefix('/').unwrap_or(append_path);
    let file_path = super::static_dir().join(relative_path);

    match tokio::fs::read(&file_path).await {
        Ok(custom) => {
            let mut result = upstream.to_vec();
            result.extend_from_slice(b"\n");
            result.extend_from_slice(&custom);
            result
        }
        Err(err) => {
            tracing::warn!(error = %err, path = %append_path, "failed to read append text file");
            upstream.to_vec()
        }
    }
}
