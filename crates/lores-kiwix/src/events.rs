use lores_kiwix_node::{LoresKiwixNode, operations::AppOperation};
use sqlx::SqlitePool;

use crate::projection::zims;

pub fn register_event_handlers(node: &LoresKiwixNode, pool: SqlitePool) {
    let mut rx = node.subscribe();

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(node_op) => {
                    println!("processing op: {:?}", node_op);

                    match node_op.op {
                        AppOperation::ZimRegisteredV1(data) => {
                            if let Err(err) = zims::insert_zim(&pool, &data).await {
                                tracing::error!(error = %err, "Failed to insert ZIM into projection");
                            }
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "Event handler lagged");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}
