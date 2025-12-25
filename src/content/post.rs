use crate::{common::layout, content::meta::Post, pages};
use axum::{extract::Path, http::StatusCode};
use maud::{Markup, PreEscaped};
use post_macros::Post;

#[derive(Post)]
struct Posts;

fn get_post_by_id(id: &str) -> Option<&'static Post> {
    Posts::get(id).filter(|post| !post.meta.draft)
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

pub fn get_posts() -> Vec<&'static Post> {
    let mut posts = Posts::iter().collect::<Vec<_>>();

    posts.sort_by_key(|post| post.meta.published);

    posts
}
