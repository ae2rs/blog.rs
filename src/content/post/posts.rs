use crate::{
    common::layout_with_head, content::format::highlight::Highlighter, pages, state::AppState,
};
use axum::{extract::Path, extract::State, http::StatusCode, response::Html};
use macros::Post;
use maud::html;
use std::{collections::HashMap, sync::Arc};

use super::{render, types::Post};

#[derive(Post)]
struct Posts;

pub struct PostState {
    posts: Vec<&'static Post>,
    pages: HashMap<&'static str, String>,
}

impl PostState {
    pub fn new(highlighter: &Highlighter) -> Self {
        let posts = Posts::published_posts().clone();
        let mut pages = HashMap::new();
        for post in posts.iter().copied() {
            pages.insert(post.id, render_post_page(post, highlighter));
        }

        Self { posts, pages }
    }

    pub fn posts(&self) -> &[&'static Post] {
        &self.posts
    }

    pub fn page(&self, id: &str) -> Option<&String> {
        self.pages.get(id)
    }
}

fn render_post_page(post: &Post, highlighter: &Highlighter) -> String {
    let content = html! {
        h1 class="text-5xl font-semibold tracking-tight text-white mt-10 mb-6" { (post.meta.title) }
        (render::render_post(post, highlighter))
    };
    let head_extras = html! {
        script src="/js/code-copy.js" defer {}
    };
    layout_with_head(post.meta.title, content, Some(head_extras)).into_string()
}

pub async fn get_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Html<String>) {
    let id = id.to_lowercase();
    if let Some(page) = state.post_page(id.as_str()) {
        return (StatusCode::OK, Html(page.clone()));
    }

    let (status, page) = pages::not_found().await;
    (status, Html(page.into_string()))
}
