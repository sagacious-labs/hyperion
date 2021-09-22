use tokio::sync::mpsc;

pub mod actor;
pub mod plugin;
pub mod proto;
pub mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create channels for facitilating communication between plugin manager
    // and the gRPC server
    let (tx, rx) = mpsc::channel(100);

    server::start(server::Config {
        host: "0.0.0.0",
        port: "10000",
        sender: tx,
    })
    .await?;

    Ok(())
}
