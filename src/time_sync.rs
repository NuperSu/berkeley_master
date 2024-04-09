use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use std::collections::HashMap;
use chrono::Utc;
use crate::slave_management::SlaveNode;
use serde_json::{json, Value};

pub async fn request_time_sync(socket: Arc<UdpSocket>, slave_nodes: Arc<Mutex<HashMap<String, SlaveNode>>>) {
    let message = json!({
        "type": "request_time"
    }).to_string();

    let mut nodes = slave_nodes.lock().await;
    let current_time = Utc::now().timestamp_millis();
    for node in nodes.values_mut() {
        node.update_request_time(); // Update the time when we're sending the request
        if let Err(e) = socket.send_to(&message.as_bytes(), &node.address).await {
            eprintln!("Error sending time request to {}: {}", node.address, e);
        }
    }
}

pub async fn process_time_report(slave_nodes: Arc<Mutex<HashMap<String, SlaveNode>>>, addr: String, reported_time: i64) {
    let mut nodes = slave_nodes.lock().await;
    if let Some(node) = nodes.get_mut(&addr) {
        let current_time = Utc::now().timestamp_millis();
        let rtt = current_time - node.request_time; // Calculate round-trip time
        let estimated_slave_time = reported_time + (rtt / 2); // Estimate slave's time
        // Here you can compare the estimated_slave_time with the master's time and decide whether an adjustment is needed
    }
}
