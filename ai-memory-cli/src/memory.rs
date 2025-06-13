use nanoid::nanoid;
use chrono::Utc;
use rig::Embed;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Deserialize, Embed, Clone)]
pub(crate) struct Memory {
    #[serde(rename = "_id", deserialize_with = "deserialize_object_id")]
    pub id: String,
    pub conversation_id: String,
    #[embed]
    pub memory: String,
    pub timestamp_created: usize,
}

impl Memory {
    pub fn new(conversation_id: &str, memory: String) -> Self {
        let id = nanoid!(10);

        let conversation_id = conversation_id.to_string();
        let timestamp_created = Utc::now().timestamp();
        let timestamp_created = usize::try_from(timestamp_created)
            .expect("Timestamp is negative or too large for usize");

        Self {
            id,
            conversation_id,
            memory,
            timestamp_created
        }
    }
}

fn deserialize_object_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Object(map) => {
            if let Some(Value::String(oid)) = map.get("$oid"){
                Ok(oid.to_string())
            } else {
                Err(serde::de::Error::custom("Expected $oid field with string value",))
            }
        }
        _ => Err(serde::de::Error::custom(
            "Expected string or object with $oid field",
        ))
    }

}