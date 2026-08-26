use sqlx::SqlitePool;

use crate::node::LoresKiwixNode;
use crate::projection::nodes;
use lores_app_node::NodeEvent;

pub fn register(node: &LoresKiwixNode, pool: SqlitePool) {
    let mut rx = node.subscribe_node_events();

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(NodeEvent::ServerConnected { node_id, region }) => {
                    tracing::info!(node_id = %node_id, region = ?region, "node event: server connected");

                    if let Err(e) = nodes::set_local_node(&pool, &node_id).await {
                        tracing::error!(error = %e, "Failed to update local node in projection");
                    }
                }
                Ok(NodeEvent::ServerDisconnected) => {
                    tracing::info!("node event: server disconnected");
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "Node event handler lagged");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}
