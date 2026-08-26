use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::Response,
};
use lores_app_node::NodeInfo;

use crate::{
    api::ApiState,
    node::LoresKiwixNode,
    projection::nodes::{NodeRow, list_nodes_holding_book},
    proxy::proxy_error,
    xml::{
        holding_libraries::{build_feed_root, build_holding_library},
        render_xml,
    },
};

pub struct NodeWithInfo {
    pub node: NodeRow,
    pub info: Option<NodeInfo>,
}

pub async fn handler(State(state): State<ApiState>, Path(book_id): Path<String>) -> Response {
    let nodes: Vec<NodeRow> = match list_nodes_holding_book(&state.pool, &book_id).await {
        Ok(h) => h,
        Err(err) => {
            tracing::error!(error = %err, "failed to query projection categories");
            return proxy_error(StatusCode::INTERNAL_SERVER_ERROR, "Database error");
        }
    };

    tracing::debug!(book_id = %book_id, count = nodes.len(), "nodes holding book");
    let nodes_with_info = add_node_info(&state.node, nodes).await;

    match render_nodes_feed(&nodes_with_info) {
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

async fn add_node_info(node: &LoresKiwixNode, nodes: Vec<NodeRow>) -> Vec<NodeWithInfo> {
    tracing::info!(count = nodes.len(), "fetching node info");
    let mut result = Vec::with_capacity(nodes.len());
    for row in nodes {
        tracing::info!(node_id = %row.id, "fetching info for node");
        let info = match node.node_info(&row.id).await {
            Ok(info) => {
                tracing::info!(node_id = %row.id, info = ?info, "successfully fetched node info");
                Some(info)
            }
            Err(err) => {
                tracing::warn!(node_id = %row.id, error = %err, "failed to fetch node info");
                None
            }
        };
        result.push(NodeWithInfo { node: row, info });
    }
    result
}

fn render_nodes_feed(nodes: &[NodeWithInfo]) -> Result<Vec<u8>, elementtree::Error> {
    let now = chrono::Utc::now();
    let mut feed = build_feed_root(now);
    for n in nodes {
        let entry = build_holding_library(&n.node, n.info.as_ref(), now, &feed);
        feed.append_child(entry);
    }

    render_xml(&feed)
}
