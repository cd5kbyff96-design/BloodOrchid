use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use serde::{Serialize, Deserialize};
use crate::factory::structural::{Node, StructuralFactory, StructuralFactoryImpl};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Create(Node),
    Query(String),
    Update(String, std::collections::HashMap<String, String>),
    Delete(String),
}

pub struct StructuralDispatcher {
    factory: Arc<dyn StructuralFactory>,
    sender: mpsc::Sender<Message>,
    retry_intervals: Vec<u64>,
}

impl StructuralDispatcher {
    pub fn new(factory: Arc<dyn StructuralFactory>, capacity: usize, retry_intervals: Vec<u64>) -> (Self, mpsc::Receiver<Message>) {
        let (tx, rx) = mpsc::channel(capacity);
        let dispatcher = Self {
            factory,
            sender: tx,
            retry_intervals,
        };
        (dispatcher, rx)
    }

    pub async fn dispatch(&self, msg: Message) -> Result<(), String> {
        self.sender.send(msg).await.map_err(|e| e.to_string())
    }

    pub async fn process_message(factory: Arc<dyn StructuralFactory>, msg: Message) -> Result<(), String> {
        match msg {
            Message::Create(node) => {
                factory.create(node).await.map_err(|e| e.to_string())?;
            }
            Message::Query(id) => {
                factory.query(&id).await.map_err(|e| e.to_string())?;
            }
            Message::Update(id, props) => {
                factory.update(&id, Box::new(move |node| {
                    for (k, v) in props {
                        node.properties.insert(k, v);
                    }
                })).await.map_err(|e| e.to_string())?;
            }
            Message::Delete(id) => {
                factory.delete(&id).await.map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub async fn dispatch_with_retry<F, Fut>(&self, msg: Message, action: F) -> Result<(), String> 
    where 
        F: Fn(Message) -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        let mut attempts = 0;
        loop {
            match action(msg.clone()).await {
                Ok(_) => return Ok(()),
                Err(e) if attempts < self.retry_intervals.len() => {
                    let wait = self.retry_intervals[attempts];
                    sleep(Duration::from_millis(wait)).await;
                    attempts += 1;
                }
                Err(e) => return Err(format!("Failed after {} retries: {}", attempts, e)),
            }
        }
    }
}
