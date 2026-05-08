use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid node ID: {0}")]
    InvalidId(String),
    #[error("Hierarchy violation: {0}")]
    HierarchyError(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub parent_id: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
    pub properties: std::collections::HashMap<String, String>,
    pub version: u64,
}

impl Node {
    pub fn new(id: String, parent_id: Option<String>) -> Self {
        Self {
            id,
            parent_id,
            metadata: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(),
            version: 0,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id.is_empty() {
            return Err(ValidationError::InvalidId("Node ID cannot be empty".into()));
        }
        
        if self.id == self.parent_id.as_deref().unwrap_or("") {
            return Err(ValidationError::HierarchyError("Node cannot be its own parent".into()));
        }

        Ok(())
    }
}
