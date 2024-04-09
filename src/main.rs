use tokio::net::UdpSocket;
use std::env;
use std::error::Error;

mod time_sync;
mod network;

use time_sync::MasterTimeSync;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <Master Bind Address> <Slave Node Address>...", args[0]);
        std::process::exit(1);
    }

    let master_address = &args[1];
    let slave_addresses = args[2..].to_vec();

    println!("Master node binding to: {}", master_address);
    let socket = UdpSocket::bind(master_address).await?;
    println!("Master node bound to {}", socket.local_addr()?);

    let master_sync = MasterTimeSync::new(socket, slave_addresses);
    master_sync.start_sync_process().await;

    Ok(())
}
