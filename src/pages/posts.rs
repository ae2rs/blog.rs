use crate::common::layout;
use maud::{Markup, html};

pub async fn page() -> Markup {
    let content = html! {
        p {
            "I'm not quite ready to write about anything yet, but I'll keep you posted!"
        }
    };

    layout("Posts", content)
}
