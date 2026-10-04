use axum::{
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::Response,
};

use crate::api::ApiState;

pub async fn handler(State(state): State<ApiState>) -> Response {
    let error_rx = state.node.subscribe_errors();
    let current_error = error_rx.borrow().clone();

    let settings = serde_json::json!({
        "toolbarEnabled": true,
        "linkBlockingEnabled": false,
        "libraryButtonEnabled": true,
        "currentError": current_error,
    });
    let js = format!("const viewerSettings = {}", settings);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .body(Body::from(js.to_string()))
        .unwrap()
}
