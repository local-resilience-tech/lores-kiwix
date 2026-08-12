use std::net::SocketAddr;

use tonic::transport::Server;
use tracing::info;

use crate::proto::panda_server::PandaServer;
use crate::service::DevPandaService;

mod proto;
mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = std::env::var("PANDA_DEV_SERVER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:50051".to_string())
        .parse()?;

    info!(%addr, "starting lores-p2panda-dev-server");

    Server::builder()
        .add_service(PandaServer::new(DevPandaService::new()))
        .serve(addr)
        .await?;

    Ok(())
}
