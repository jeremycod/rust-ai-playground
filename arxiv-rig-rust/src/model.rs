use rig::providers::openai;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Paper {
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub url: String,
    pub categories: Vec<String>,
}

impl Paper {
    pub(crate) fn new() -> Self {
        Self {
            title: String::new(),
            authors: Vec::new(),
            abstract_text: String::new(),
            url: String::new(),
            categories: Vec::new(),
        }
    }
}

// Request structure for search endpoint
#[derive(serde::Deserialize)]
pub(crate) struct SearchRequest {
    pub(crate) query: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SearchArgs {
    pub(crate) query: String,
    pub(crate) max_results: Option<i32>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ArxivSearchTool;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) openai_client: openai::Client,
}
