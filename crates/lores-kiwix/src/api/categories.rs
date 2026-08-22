use axum::{
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::Response,
};

use crate::api::ApiState;
use crate::projection::zims;
use crate::proxy::proxy_error;
use crate::xml::categories::{build_category, build_feed_root};
use crate::xml::render_xml;

pub async fn handler(State(state): State<ApiState>) -> Response {
    let library_categories = {
        let library = state.library.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        libkiwix_rust::library_get_books_categories(&library)
    };

    let projection_categories = match zims::list_categories(&state.pool).await {
        Ok(cats) => cats,
        Err(err) => {
            tracing::error!(error = %err, "failed to query projection categories");
            return proxy_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to query projection categories",
            );
        }
    };

    let mut categories: Vec<String> = library_categories.into_iter().chain(projection_categories).collect();
    categories.sort_unstable();
    categories.dedup();

    match render_categories(&categories) {
        Ok(buf) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")
            .body(Body::from(buf))
            .unwrap(),
        Err(err) => {
            tracing::error!(error = %err, "failed to serialize categories feed");
            proxy_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to serialize categories feed")
        }
    }
}

fn render_categories(categories: &[String]) -> Result<Vec<u8>, elementtree::Error> {
    let now = chrono::Utc::now();
    let mut feed = build_feed_root(now);
    for category in categories {
        let entry = build_category(category, now, &feed);
        feed.append_child(entry);
    }

    render_xml(&feed)
}
