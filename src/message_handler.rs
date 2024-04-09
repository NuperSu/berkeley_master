use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use crate::slave_management::{SlaveNode, manage_node_introduction, clean_up_stale_nodes};
use crate::time_sync::{process_time_report};
use serde_json::Value;

pub async fn handle_message(slave_nodes: Arc<Mutex<HashMap<String, SlaveNode>>>, msg: String, addr: String) {
    match serde_json::from_str::<Value>(&msg) {
        Ok(val) => match val["type"].as_str() {
            Some("introduce") => {
                manage_node_introduction(slave_nodes, addr).await;
            },
            Some("time_report") => {
                if let Some(time) = val["time"].as_i64() {
                    process_time_report(slave_nodes, addr, time).await;
                }
            },
            _ => eprintln!("Unknown message type"),
        },
        Err(e) => eprintln!("Failed to parse message: {}", e),
    }
}
