use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use axum::{Router, routing::get};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

use blib::content;
use blib::pages;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(pages::index))
        .route("/about", get(pages::about))
        .route("/posts", get(pages::posts))
        .route("/post/{id}", get(content::get_post))
        .nest_service("/style", ServeDir::new("build/style"))
        .nest_service("/img", ServeDir::new("build/img"))
        .fallback(pages::not_found)
        .layer(SetResponseHeaderLayer::overriding(
            CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=86400"),
        ));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind");
    axum::serve(listener, app).await.expect("failed to serve");
}
