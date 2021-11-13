mod actor;
mod config;
mod proto;
mod server;
mod utility;
mod woduler;

use actor::Actor;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logger
    env_logger::init();
    
    // Create woduler manager
    let manager = woduler::manager::Manager::new();
    // Start the manager actor
    let manager_mailbox = manager.start();

    // Create api server
    server::start(server::Config {
        host: &Config::get_host(),
        port: &Config::get_port(),
        mailbox: manager_mailbox,
    })
    .await?;

    Ok(())
}
