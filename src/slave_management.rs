use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::Utc;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SlaveNode {
    pub address: String,
    pub last_response: i64,
}

pub async fn process_message(msg: &str, src_addr: SocketAddr, slave_nodes: &Arc<Mutex<HashMap<String, SlaveNode>>>) {
    let parsed_msg: Value = serde_json::from_str(msg).unwrap_or_else(|_| Value::String("unknown".to_string()));
    let mut nodes = slave_nodes.lock().unwrap();
    match parsed_msg["type"].as_str() {
        Some("time_report") => {
            let addr = src_addr.to_string();
            if !nodes.contains_key(&addr) {
                nodes.insert(addr.clone(), SlaveNode {
                    address: addr.clone(),
                    last_response: Utc::now().timestamp_millis(),
                });
            }

            if let Some(node) = nodes.get_mut(&addr) {
                node.last_response = Utc::now().timestamp_millis();
            }
        }
        _ => eprintln!("Unknown message type received"),
    }
}
