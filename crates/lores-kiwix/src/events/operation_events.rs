use crate::node::LoresKiwixNode;
use crate::node::operations::{AppOperation, BookDeregisteredDataV1, BookRegisteredDataV1};
use lores_app_node::AppNodeOperation;
use sqlx::SqlitePool;

use crate::projection::{books, holdings, nodes};

pub fn register(node: &LoresKiwixNode, pool: SqlitePool) {
    let mut rx = node.subscribe();

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(node_op) => {
                    tracing::info!(op = ?node_op.op, "operation received");

                    match &node_op.op {
                        AppOperation::BookRegisteredV1(data) => {
                            if let Err(err) = project_book_registered(&pool, &node_op, data).await {
                                tracing::error!(error = %err, "Failed to project book registration");
                            }
                        }
                        AppOperation::BookDeregisteredV1(data) => {
                            if let Err(err) = project_book_deregistered(&pool, &node_op, data).await {
                                tracing::error!(error = %err, "Failed to project book deregistration");
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
        nodes::ensure_node(&mut *tx, &node_id).await?;
        holdings::insert_holding(&mut *tx, &data.book_id, &node_id).await?;
    }

    tx.commit().await
}

async fn project_book_deregistered(
    pool: &SqlitePool,
    _node_op: &AppNodeOperation<AppOperation>,
    data: &BookDeregisteredDataV1,
) -> Result<(), sqlx::Error> {
    holdings::delete_local_holding(pool, &data.book_id).await?;
    Ok(())
}
