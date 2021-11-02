#![allow(dead_code, unused_variables)]

use actor::Actor;
mod actor;
mod proto;
mod server;
mod utility;
mod woduler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create woduler manager
    let manager = woduler::manager::Manager::new();
    // Start the manager actor
    let manager_mailbox = manager.start();

    // Create api server
    server::start(server::Config {
        host: "0.0.0.0",
        port: "2310",
        mailbox: manager_mailbox,
    })
    .await?;

    Ok(())
}
