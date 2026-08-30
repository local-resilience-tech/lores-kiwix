use std::env;

use libkiwix_rust as kiwix;
use lores_kiwix::{BootConfig, boot};

const PANDA_GRPC_ADDR_ENV: &str = "PANDA_GRPC_ADDR";
const PANDA_GRPC_ADDR_DEFAULT: &str = "http://127.0.0.1:50051";

const APP_ID_ENV: &str = "LORES_APP_ID";
const APP_ID_DEFAULT: &str = "lores-websites";

const INSTANCE_ID_ENV: &str = "LORES_INSTANCE_ID";
const INSTANCE_ID_DEFAULT: &str = "default";

const DATA_DIR_ENV: &str = "DATA_DIR";
const DATA_DIR_DEFAULT: &str = "./data";

const KIWIX_INTERNAL_BIND_ENV: &str = "KIWIX_INTERNAL_BIND";
const KIWIX_INTERNAL_BIND_DEFAULT: &str = "127.0.0.1:18080";

fn usage(program: &str) {
    eprintln!("Usage: {} <zim-file-or-dir> [address:port]", program);
    eprintln!("  address:port defaults to 0.0.0.0:8080");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    match kiwix::verify_linked_version() {
        Ok(version) => tracing::info!(libkiwix = version, "libkiwix version verified"),
        Err((runtime, built_against)) => tracing::warn!(
            runtime,
            built_against,
            "libkiwix runtime version differs from the version this build was compiled against"
        ),
    }

    let args: Vec<String> = env::args().collect();
    let program = args.first().map(|s| s.as_str()).unwrap_or("lores-kiwix");

    if args.len() < 2 {
        usage(program);
        std::process::exit(1);
    }

    let config = BootConfig {
        path: args[1].clone(),
        panda_grpc_addr: env::var(PANDA_GRPC_ADDR_ENV).unwrap_or_else(|_| PANDA_GRPC_ADDR_DEFAULT.to_string()),
        app_id: env::var(APP_ID_ENV).unwrap_or_else(|_| APP_ID_DEFAULT.to_string()),
        instance_id: env::var(INSTANCE_ID_ENV).unwrap_or_else(|_| INSTANCE_ID_DEFAULT.to_string()),
        data_dir: env::var(DATA_DIR_ENV).unwrap_or_else(|_| DATA_DIR_DEFAULT.to_string()),
        internal_bind: env::var(KIWIX_INTERNAL_BIND_ENV).unwrap_or_else(|_| KIWIX_INTERNAL_BIND_DEFAULT.to_string()),
        projection_db: lores_kiwix::ProjectionDbConfig::InMemory,
    };

    let result = boot(&config).await.expect("failed to boot lores-kiwix");

    let public_bind = args
        .get(2)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "0.0.0.0:8080".to_string());

    let app = lores_kiwix::proxy::app(
        &result.upstream,
        result.projection_pool,
        result.shared_library,
        result.node,
    );
    let listener = tokio::net::TcpListener::bind(&public_bind)
        .await
        .expect("failed to bind public proxy port");

    println!("Starting lores-kiwix proxy on http://{}", public_bind);
    println!("Press Ctrl+C to stop.");

    axum::serve(listener, app).await.expect("proxy server failed");
}
