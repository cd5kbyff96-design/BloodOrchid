use std::sync::Arc;
use tokio::sync::mpsc;
use structural::{StructuralFactoryImpl, Node};
use channels::structural_dispatcher::{StructuralDispatcher, Message};

#[tokio::test]
async fn test_agent_simulation() {
    let factory = Arc::new(StructuralFactoryImpl::new());
    let (dispatcher, mut rx) = StructuralDispatcher::new(
        factory.clone(),
        1024,
        vec![100, 250, 500]
    );

    // Agent 1: Create node
    let node = Node::new("agent1_node".to_string(), None);
    dispatcher.dispatch(Message::Create(node)).await.unwrap();

    // Agent 2: Query node
    dispatcher.dispatch(Message::Query("agent1_node".to_string())).await.unwrap();

    // Processor loop
    let mut processed = 0;
    while processed < 2 {
        if let Some(msg) = rx.recv().await {
            StructuralDispatcher::process_message(factory.clone(), msg).await.unwrap();
            processed += 1;
        }
    }

    let result = factory.query("agent1_node").await.unwrap();
    assert!(result.is_some());
}
