use maud::{DOCTYPE, Markup, html};

fn doctype() -> Markup {
    html! {
        (DOCTYPE)
    }
}

pub async fn page() -> Markup {
    html! {
        (doctype())
        html lang="en" {
            head {
                meta charset="utf-8";
                title { "Lucas' Hut" };
                link rel="stylesheet" href="/style/index.css";
            }
            body {
                h1 { "Lucas' Hut" }
                p { "Welcome to my hut!" }
            }
        }
    }
}
