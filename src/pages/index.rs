use crate::common::layout;
use maud::{Markup, html};

pub async fn page() -> Markup {
    let content = html! {
        div class="flex flex-col items-start gap-6 sm:flex-row sm:items-center my-4" {
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

        section class="mt-8" {
            h2 class="text-xl font-semibold" { "Latest posts" }
            ul class="mt-4 space-y-2" {
                li {
                    a href="#" { "Read the first post" }
                }
            }
        }
    };

    layout("Lucas' Hut", content)
}
