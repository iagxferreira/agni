use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Entry {
    pub id: Uuid,
    pub key: String,
    #[serde(serialize_with = "serialize_value")]
    pub value: Vec<u8>,
}

fn serialize_value<S>(value: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&STANDARD.encode(value))
}

impl Entry {
    pub fn new(key: String, value: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            key,
            value,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
