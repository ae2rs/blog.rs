use maud::{DOCTYPE, Markup, html};

pub fn layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                title { (title) }
                ;
                link rel="stylesheet" href="/style/index.css";
            }
            body {
                main {
                    h1 class="text-3xl font-semibold" { "Lucas' Hut" }
                    nav class="mb-6 flex items-center gap-4" {
                        a class="" href="/" { "Home" }
                        a href="/about" { "About" }
                    }
                    (content)
                }
            }
        }
    }
}
