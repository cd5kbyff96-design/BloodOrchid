use structural::{StructuralFactory, StructuralFactoryImpl, Node};

#[tokio::test]
async fn test_roundtrip() {
    let factory = StructuralFactoryImpl::new();
    let node = Node::new("node1".to_string(), None);
    
    let created = factory.create(node).await.unwrap();
    assert_eq!(created.id, "node1");
    
    let queried = factory.query("node1").await.unwrap();
    assert!(queried.is_some());
    assert_eq!(queried.unwrap().id, "node1");
}

#[tokio::test]
async fn test_hierarchy() {
    let factory = StructuralFactoryImpl::new();
    
    let parent = Node::new("parent".to_string(), None);
    factory.create(parent).await.unwrap();
    
    let child = Node::new("child".to_string(), Some("parent".to_string()));
    factory.create(child).await.unwrap();
    
    let child_query = factory.query("child").await.unwrap().unwrap();
    assert_eq!(child_query.parent_id.as_deref(), Some("parent"));
}

#[tokio::test]
async fn test_determinism() {
    let factory = StructuralFactoryImpl::new();
    let node = Node::new("node1".to_string(), None);
    
    let node1 = factory.create(node.clone()).await.unwrap();
    
    // Test update consistency
    let updated = factory.update("node1", Box::new(|n| {
        n.properties.insert("key".into(), "value".into());
    })).await.unwrap();
    
    assert_eq!(updated.properties.get("key").unwrap(), "value");
    
    let query_res = factory.query("node1").await.unwrap().unwrap();
    assert_eq!(query_res.properties.get("key").unwrap(), "value");
}

#[tokio::test]
async fn test_validation() {
    let factory = StructuralFactoryImpl::new();
    
    // Empty ID should fail
    let node_empty = Node::new("".to_string(), None);
    assert!(factory.create(node_empty).await.is_err());
    
    // Self-parenting should fail
    let node_self = Node::new("self".to_string(), Some("self".to_string()));
    assert!(factory.create(node_self).await.is_err());
}

#[tokio::test]
async fn test_deletion() {
    let factory = StructuralFactoryImpl::new();
    let node = Node::new("delete_me".to_string(), None);
    
    factory.create(node).await.unwrap();
    assert!(factory.query("delete_me").await.unwrap().is_some());
    
    factory.delete("delete_me").await.unwrap();
    assert!(factory.query("delete_me").await.unwrap().is_none());
}
