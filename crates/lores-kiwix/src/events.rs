use crate::node::LoresKiwixNode;
use crate::node::operations::{AppOperation, BookRegisteredDataV1};
use lores_app_node::AppNodeOperation;
use sqlx::SqlitePool;

use crate::projection::{books, holdings};

pub fn register_event_handlers(node: &LoresKiwixNode, pool: SqlitePool) {
    let mut rx = node.subscribe();

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(node_op) => {
                    println!("processing op: {:?}", node_op);

                    match &node_op.op {
                        AppOperation::BookRegisteredV1(data) => {
                            if let Err(err) = project_book_registered(&pool, &node_op, data).await {
                                tracing::error!(error = %err, "Failed to project book registration");
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

/// Insert the book and, when the operation carries a node identity, its holding
/// in a single transaction so the projection can't record one without the other.
async fn project_book_registered(
    pool: &SqlitePool,
    node_op: &AppNodeOperation<AppOperation>,
    data: &BookRegisteredDataV1,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    books::insert_book(&mut *tx, data).await?;

    if let Some(node) = &node_op.author_node_id {
        let node_id = hex::encode(&node.0);
        holdings::insert_holding(&mut *tx, &data.book_id, &node_id).await?;
    }

    tx.commit().await
}
