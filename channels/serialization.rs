use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct ChannelConfig {
    pub capacity: usize,
    pub retry_ms: Vec<u64>,
    pub serialization: String,
}

impl ChannelConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: ChannelConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

pub struct Serializer;

impl Serializer {
    pub fn to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
        serde_json::to_string(value)
    }

    pub fn from_json<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T, serde_json::Error> {
        serde_json::from_str(json)
    }

    // For protobuf, in a real implementation we would use prost
    pub fn to_proto<T: prost::Message>(value: &T) -> Result<Vec<u8>, prost::EncodeError> {
        let mut buf = Vec::new();
        value.encode(&mut buf)?;
        Ok(buf)
    }
}
