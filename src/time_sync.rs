use tokio::net::UdpSocket;
use std::sync::{Arc, Mutex};
use serde_json::json;
use std::collections::HashMap;
use chrono::Utc;

pub async fn synchronize_time(slave_nodes: &Arc<Mutex<HashMap<String, super::slave_management::SlaveNode>>>, socket: &UdpSocket) {
    let nodes = slave_nodes.lock().unwrap();
    let mut total_offset = 0i64;
    let mut count = 0i64;
    let current_time = Utc::now().timestamp_millis();

    for node in nodes.values() {
        let node_time_offset = current_time - node.last_response;
        total_offset += node_time_offset;
        count += 1;
    }

    if count > 0 {
        let average_offset = total_offset / count;
        for node in nodes.keys() {
            let adjustment = json!({
                "type": "adjust_time",
                "adjustment": average_offset,
            }).to_string();
            if let Err(e) = socket.send_to(adjustment.as_bytes(), node).await {
                eprintln!("Error sending time adjustment to {}: {}", node, e);
            }
        }
    }
}
