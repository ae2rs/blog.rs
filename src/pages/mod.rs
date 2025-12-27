mod about;
mod index;
mod not_found;
mod posts;

use axum::extract::State;
use maud::Markup;
use std::sync::Arc;

use crate::state::AppState;

pub use about::page as about;
pub use not_found::page as not_found;

pub async fn index(State(state): State<Arc<AppState>>) -> Markup {
    index::page(state.posts()).await
}

pub async fn posts(State(state): State<Arc<AppState>>) -> Markup {
    posts::page(state.posts()).await
}
