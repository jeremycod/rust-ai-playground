use axum::response::{IntoResponse, Response};


#[derive(Debug, thiserror::Error)]
pub enum ArxivError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("XML parsing error: {0}")]
    XmlParsing(#[from] quick_xml::Error),
    #[error("No results found")]
    NoResults,
    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}