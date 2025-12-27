use crate::{common::layout, content::Post};
use maud::{Markup, html};

pub async fn page(posts: &[&'static Post]) -> Markup {
    let posts = html! {
        @for post in posts {
            div {
                a class="text-white/80 hover:text-white" href=(format!("/post/{}", post.id)) {
                    (post.meta.title)
                }
                span class="text-gray-500 whitespace-nowrap" {
                    " · "
                    (post.meta.published.year)
                    "-"
                    (post.meta.published.month)
                    "-"
                    (post.meta.published.day)
                }
            }
        }
    };

    let content = html! {
        div class="mt-4 space-y-4" {
            h2 class="text-xl font-semibold" { "Posts" }
            ul class="mt-4" { (posts) }
        }
    };

    layout("Posts", content)
}
