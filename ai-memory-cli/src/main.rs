mod memory;
mod db;
mod summarize;

use std::io::{stdin, stdout, Write};
use mongodb::{bson, Collection};
use mongodb::bson::doc;
use nanoid::nanoid;

use rig::completion::{Chat, Message};
use rig::providers::openai::{Client, EmbeddingModel};
use rig::embeddings::EmbeddingModel as EmbeddingModelTrait;
use futures::stream::TryStreamExt;

fn take_user_input() -> String {
    let mut string = String::new();

    print!("> ");
    let _ = stdout().flush();

    stdin().read_line(&mut string).unwrap();

    string
}

async fn _chat_with_short_term_memory() -> Result<(), Box<dyn std::error::Error>> {
    let mut messages = Vec::new();

    let openai_client = Client::from_env();

    loop {
        let query: String = take_user_input();

        let agent = openai_client
            .agent("gpt-4o")
            .preamble("You are a helpful agent.")
            .build();

        let response = agent.chat(query.as_str(), messages.clone()).await?;

        println!("{response}");
        messages.push(Message::user(query));
        messages.push(Message::assistant(response));
    }
}


// Helper function for cosine similarity
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
    let norm_a = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

async fn get_relevant_memories(
    mongo: &Collection<bson::Document>,
    query: &str,
    model: &EmbeddingModel,
) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
    // Compute embedding for the query
    let query_embedding = model.embed_text(query).await?;

    // Fetch all memories
    let mut cursor = mongo.find(doc! {}).await?;
    let mut memories = Vec::new();

    while let Some(doc) = cursor.try_next().await? {
        // Adjust field names as needed
        let embedding_bson = doc.get_array("embedding")?;
        let embedding: Vec<f64> = embedding_bson
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f64)
            .collect();
        let text = doc.get_str("text").unwrap_or("").to_string();
        memories.push((embedding, text));
    }

    // Compute similarity and get top 5
    let mut scored_memories: Vec<(f64, String)> = memories
        .into_iter()
        .map(|(embedding, text)| (cosine_similarity(&embedding, &query_embedding.vec.as_slice()), text))
        .collect();

    scored_memories.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let top_memories: Vec<String> = scored_memories
        .into_iter()
        .take(5)
        .map(|(_, text)| text)
        .collect();

    if top_memories.is_empty() {
        Ok(None)
    } else {
        Ok(Some(top_memories))
    }
}

async fn chat_with_long_term_memory() -> Result<(), Box<dyn std::error::Error>> {
    let mut messages = Vec::new();

    let openai_client = Client::from_env();
    let embedding_model = openai_client.embedding_model("text-embedding-ada-002");
    let mongo = db::connect_to_mongodb().await;
    let conversation_id = nanoid!(6);

    loop {
        let query: String = take_user_input();

        let additional_input = if let Some(messages) =
            get_relevant_memories(&mongo, &query, &embedding_model).await?
        {
            format!(
                "\n\nRelevant memories from previous conversations: {}",
                messages.join("\n")
            )

        } else {
            String::new()
        };

        let agent = openai_client
            .agent("gpt-4o")
            .preamble("You are a helpful agent.")
            .append_preamble(&additional_input)
            .build();

        let response = agent.chat(query.as_str(), messages.clone()).await?;

        println!("{response}");
        messages.push(Message::user(query));
        messages.push(Message::assistant(response));

        if summarize::summarize_chunks(
            &openai_client,
            &mongo,
            messages.clone(),
            &conversation_id,
            &embedding_model
        )
            .await
            .is_ok()
        {
            println!("Saved new memories")
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    chat_with_long_term_memory().await
}
