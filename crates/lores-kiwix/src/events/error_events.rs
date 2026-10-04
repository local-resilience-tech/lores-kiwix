use crate::node::LoresKiwixNode;

pub fn register(node: &LoresKiwixNode) {
    let mut rx = node.subscribe_errors();
    tracing::info!("Subscribed to errors");

    tokio::spawn(async move {
        loop {
            // `changed()` marks the current value as seen and waits for the next
            // change. We call `borrow_and_update()` first so that an error already
            // set before this client connected is sent immediately on connect.
            let error = rx.borrow_and_update().clone();

            if let Some(error) = error {
                tracing::warn!("Lores-kiwix got a node error: {error}");
            }

            // Wait for the next change.
            if rx.changed().await.is_err() {
                tracing::warn!("Error channel closed");
                break;
            }
        }
    });
}
