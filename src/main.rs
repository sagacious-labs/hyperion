pub mod server;
pub mod proto;
pub mod actor;
pub mod plugin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::start("0.0.0.0", "10000").await?;

    Ok(())
}
