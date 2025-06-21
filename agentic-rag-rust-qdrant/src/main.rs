mod agent;
mod file;

use std::sync::Arc;
use dotenv::dotenv;
use axum::{Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{CreateCollectionBuilder, VectorParamsBuilder};
use serde::Deserialize;
use tokio::net::TcpListener;
use tracing::info;
use crate::agent::{MyAgent, COLLECTION, COLLECTION_SIZE};
use crate::file::File;

#[derive(Deserialize)]
pub struct Prompt {
    prompt: String,
}

#[derive(Clone)]
pub struct AppState {
    agent: MyAgent,
}

async fn prompt(
    State(state): State<AppState>,
    Json(json): Json<Prompt>,
) -> Result<impl IntoResponse, StatusCode> {
    let prompt_response = state.agent.prompt(&json.prompt).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, prompt_response))
}

async fn hello_world() -> impl IntoResponse {
    Html("Hello world!")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // note that this already assumes you have a file called "test.csv"
    // in your project root
    let file = File::new("test.csv".into())?;
    let qdrant_client = Arc::new(Qdrant::from_url("http://localhost:6334").build()?);
    if !qdrant_client.collection_exists(COLLECTION).await? {
        qdrant_client
            .create_collection(
                CreateCollectionBuilder::new(COLLECTION)
                    .vectors_config(VectorParamsBuilder::new(COLLECTION_SIZE as u64, qdrant_client::qdrant::Distance::Cosine)),
            )
            .await?;
    }
    let state = AppState {
        agent: MyAgent::new(qdrant_client),
    };

    state.agent.embed_document(file).await?;

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/prompt", post(prompt))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:8004").await.unwrap();
    info!("->> {:<12} - {:?}\n", "LISTENING", listener.local_addr());

    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
    Ok(())
}
