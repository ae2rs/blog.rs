use axum::http::StatusCode;
use maud::{Markup, html};

use crate::common::layout;

pub async fn page() -> (StatusCode, Markup) {
    let content = html! {
        div class="min-h-[60vh] flex flex-col items-center justify-center text-center" {
            h2 class="text-lg font-medium text-white/60" { "not found — sorry." }
        }
    };

    (StatusCode::NOT_FOUND, layout("Not Found", content))
}
