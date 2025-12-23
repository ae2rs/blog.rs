use crate::common::layout;
use maud::{Markup, html};

pub async fn page() -> Markup {
    let content = html! {
        h1 { "Lucas' Hut" }
        div class="flex flex-col items-start gap-4 sm:flex-row sm:items-center my-4" {
            img
                src="/img/avatar.png"
                alt="Portrait of Lucas de Castro"
                class="h-24 w-24 shrink-0 rounded-full object-cover";
            p class="text-base leading-relaxed" {
                "Hey, I'm Lucas de Castro. I'm a backend software engineer, and this open source blog is where I (irregularly) post about pretty much anything I find interesting."
            }
        }

        p {
            a href="#" { "Read the first post" }
        }
    };

    layout("Lucas' Hut", content)
}
