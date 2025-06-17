use rig::providers::openai;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Define an enum to represent sentiment categories
#[derive(Debug, Deserialize, JsonSchema, Serialize)]
enum Sentiment {
    Positive,
    Negative,
    Neutral,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
enum Topic {
    Politics,
    Technology,
    Sports,
    Entertainment,
    Other(String)
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub(crate) struct NewsArticleClassification {
    topic: Topic,
    sentiment: SentimentClassification,
    summary: String
}

// Define a struct to hold the sentiment classification result
#[derive(Debug, Deserialize, JsonSchema, Serialize)]
pub(crate) struct SentimentClassification {
    sentiment: Sentiment,
    confidence: f32,
}

pub(crate) fn pretty_print_result(article: &str, result: &NewsArticleClassification) {
    println!("Article: \"{}...\"", &article[..100]); // Print first 100 characters
    println!("Classification Result:");
    println!("  Topic: {:?}", result.topic);
    println!("  Sentiment: {:?}", result.sentiment.sentiment);
    println!("  Confidence: {:.2}%", result.sentiment.confidence * 100.0);
    println!("  Summary: {}", result.summary);
    println!();
}