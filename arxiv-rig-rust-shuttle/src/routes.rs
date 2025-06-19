use std::sync::Arc;
use axum::extract::State;
use axum::Json;
use axum::response::{Html, IntoResponse};
use serde::Deserialize;
//use serde_json::ser::State;
use rig::providers::openai::GPT_4;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use crate::model::{ArxivSearchTool, AppState, Paper, SearchRequest};
use crate::error::AppError;
use crate::util;


pub(crate) async fn search_papers(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SearchRequest>,
) -> Result<impl IntoResponse, AppError> {
    let paper_agent = state.openai_client
        .agent(GPT_4)
        .preamble(
            "You are a helpful research assistant that can search and analyze academic papers from arXiv. \
             When asked about a research topic, use the search_arxiv tool to find relevant papers and \
             return only the raw JSON response from the tool."
        )
        .tool(ArxivSearchTool)
        .build();

    let response = paper_agent
        .prompt(&request.query)
        .await?;

    // return the response as HTML
    // note that if you want to return just a JSON response
    // you can return `Ok(axum::Json(papers))`
    let papers: Vec<Paper> = serde_json::from_str(&response)?;

    let html = util::format_papers_as_html(&papers)?; // see below!
    Ok(Html(html))
}