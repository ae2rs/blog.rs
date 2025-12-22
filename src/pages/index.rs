use maud::{DOCTYPE, Markup, html};

pub async fn page() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                title { "Lucas' Hut" };
                link rel="stylesheet" href="/style/index.css";
            }
            body {
                main {
                    h1 { "Lucas' Hut" }
                    p { "Welcome to my hut!" }
                    p {
                        a href="#" { "Read the first post" }
                    }
                }
            }
        }
    }
}
