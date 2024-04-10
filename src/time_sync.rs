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
        let master_time = Utc::now().timestamp_millis();
        let mut times = HashMap::<String, i64>::new();
        let mut total_offset = 0i64;
        let mut valid_responses = 0i64;

        for addr in &self.slave_addresses {
            let send_time = Utc::now().timestamp_millis();
            send_message(&self.socket, &serde_json::to_string(&TimeMessage {
                msg_type: "request_time".to_string(),
                time: None,
                adjustment: None,
            })?, addr).await.expect("Failed to send time request");

            if let Ok(msg) = receive_message(&self.socket, Duration::from_secs(5)).await {
                let receive_time = Utc::now().timestamp_millis();
                let latency = (receive_time - send_time) / 2;

                if let Ok(parsed_msg) = serde_json::from_str::<TimeMessage>(&msg) {
                    if parsed_msg.msg_type == "time_report" {
                        if let Some(time) = parsed_msg.time {
                            let adjusted_slave_time = time - latency;
                            let offset = master_time - adjusted_slave_time;
                            times.insert(addr.clone(), adjusted_slave_time);
                            total_offset += offset;
                            valid_responses += 1;
                        }
                    }
                }
            }
        }

        if valid_responses == 0 {
            // Return Ok(false) to indicate no slaves responded
            return Ok(false);
        }

        let average_offset = if valid_responses > 0 { total_offset / valid_responses } else { 0 };

        for addr in &self.slave_addresses {
            // Send the adjustment based on the average offset, but only if there's a significant difference
            if average_offset.abs() > 0 {
                send_message(&self.socket, &serde_json::to_string(&TimeMessage {
                    msg_type: "adjust_time".to_string(),
                    time: None,
                    adjustment: Some(average_offset),
                })?, &addr).await.expect("Failed to send time adjustment");
            }
        }

        // Return Ok(true) to indicate at least one slave responded and adjustments were sent
        Ok(true)
    }
}
