use crate::{common::layout, content::meta::Post, pages};
use axum::{extract::Path, http::StatusCode};
use macros::Post;
use maud::{Markup, html};

mod highlight;
mod render;

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
        layout(
            post.meta.title,
            html! {
                h1 class="text-5xl font-semibold tracking-tight text-white mt-10 mb-6" {
                    (post.meta.title)
                }
                (render::render_post(post))
            },
        ),
    )
}

pub fn get_posts() -> &'static Vec<&'static Post> {
    Posts::published_posts()
}
