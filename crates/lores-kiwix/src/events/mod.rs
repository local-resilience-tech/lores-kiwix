use crate::node::LoresKiwixNode;
use sqlx::SqlitePool;

mod node_events;
mod operation_events;

pub fn register_event_handlers(node: &LoresKiwixNode, pool: SqlitePool) {
    node_events::register(node, pool.clone());
    operation_events::register(node, pool);
}
