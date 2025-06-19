use crate::model::Paper;

pub(crate) fn convert_pdf_url(url: &str) -> String {
    if url.contains("arxiv.org/abs/") {
        // Convert abstract URL to PDF URL
        url.replace("arxiv.org/abs/", "arxiv.org/pdf/")
            .replace("http://", "https://")
            + ".pdf"
    } else if url.contains("arxiv.org/pdf/") {
        // Ensure PDF URL uses HTTPS
        url.replace("http://", "https://")
    } else {
        // Fallback for other URLs
        url.replace("http://", "https://")
    }
}

// HTML formatting function for papers
pub fn format_papers_as_html(papers: &[Paper]) -> Result<String, anyhow::Error> {
    let tpl = std::fs::read_to_string("static/table.html")?;
    let mut context = tera::Context::new();
    context.insert("papers", papers);

    let result = tera::Tera::one_off(&tpl, &context, false)?;

    Ok(result)
}