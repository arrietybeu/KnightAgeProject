mod network;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("===========================================");
    println!("     Knight Age Server - Starting...       ");
    println!("===========================================");

    network::start_net_work().await?;
    Ok(())
}



