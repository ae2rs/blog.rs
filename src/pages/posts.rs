use crate::{common::layout, content::get_posts};
use maud::{Markup, html};

pub async fn page() -> Markup {
    let posts = get_posts();

    let content = html! {
        @for post in posts {
            div class="mb-4" {
                a href=(format!("/{}", post.id)) { (post.meta.title) }
                p class="text-gray-500" {
                    (post.meta.published.year)
                    "-"
                    (post.meta.published.month)
                    "-"
                    (post.meta.published.day)
                }
            }
        }
    };

    layout("Posts", content)
}
