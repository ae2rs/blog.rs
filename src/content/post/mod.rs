use crate::{common::layout_with_head, pages};
use axum::{extract::Path, http::StatusCode, response::Html};
use macros::Post;
use maud::html;
use meta::Post;
use std::{collections::HashMap, sync::OnceLock};

mod highlight;
mod meta;
mod render;

#[derive(Post)]
struct Posts;

static POST_PAGES: OnceLock<HashMap<&'static str, String>> = OnceLock::new();

fn render_post_page(post: &Post) -> String {
    let content = html! {
        h1 class="text-5xl font-semibold tracking-tight text-white mt-10 mb-6" { (post.meta.title) }
        (render::render_post(post))
    };
    let head_extras = html! {
        script src="/js/code-copy.js" defer {}
    };
    layout_with_head(post.meta.title, content, Some(head_extras)).into_string()
}

fn post_pages() -> &'static HashMap<&'static str, String> {
    POST_PAGES.get_or_init(|| {
        let mut pages = HashMap::new();
        for post in Posts::published_posts() {
            pages.insert(post.id, render_post_page(post));
        }
        pages
    })
}

pub fn pre_render_posts() {
    let _ = post_pages();
}

pub async fn get_post(Path(id): Path<String>) -> (StatusCode, Html<String>) {
    let id = id.to_lowercase();
    if let Some(page) = post_pages().get(id.as_str()) {
        return (StatusCode::OK, Html(page.clone()));
    }

    let (status, page) = pages::not_found().await;
    (status, Html(page.into_string()))
}

pub fn get_posts() -> &'static Vec<&'static Post> {
    Posts::published_posts()
}
