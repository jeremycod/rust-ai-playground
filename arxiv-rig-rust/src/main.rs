use std::sync::Arc;
use axum::response::{Html, IntoResponse};
use axum::{Router};
use axum::routing::{get, post};

use dotenv::dotenv;
use rig::client::{ProviderClient};
use rig::providers::openai::{Client};
use tokio::net::TcpListener;
use crate::model::AppState;
use tower_http::cors::{CorsLayer, Any};
use tracing::info;

mod model;
mod error;
mod arxiv_search_tool;
mod arxiv_parser;
mod util;
mod routes;

// Handler for serving the static index.html
async fn serve_index() -> impl IntoResponse {
    Html(include_str!("../static/index.html"))
}
async fn serve_test() -> impl IntoResponse {
    Html("Hello world!")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
  /*  let openai_key = secrets
        .get("OPENAI_API_KEY")
        .context("OPENAI_API_KEY secret not found")?;*/
    let openai_client = Client::from_env();

   // let openai_client = openai::Client::new(&openai_key);

    // Create shared state
    let state = Arc::new(AppState {
        openai_client,
    });

    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers(Any);

    // Create router
    let router = Router::new()
        .route("/", get(serve_index))
        .route("/test", get(serve_test))
        .route("/api/search", post(routes::search_papers))
        .layer(cors)
        .with_state(state.clone());
    let listener = TcpListener::bind("127.0.0.1:8003").await.unwrap();
    info!("->> {:<12} - {:?}\n", "LISTENING", listener.local_addr());

    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
    Ok(())
}
