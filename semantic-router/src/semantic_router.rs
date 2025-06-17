use rig::client::EmbeddingsClient;
use std::env;
use std::sync::Arc;
use dotenv::dotenv;
use rig::{embeddings::EmbeddingsBuilder, providers::openai::{Client, EmbeddingModel as Model, TEXT_EMBEDDING_ADA_002}};
use rig_qdrant::QdrantVectorStore;
use qdrant_client::{qdrant::{CreateCollectionBuilder, PointStruct, QueryPointsBuilder, VectorParamsBuilder}, Payload, Qdrant};
use qdrant_client::qdrant::UpsertPointsBuilder;
//use rig::client::embeddings::EmbeddingsClientDyn;
use rig::vector_store::VectorStoreIndex;
use crate::topics::Utterance;

pub struct SemanticRouter {
    model: Model,
    qdrant: Arc<Qdrant>,
    vector_store: QdrantVectorStore<Model>,
}

const COLLECTION_NAME: &str = "SEMANTIC_ROUTING";
const COLLECTION_SIZE: usize = 1536;
impl SemanticRouter {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>>{
        dotenv().ok();
        // Initialize OpenAI client.
        // Get your API key from https://platform.openai.com/api-keys
        let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let openai_client = Client::new(&openai_api_key);

        let model = openai_client.embedding_model(TEXT_EMBEDDING_ADA_002);

        let qdrant = Arc::new(Qdrant::from_url("http://localhost:6334").build()?);

        // note that we use `Arc::into_inner() here` because
        // the Qdrant client doesn't actually implement Clone
        // so we need to create an `Arc::clone` then get the inner value
        //let qdrant_inner = Arc::clone(&qdrant);
        //let qdrant_inner = Arc::into_inner(qdrant_inner).unwrap();

        // Create a collection with 1536 dimensions if it doesn't exist
        // Note: Make sure the dimensions match the size of the embeddings returned by the
        // model you are using
        if !qdrant.collection_exists(COLLECTION_NAME).await? {
            qdrant
                .create_collection(
                    CreateCollectionBuilder::new(COLLECTION_NAME)
                        .vectors_config(VectorParamsBuilder::new(COLLECTION_SIZE as u64, qdrant_client::qdrant::Distance::Cosine)),
                )
                .await?;
        }

        let qdrant_for_store = Qdrant::from_url("http://localhost:6334").build()?;
        let query_params = QueryPointsBuilder::new(COLLECTION_NAME).with_payload(true);
        let vector_store = QdrantVectorStore::new(qdrant_for_store, model.clone(), query_params.build());
        Ok(Self {
            model,
            qdrant,
            vector_store
        })
    }
    pub async fn embed_utterances(&self, utterances: Vec<Utterance>) -> Result<(), Box<dyn std::error::Error>> {
        let documents = EmbeddingsBuilder::new(self.model.clone())
            .documents(utterances)?
            .build()
            .await?;

        let points: Vec<PointStruct> = documents
            .into_iter()
            .map(|(d, embeddings)| {
                let vec: Vec<f32> = embeddings.first().vec.iter().map(|&x| x as f32).collect();
                PointStruct::new(
                    d.id.clone(),
                    vec,
                    Payload::try_from(serde_json::to_value(&d).unwrap()).unwrap(),
                )
            })
            .collect();

        self.qdrant
            .upsert_points(UpsertPointsBuilder::new(COLLECTION_NAME, points))
            .await?;

        Ok(())
    }

    pub async fn query(&self, query: &str) -> Result<Utterance, Box<dyn std::error::Error>> {

        let results = self.vector_store
            .top_n::<Utterance>(query, 1)
            .await?;

        if results[0].0 <= 0.85 {
            return Err("No relevant snippet found.".into());
        }

        Ok(results[0].2.clone())
    }
}