use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct SlaveNode {
    pub address: String,
    pub request_time: i64, // Unix timestamp of the last request
    pub last_response: i64, // Unix timestamp of the last communication
}

impl SlaveNode {
    pub fn new(address: String) -> Self {
        Self {
            address,
            request_time: Utc::now().timestamp(),
            last_response: Utc::now().timestamp(),
        }
    }

    pub fn update_last_response(&mut self) {
        self.last_response = Utc::now().timestamp();
    }

    pub fn update_request_time(&mut self) {
        self.request_time = Utc::now().timestamp_millis();
    }
}

pub async fn manage_node_introduction(slave_nodes: Arc<Mutex<HashMap<String, SlaveNode>>>, address: String) {
    let mut nodes = slave_nodes.lock().await;
    nodes.entry(address.clone())
        .or_insert_with(|| SlaveNode::new(address))
        .update_last_response();
}

pub async fn clean_up_stale_nodes(slave_nodes: Arc<Mutex<HashMap<String, SlaveNode>>>) {
    let mut nodes = slave_nodes.lock().await;
    nodes.retain(|_, v| Utc::now().timestamp() - v.last_response < 60);
}
