use std::sync::Arc;
use std::io::Error;
use crate::file::File;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest, CreateChatCompletionRequestArgs, CreateEmbeddingRequest, EmbeddingInput};
use async_openai::{Client as OpenAIClient, Embeddings, config::OpenAIConfig};
use qdrant_client::qdrant::PointStruct;
use qdrant_client::{Payload, Qdrant};

use qdrant_client::qdrant::with_payload_selector::{SelectorOptions};
use qdrant_client::qdrant::WithPayloadSelector;
use qdrant_client::qdrant::UpsertPointsBuilder;
use qdrant_client::qdrant::SearchPoints;

static PROMPT_MODEL: &str = "gpt-4o";

pub static COLLECTION: &str = "my-collection";
pub static COLLECTION_SIZE: usize = 1536;
static EMBED_MODEL: &str = "text-embedding-ada-002";


#[derive(Clone)]
pub struct MyAgent {
    openai_client: OpenAIClient<OpenAIConfig>,
    qdrant_client: Arc<Qdrant>,
}

static SYSTEM_MESSAGE: &str = "
        You are a world-class data analyst, specialising in analysing comma-delimited CSV files.
	    Your job is to analyse some CSV snippets and determine what the results are for the question that the user is asking.
	    You should aim to be concise. If you don't know something, don't make it up but say 'I don't know.'.
";

impl MyAgent {
    pub async fn prompt(&self, prompt: &str) -> anyhow::Result<String> {
        let context = self.search_document(prompt.to_owned()).await?;
        let input = format!(
            "{prompt}

            Provided context:
            {}
            ",
            context // this is the payload from Qdrant
        );

        let res = self
            .openai_client
            .chat()
            .create(
                CreateChatCompletionRequestArgs::default()
                    .model(PROMPT_MODEL)
                    .messages(vec![
                        ChatCompletionRequestMessage::System(
                            ChatCompletionRequestSystemMessageArgs::default()
                                .content(SYSTEM_MESSAGE)
                                .build()?,
                        ),
                        //THen we add our prompt
                        ChatCompletionRequestMessage::User(
                            ChatCompletionRequestUserMessageArgs::default()
                                .content(input)
                                .build()?,
                        )
                    ])
                    .build()?,
            )
            .await
            .map(|res| {
              // We extract the first one
                res.choices[0].message.content.clone().unwrap()
            })?;
        println!("Retrieved result from prompt: {res}");
        Ok(res)
    }
    async fn search_document(&self, prompt: String) -> anyhow::Result<String> {
        let request = CreateEmbeddingRequest {
            model: EMBED_MODEL.to_string(),
            input: EmbeddingInput::String(prompt),
            user: None,
           // dimensions: Some(1536),
            ..Default::default()
        };

        let embeddings_result = Embeddings::new(&self.openai_client).create(request).await?;

        let embedding = &embeddings_result.data.first().unwrap().embedding;

        let payload_selector = WithPayloadSelector {
            selector_options: Some(SelectorOptions::Enable(true)),
        };

        // set parameters for search
        let search_points = SearchPoints {
            collection_name: COLLECTION.to_string(),
            vector: embedding.to_owned(),
            limit: 1,
            with_payload: Some(payload_selector),
            ..Default::default()
        };

        // if the search is successful
        // attempt to iterate through the results vector and find a result
        let search_result = self.qdrant_client.search_points(search_points).await?;
        let result = search_result.result.into_iter().next();

        match result {
            Some(res) => Ok(res.payload.get("content").unwrap().to_string()),
            None => Err(anyhow::anyhow!("There were no results that matched :(")),
        }
    }
    pub async fn embed_document(&self, file: File) -> anyhow::Result<()> {
        if file.rows.is_empty() {
            return Err(anyhow::anyhow!("There's no rows to embed"));
        }
        let request = CreateEmbeddingRequest {
            model: EMBED_MODEL.to_string(),
            input: EmbeddingInput::StringArray(file.rows.clone()),
            encoding_format: None,
            user: None,
           // dimensions: Some(1536),
            ..Default::default()
        };

        let embeddings_result = Embeddings::new(&self.openai_client).create(request).await?;

        for embedding in embeddings_result.data {
            let payload: Payload = serde_json::json!({
                "id": file.path.clone(),
                "content": file.contents,
                "rows": file.rows
            })
            .try_into()?;

            println!("Embedded: {}", file.path);

            let vec = embedding.embedding;
            let points = vec![PointStruct::new(
                uuid::Uuid::new_v4().to_string(),
                vec,
                payload,
            )];
            let upsert = UpsertPointsBuilder::new(COLLECTION, points)
                .build();
            self.qdrant_client
                .upsert_points(upsert)
                .await?;
        }
        Ok(())
    }

    pub fn new(qdrant_client: Arc<Qdrant>) -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap();
        let config = OpenAIConfig::new().with_api_key(api_key);

        let openai_client = OpenAIClient::with_config(config);

        Self {
            openai_client,
            qdrant_client,
        }
    }
}
