use mongodb::{bson, Collection};
use mongodb::bson::doc;
use rig::providers::openai::{Client, EmbeddingModel};
use rig::completion::Message;
use rig::embeddings::EmbeddingsBuilder;
use crate::memory::Memory;
use schemars::JsonSchema;

#[derive(serde::Deserialize, serde::Serialize, JsonSchema)]
struct Factoid {
    fact: String,
}
#[derive(serde::Deserialize, serde::Serialize, JsonSchema)]
struct Factoids {
    factoids: Vec<Factoid>,
}

pub(crate) async fn summarize_chunks(
    openai_client: &Client,
    mongo: &Collection<bson::Document>,
    messages: Vec<Message>,
    conversation_id: &str,
    embedding_model: &EmbeddingModel,
) -> Result<(), Box<dyn std::error::Error>> {
    let messages = messages.into_iter().rev().take(6).collect::<Vec<Message>>();

    // Ensure there are a reasonable number of turns that have passed before trying to summarize the message.
    // In this case, we want to summarize every 3 turns - so we need to ensure the remainder after dividing by 6 is 0
    // (because 3 turns = 6 messages)
    // If it doesn't meet the required criteria, just return early

    if messages.len() % 6 != 0 {
        return Err("Not enough new messages to summarize"
            .into());
    }

    let agent = openai_client
        .extractor::<Factoids>("gpt-4o")
        .preamble("Please summarize the inputted conversation chunk by the user into a list of factoids, using the JSON schema provided.")
        .build();

    let messages_as_string = serde_json::to_string_pretty(&messages).unwrap();

    let res = agent.extract(&messages_as_string).await.unwrap();

    let response_as_memory_entries = res
        .factoids
        .into_iter()
        .map(|x| Memory::new(&conversation_id, x.fact))
        .collect::<Vec<Memory>>();

    let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        .documents(response_as_memory_entries)?
        .build()
        .await?;

    let mongo_documents = embeddings
        .iter()
        .map(
            |(
                 Memory {
                     id,
                     conversation_id,
                     memory,
                     timestamp_created,
                 },
                 embedding,
             )| {
                doc! {
                    "id": id.clone(),
                    "conversation_id": conversation_id.clone(),
                    "memory": memory.clone(),
                    "timestamp_created": i64::try_from(*timestamp_created).unwrap(),
                    "embedding": embedding.first().vec.clone(),
                }
            },
        )
        .collect::<Vec<_>>();

    mongo.insert_many(mongo_documents).await?;


    Ok(())

}