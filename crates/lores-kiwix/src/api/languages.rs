use axum::{
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::Response,
};
use std::collections::HashMap;

use crate::api::ApiState;
use crate::projection::books;
use crate::proxy::proxy_error;
use crate::xml::languages::{build_feed_root, build_language};
use crate::xml::render_xml;

pub async fn handler(State(state): State<ApiState>) -> Response {
    let library_languages: Vec<libkiwix_rust::LanguageEntry> = {
        let library = state.library.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        libkiwix_rust::library_get_books_languages(&library)
    };

    let projection_lang_counts = match books::list_languages(&state.pool).await {
        Ok(langs) => langs,
        Err(err) => {
            tracing::error!(error = %err, "failed to query projection languages");
            return proxy_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to query projection languages",
            );
        }
    };

    let projection_languages = resolve_language_names(projection_lang_counts);

    let languages = merge_languages(&library_languages, &projection_languages);

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

fn resolve_language_names(counts: Vec<(String, u32)>) -> Vec<libkiwix_rust::LanguageEntry> {
    counts
        .into_iter()
        .map(|(code, count)| libkiwix_rust::LanguageEntry {
            lang_name: libkiwix_rust::language_self_name(&code),
            lang_code: code,
            book_count: count,
        })
        .collect()
}

fn merge_languages(
    library: &[libkiwix_rust::LanguageEntry],
    projection: &[libkiwix_rust::LanguageEntry],
) -> Vec<libkiwix_rust::LanguageEntry> {
    // Library entries take precedence for lang_name; counts are summed.
    let mut map: HashMap<String, (String, u32)> = library
        .iter()
        .map(|e| (e.lang_code.clone(), (e.lang_name.clone(), e.book_count)))
        .collect();
    for e in projection {
        let entry = map.entry(e.lang_code.clone()).or_insert((e.lang_name.clone(), 0));
        entry.1 += e.book_count;
    }
    let mut result: Vec<libkiwix_rust::LanguageEntry> = map
        .into_iter()
        .map(|(code, (name, count))| libkiwix_rust::LanguageEntry {
            lang_code: code,
            lang_name: name,
            book_count: count,
        })
        .collect();
    result.sort_unstable_by(|a, b| a.lang_code.cmp(&b.lang_code));
    result
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
