use crate::{common::layout, content::get_posts};
use maud::{Markup, html};

pub fn post_section() -> Markup {
    let posts = get_posts();

    let content = html! {
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

    content
}

pub async fn page() -> Markup {
    let content = html! {
        div class="flex flex-col items-start gap-6 sm:flex-row sm:items-center my-4 mt-10" {
            img src="/img/avatar.png"
                alt="Portrait of Lucas de Castro"
                class="size-32 shrink-0 rounded-full border-4 border-white/20 object-cover";
            p class="m-0 text-base leading-relaxed" {
                "Hey, I'm Lucas de Castro. I'm a backend software engineer, and this "
                a href="https://github.com/ae2rs/blog.rs" { "open source" }
                " blog is where I (irregularly) post about pretty much anything I find interesting."
                br;
                a class="mt-2 inline-block" href="/about" { "Read more ››" }
            }
        }

        section class="mt-20" {
            h2 class="text-xl font-semibold" { "Latest posts" }
            ul class="mt-4" { (post_section()) }
        }
    };

    layout("Lucas' Hut", content)
}
