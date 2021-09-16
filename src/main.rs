use hyperion::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::start("0.0.0.0", "10000").await?;

    Ok(())
}
