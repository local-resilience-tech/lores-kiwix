use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    response::Response,
    routing::any,
};
use axum_reverse_proxy::ReverseProxy;

#[derive(Clone)]
pub struct ProxyState {
    upstream: String,
}

/// Build an Axum application that proxies every request to `upstream`,
/// except for `/catalog/v2/entries` which is handled separately so we can
/// later merge in entries from remote nodes.
pub fn app(upstream: impl Into<String>) -> Router {
    let upstream = upstream.into();

    Router::new()
        .route("/catalog/v2/entries", any(catalog_entries_handler))
        .fallback_service(ReverseProxy::new("/", upstream.as_str()))
        .with_state(ProxyState { upstream })
}

/// Currently this just proxies the catalog endpoint unchanged.
///
/// This is the hook where we will later query the projection database and
/// merge ZIM entries announced by other nodes.
async fn catalog_entries_handler(State(state): State<ProxyState>, req: Request) -> Response {
    // TODO: merge remote ZIMs from the projection database.
    let proxy = ReverseProxy::new("/", state.upstream.as_str());
    proxy.proxy_request(req).await.unwrap_or_else(|_| {
        Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .body("bad gateway".into())
            .unwrap()
    })
}
