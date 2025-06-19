use std::sync::Arc;
use anyhow::Context;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::{Json, Router};
use axum::routing::{get, post};
use rig::client::CompletionClient;
use rig::completion::Prompt;
//use rig::providers::anthropic::completion::ToolChoice::Any;
use rig::providers::openai;
use rig::providers::openai::GPT_4;
use shuttle_runtime::SecretStore;
use crate::model::{AppState, ArxivSearchTool, Paper};
use tower_http::cors::{CorsLayer, Any};
use crate::error::AppError;

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


#[shuttle_runtime::main]
async fn axum(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    for (key, value) in std::env::vars() {
        println!("{}: {}", key, value);
    }
    // Initialize OpenAI client from secrets
    let openai_key = secrets
        .get("OPENAI_API_KEY")
        .context("OPENAI_API_KEY secret not found")?;

    let openai_client = openai::Client::new(&openai_key);

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
        .route("/api/search", post(routes::search_papers))
        .layer(cors)
        .with_state(state.clone());

    Ok(router.into())
}
