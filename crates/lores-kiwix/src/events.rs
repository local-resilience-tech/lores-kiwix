use lores_kiwix_node::{LoresKiwixNode, operations::AppOperation};

pub fn register_event_handlers(node: &LoresKiwixNode) {
    let mut rx = node.subscribe();

    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(AppOperation::ZimRegisteredV1(data)) => {
                    println!("ZimRegisteredV1 event received: {:?}", data);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    println!("Event handler lagged, skipped {n} events");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}
