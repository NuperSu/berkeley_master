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
                }
                Err(e) => eprintln!("Sync cycle failed: {}", e),
            }
            tokio::time::sleep(Duration::from_secs(60)).await; // Sync interval
        }
    }

    async fn sync_cycle(&self) -> JsonResult<bool> {
        let master_time = Utc::now().timestamp_millis();
        let mut times = HashMap::<String, i64>::new();
        let mut latencies = HashMap::<String, i64>::new();

        for addr in &self.slave_addresses {
            let send_time = Utc::now().timestamp_millis();
            send_message(&self.socket, &serde_json::to_string(&TimeMessage {
                msg_type: "request_time".to_string(),
                time: None,
                adjustment: None,
            })?, addr).await.expect("Failed to send time request");

            if let Ok(msg) = receive_message(&self.socket, Duration::from_secs(5)).await {
                let receive_time = Utc::now().timestamp_millis();
                let latency = (receive_time - send_time) / 2; // Mid-point latency approximation

                if let Ok(parsed_msg) = serde_json::from_str::<TimeMessage>(&msg) {
                    if parsed_msg.msg_type == "time_report" {
                        if let Some(time) = parsed_msg.time {
                            // Adjust slave time by the estimated latency to account for network delay
                            times.insert(addr.clone(), time - latency);
                            latencies.insert(addr.clone(), latency);
                        }
                    }
                }
            }
        }

        if times.is_empty() {
            return Ok(false); // No slaves responded
        }

        for (addr, adjusted_slave_time) in times {
            let slave_time_adjustment = master_time - adjusted_slave_time;
            // Only send adjustment if it's significant enough considering network latency
            if slave_time_adjustment.abs() > latencies[&addr] {
                send_message(&self.socket, &serde_json::to_string(&TimeMessage {
                    msg_type: "adjust_time".to_string(),
                    time: None,
                    adjustment: Some(slave_time_adjustment),
                })?, &addr).await.expect("Failed to send time adjustment");
            }
        }

        Ok(true) // At least one slave responded and adjustments were made
    }
}
