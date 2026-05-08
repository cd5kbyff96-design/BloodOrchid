use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::node::{Node, ValidationError};

#[async_trait]
pub trait StructuralFactory: Send + Sync {
    async fn create(&self, node: Node) -> Result<Arc<Node>, ValidationError>;
    async fn query(&self, id: &str) -> Result<Option<Arc<Node>>, ValidationError>;
    async fn update(&self, id: &str, update_fn: Box<dyn FnOnce(&mut Node) + Send>) -> Result<Arc<Node>, ValidationError>;
    async fn delete(&self, id: &str) -> Result<(), ValidationError>;
}

pub struct StructuralFactoryImpl {
    storage: Arc<Mutex<HashMap<String, Arc<Node>>>>,
}

impl StructuralFactoryImpl {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl StructuralFactory for StructuralFactoryImpl {
    async fn create(&self, node: Node) -> Result<Arc<Node>, ValidationError> {
        node.validate()?;
        
        let mut storage = self.storage.lock().unwrap();
        if storage.contains_key(&node.id) {
            return Err(ValidationError::InvalidId(format!("Node {} already exists", node.id)));
        }
        
        let node_arc = Arc::new(node);
        storage.insert(node_arc.id.clone(), Arc::clone(&node_arc));
        Ok(node_arc)
    }

    async fn query(&self, id: &str) -> Result<Option<Arc<Node>>, ValidationError> {
        let storage = self.storage.lock().unwrap();
        Ok(storage.get(id).cloned())
    }

    async fn update(&self, id: &str, update_fn: Box<dyn FnOnce(&mut Node) + Send>) -> Result<Arc<Node>, ValidationError> {
        let mut storage = self.storage.lock().unwrap();
        let node_arc = storage.get(id).ok_or_else(|| ValidationError::InvalidId(format!("Node {} not found", id)))?;
        
        // Create a mutable copy for the update
        let mut node_clone = (**node_arc).clone();
        update_fn(&mut node_clone);
        node_clone.validate()?;
        
        let new_node_arc = Arc::new(node_clone);
        storage.insert(id.to_string(), Arc::clone(&new_node_arc));
        Ok(new_node_arc)
    }

    async fn delete(&self, id: &str) -> Result<(), ValidationError> {
        let mut storage = self.storage.lock().unwrap();
        if storage.remove(id).is_none() {
            return Err(ValidationError::InvalidId(format!("Node {} not found", id)));
        }
        Ok(())
    }
}
