use tokio::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde_json::json;
use chrono::Utc;
use std::net::SocketAddr;
use super::slave_management::SlaveNode;

pub async fn synchronize_time(slave_nodes: &Arc<Mutex<HashMap<String, SlaveNode>>>, socket: &UdpSocket) {
    let adjustments = {
        let nodes = slave_nodes.lock().unwrap();
        let mut adjustments = Vec::new();
        let current_time = Utc::now().timestamp_millis();
        let mut total_offset = 0i64;
        let mut count = 0i64;

        for node in nodes.values() {
            let node_time_offset = current_time - node.last_response;
            total_offset += node_time_offset;
            count += 1;
        }

        if count > 0 {
            let average_offset = total_offset / count;
            for (addr, _) in nodes.iter() {
                adjustments.push((addr.clone(), average_offset));
            }
        }
        adjustments
    };

    for (addr, adjustment) in adjustments {
        let message = json!({
            "type": "adjust_time",
            "adjustment": adjustment,
        }).to_string();

        match addr.parse::<SocketAddr>() {
            Ok(socket_addr) => {
                if let Err(e) = socket.send_to(&message.as_bytes(), &socket_addr).await {
                    eprintln!("Error sending time adjustment to {}: {}", addr, e);
                } else {
                    println!("Successfully sent time adjustment to {}: {}", addr, message);
                }
            },
            Err(_) => eprintln!("Failed to parse address: {}", addr),
        }
    }
}
