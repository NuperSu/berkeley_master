mod slave_management;
mod time_sync;
mod message_handler;

use tokio::{net::UdpSocket, sync::mpsc, time::{self, Duration}};
use std::{env, error::Error, sync::Arc};
use std::collections::HashMap;
use tokio::sync::Mutex;
use crate::slave_management::{SlaveNode, clean_up_stale_nodes};
use crate::message_handler::handle_message;
use crate::time_sync::request_time_sync;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} [Master Node Address]", args[0]);
        return Ok(());
    }

    let master_address = &args[1];
    let socket = Arc::new(UdpSocket::bind(master_address).await?);
    println!("Master node running on {}", master_address);

    let slave_nodes = Arc::new(Mutex::new(HashMap::<String, SlaveNode>::new()));

    let (tx, mut rx) = mpsc::channel(32);

    // Incoming message listener
    let socket_clone = Arc::clone(&socket);
    tokio::spawn(async move {
        listen_for_messages(socket_clone, tx).await;
    });

    // Main loop processing incoming messages and managing nodes
    let slave_nodes_clone_for_main = Arc::clone(&slave_nodes);
    let socket_clone_for_sync = Arc::clone(&socket);
    tokio::spawn(async move {
        loop {
            if let Some((addr, msg)) = rx.recv().await {
                handle_message(slave_nodes_clone_for_main.clone(), msg, addr).await;
            }
        }
    });

    // Periodic tasks: Time sync requests and node cleanup
    let slave_nodes_clone_for_periodic = Arc::clone(&slave_nodes);
    let socket_clone_for_periodic = Arc::clone(&socket);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            request_time_sync(socket_clone_for_periodic.clone(), slave_nodes_clone_for_periodic.clone()).await;
            clean_up_stale_nodes(slave_nodes_clone_for_periodic.clone()).await;
        }
    });

    loop {
        time::sleep(Duration::from_secs(3600)).await; // Keep the main task alive, practically infinite loop
    }
}

async fn listen_for_messages(socket: Arc<UdpSocket>, tx: mpsc::Sender<(String, String)>) {
    let mut buf = [0u8; 1024];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                if let Err(e) = tx.send((addr.to_string(), String::from_utf8_lossy(&buf[..len]).to_string())).await {
                    eprintln!("Failed to forward message for processing: {}", e);
                }
            },
            Err(e) => eprintln!("Failed to receive message: {}", e),
        }
    }
}
