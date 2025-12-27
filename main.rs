use axum::{Router, routing::get};
use std::sync::Arc;
use tower_http::services::ServeDir;

use blib::content;
use blib::pages;
use blib::state::AppState;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/", get(pages::index))
        .route("/about", get(pages::about))
        .route("/posts", get(pages::posts))
        .route("/post/{id}", get(content::get_post))
        .nest_service("/style", ServeDir::new("build/style"))
        .nest_service("/img", ServeDir::new("build/img"))
        .nest_service("/js", ServeDir::new("build/js"))
        .fallback(pages::not_found)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind");
    axum::serve(listener, app).await.expect("failed to serve");
}
