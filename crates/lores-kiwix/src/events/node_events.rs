use crate::node::LoresKiwixNode;
use lores_app_node::NodeEvent;

pub fn register(node: &LoresKiwixNode) {
    let mut rx = node.subscribe_node_events();

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(NodeEvent::ServerConnected { node_id }) => {
                    println!("node event: server connected (node_id={:?})", node_id);
                }
                Ok(NodeEvent::ServerDisconnected) => {
                    println!("node event: server disconnected");
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "Node event handler lagged");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}
