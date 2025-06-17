use rig::{embeddings::{EmbedError, TextEmbedder}, Embed};
use uuid;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Utterance {
    pub id: String,
    pub topic: String,
    pub content: String,
}

impl Utterance {
    pub fn new(topic: &str, content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            topic: topic.to_string(),
            content: content.to_string(),
        }
    }
}
impl Embed for Utterance {
    fn embed(&self, embedder: &mut TextEmbedder) -> Result<(), EmbedError> {
        // Embeddings only need to be generated for `content` field.
        // Queries will be compared against the content        
        embedder.embed(self.content.to_owned());

        Ok(())
    }
}

pub struct Topic {
    name: String
}

impl Topic {
    pub fn new(name: &str) -> Self {
        let name = name.to_string();
        Self {
            name
        }
    }

    pub fn new_utterance(&self, content: &str) -> Utterance {
        Utterance::new(&self.name, content)
    }
}