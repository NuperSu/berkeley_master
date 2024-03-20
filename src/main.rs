mod slave_management;
mod time_sync;

use tokio::{net::UdpSocket, time::{sleep, Duration}};
use std::{env, error::Error, sync::Arc, collections::HashMap};
use slave_management::process_message;
use time_sync::synchronize_time;
use std::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    if args.len() < 2 {
        println!("Usage: {} [Master Node Address]", args.next().unwrap());
        return Ok(());
    }

    let master_address = args.nth(1).unwrap();
    let socket = Arc::new(UdpSocket::bind(&master_address).await?);
    let slave_nodes = Arc::new(Mutex::new(HashMap::new()));

    let slave_nodes_clone_for_messages = Arc::clone(&slave_nodes);
    let socket_clone_for_messages = Arc::clone(&socket);

    tokio::spawn(async move {
        loop {
            let mut buf = [0; 1024];
            match socket_clone_for_messages.recv_from(&mut buf).await {
                Ok((number_of_bytes, src_addr)) => {
                    let msg = String::from_utf8_lossy(&buf[..number_of_bytes]);
                    process_message(&msg, src_addr, &slave_nodes_clone_for_messages).await;
                }
                Err(e) => {
                    eprintln!("Couldn't receive a datagram: {}", e);
                }
            }
        }
    });

    let slave_nodes_clone_for_sync = Arc::clone(&slave_nodes);
    let socket_clone_for_sync = Arc::clone(&socket);

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await; // Sync interval
            synchronize_time(&slave_nodes_clone_for_sync, &socket_clone_for_sync).await;
        }
    });

    println!("Master node running on {}", master_address);
    loop {
        sleep(Duration::from_secs(3600)).await; // Keep the main task alive
    }
}
