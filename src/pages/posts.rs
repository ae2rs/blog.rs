use crate::{common::layout, content::get_posts};
use maud::{Markup, html};

pub async fn page() -> Markup {
    let posts = get_posts();

    let posts = html! {
        @for post in posts {
            div {
                a class="text-white/80 hover:text-white" href=(format!("/post/{}", post.id)) {
                    (post.meta.title)
                }
                span class="text-gray-500" {
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
