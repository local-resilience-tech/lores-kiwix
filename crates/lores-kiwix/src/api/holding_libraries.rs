use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::Response,
};

use crate::{api::ApiState, proxy::proxy_error};

pub async fn handler(State(state): State<ApiState>, Path(book_id): Path<String>) -> Response {
    let holdings: Vec<Holding> = match fetch_holdings(&state, &book_id).await {
        Ok(h) => h,
        Err(e) => {
            return proxy_error(StatusCode::INTERNAL_SERVER_ERROR, &e);
        }
    };

    let xml = render_holdings_feed(&holdings);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .body(Body::from(xml))
        .unwrap()
}

pub struct Holding {
    pub title: String,
}

async fn fetch_holdings(_state: &ApiState, _book_id: &str) -> Result<Vec<Holding>, String> {
    // TODO: query holdings from the database by book_id
    Ok(vec![])
}

fn render_holdings_feed(holdings: &[Holding]) -> String {
    let entries: String = holdings
        .iter()
        .map(|h| format!("<entry><title>{}</title></entry>", xml_escape(&h.title)))
        .collect();

    format!(r#"<?xml version="1.0" encoding="UTF-8"?><feed xmlns="http://www.w3.org/2005/Atom">{entries}</feed>"#)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
