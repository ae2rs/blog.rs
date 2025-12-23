use axum::{Router, routing::get};
use tower_http::services::ServeDir;

use blib::pages;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(pages::index))
        .nest_service("/style", ServeDir::new("public/style"))
        .nest_service("/img", ServeDir::new("public/img"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind");
    axum::serve(listener, app).await.expect("failed to serve");
}
