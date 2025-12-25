use crate::{common::layout, pages};
use axum::{extract::Path, http::StatusCode};
use maud::{Markup, PreEscaped, html};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "content/post/"]
struct PostFile;

fn get_raw_post(id: &str) -> Option<String> {
    let id = id.to_lowercase() + ".md";

    if let Some(content) = PostFile::get(id.as_str()) {
        Some(String::from_utf8_lossy(content.data.as_ref()).to_string())
    } else {
        None
    }
}

pub async fn get_post(Path(id): Path<String>) -> (StatusCode, Markup) {
    let content = if let Some(content) = get_raw_post(&id) {
        content
    } else {
        return (StatusCode::NOT_FOUND, pages::not_found().await.1);
    };
    (StatusCode::OK, layout("Post", PreEscaped(content)))
}
