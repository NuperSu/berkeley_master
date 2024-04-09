use tokio::net::UdpSocket;
use serde::{Serialize, Deserialize};
use serde_json::Result as JsonResult;
use std::time::Duration;
use crate::network::{send_message, receive_message};
use std::collections::HashMap;
use chrono::Utc;

#[derive(Serialize, Deserialize)]
struct TimeMessage {
    #[serde(rename = "type")]
    msg_type: String,
    time: Option<i64>,
    adjustment: Option<i64>,
}

pub struct MasterTimeSync {
    socket: UdpSocket,
    slave_addresses: Vec<String>,
}

impl MasterTimeSync {
    pub fn new(socket: UdpSocket, slave_addresses: Vec<String>) -> Self {
        Self { socket, slave_addresses }
    }

    pub async fn start_sync_process(&self) {
        loop {
            match self.sync_cycle().await {
                Ok(synced) => {
                    if synced {
                        println!("Sync cycle completed successfully.");
                    } else {
                        eprintln!("Sync cycle completed: No slave responses received.");
                    }
                },
                Err(e) => eprintln!("Sync cycle failed: {}", e),
            }
            tokio::time::sleep(Duration::from_secs(60)).await; // Sync interval
        }
    }

    async fn sync_cycle(&self) -> JsonResult<bool> {
        let mut times = HashMap::new();
        for addr in &self.slave_addresses {
            send_message(&self.socket, &serde_json::to_string(&TimeMessage {
                msg_type: "request_time".to_string(),
                time: None,
                adjustment: None,
            })?, addr).await.expect("Failed to send time request");

            if let Ok(msg) = receive_message(&self.socket, Duration::from_secs(5)).await {
                if let Ok(parsed_msg) = serde_json::from_str::<TimeMessage>(&msg) {
                    if parsed_msg.msg_type == "time_report" {
                        if let Some(time) = parsed_msg.time {
                            times.insert(addr.clone(), time);
                        }
                    }
                }
            }
        }

        if times.is_empty() {
            // Return Ok(false) to indicate no slaves responded
            return Ok(false);
        }

        let average_time = times.values().sum::<i64>() / times.len() as i64;
        let master_time = Utc::now().timestamp_millis();
        let adjustment = average_time - master_time;

        for addr in &self.slave_addresses {
            send_message(&self.socket, &serde_json::to_string(&TimeMessage {
                msg_type: "adjust_time".to_string(),
                time: None,
                adjustment: Some(adjustment),
            })?, addr).await.expect("Failed to send time adjustment");
        }

        // Return Ok(true) to indicate at least one slave responded and adjustments were sent
        Ok(true)
    }
}
