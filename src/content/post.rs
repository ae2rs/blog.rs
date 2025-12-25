use crate::{common::layout, content::meta::Post, pages};
use axum::{extract::Path, http::StatusCode};
use macros::Post;
use maud::{Markup, PreEscaped};

#[derive(Post)]
struct Posts;

fn get_post_by_id(id: &str) -> Option<&'static Post> {
    Posts::get_published(id)
}

pub async fn get_post(Path(id): Path<String>) -> (StatusCode, Markup) {
    let id = id.to_lowercase();
    let post = if let Some(post) = get_post_by_id(&id) {
        post
    } else {
        return (StatusCode::NOT_FOUND, pages::not_found().await.1);
    };
    (
        StatusCode::OK,
        layout(post.meta.title, PreEscaped(post.html.to_string())),
    )
}

pub fn get_posts() -> &'static Vec<&'static Post> {
    Posts::published_posts()
}
