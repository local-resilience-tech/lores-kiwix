use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::Response,
};

use crate::{
    api::ApiState,
    projection::nodes::{NodeRow, list_nodes_holding_book},
    proxy::proxy_error,
    xml::{
        holding_libraries::{build_feed_root, build_holding_library},
        render_xml,
    },
};

pub async fn handler(State(state): State<ApiState>, Path(book_id): Path<String>) -> Response {
    let nodes: Vec<NodeRow> = match list_nodes_holding_book(&state.pool, &book_id).await {
        Ok(h) => h,
        Err(err) => {
            tracing::error!(error = %err, "failed to query projection categories");
            return proxy_error(StatusCode::INTERNAL_SERVER_ERROR, "Database error");
        }
    };

    match render_nodes_feed(&nodes) {
        Ok(buf) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")
            .body(Body::from(buf))
            .unwrap(),
        Err(err) => {
            tracing::error!(error = %err, "failed to serialize holding libraries feed");
            proxy_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to serialize holding libraries feed",
            )
        }
    }
}

fn render_nodes_feed(nodes: &[NodeRow]) -> Result<Vec<u8>, elementtree::Error> {
    let now = chrono::Utc::now();
    let mut feed = build_feed_root(now);
    for node in nodes {
        let entry = build_holding_library(node, now, &feed);
        feed.append_child(entry);
    }

    render_xml(&feed)
}
