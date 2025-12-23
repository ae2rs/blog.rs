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
                main { (content) }
            }
        }
    }
}
