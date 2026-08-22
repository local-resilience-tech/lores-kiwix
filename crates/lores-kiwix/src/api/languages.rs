use axum::{
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::Response,
};

use crate::api::ApiState;
use crate::proxy::proxy_error;
use crate::xml::languages::{build_feed_root, build_language};
use crate::xml::render_xml;

pub async fn handler(State(state): State<ApiState>) -> Response {
    let languages = {
        let library = state.library.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        libkiwix_rust::library_get_books_languages(&library)
    };

    match render_languages(&languages) {
        Ok(buf) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")
            .body(Body::from(buf))
            .unwrap(),
        Err(err) => {
            tracing::error!(error = %err, "failed to serialize languages feed");
            proxy_error(StatusCode::INTERNAL_SERVER_ERROR, "failed to serialize languages feed")
        }
    }
}

fn render_languages(languages: &[libkiwix_rust::LanguageEntry]) -> Result<Vec<u8>, elementtree::Error> {
    let now = chrono::Utc::now();
    let mut feed = build_feed_root(now);
    for lang in languages {
        let entry = build_language(lang, now, &feed);
        feed.append_child(entry);
    }
    render_xml(&feed)
}
