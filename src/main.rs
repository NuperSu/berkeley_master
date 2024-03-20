mod slave_management;
mod time_sync;

use tokio::{net::UdpSocket, time::{sleep, Duration}};
use std::{env, error::Error, sync::{Arc, Mutex}};
use slave_management::SlaveNode;
use time_sync::synchronize_time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} [Master Node Address] [Slave Node Addresses...]", args[0]);
        return Ok(());
    }

    let master_address = &args[1];
    let slave_addresses = &args[2..];
    println!("Master node starting. Binding to address: {}", master_address);

    let socket = Arc::new(UdpSocket::bind(master_address).await?);
    let slave_nodes = Arc::new(Mutex::new(std::collections::HashMap::new()));

    // Initialize slave_nodes with provided slave addresses
    {
        let mut nodes = slave_nodes.lock().unwrap();
        for addr in slave_addresses {
            nodes.insert(addr.to_string(), SlaveNode {
                address: addr.to_string(),
                last_response: 0, // Initial timestamp could be 0 or current time
            });
        }
    }

    let slave_nodes_clone_for_sync = Arc::clone(&slave_nodes);
    let socket_clone_for_sync = Arc::clone(&socket);

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await; // Sync interval
            println!("Initiating time synchronization process.");
            synchronize_time(&slave_nodes_clone_for_sync, &socket_clone_for_sync).await;
        }
    });

    println!("Master node running on {}", master_address);
    loop {
        sleep(Duration::from_secs(3600)).await; // Keep the main task alive
    }
}
