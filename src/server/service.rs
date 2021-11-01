use tokio::sync::mpsc;
use tonic::transport::Server;

use super::api::HyperionAPIService;
use crate::{plugin::Command, proto::api::hyperion_api_service_server::HyperionApiServiceServer};

pub struct Config<'a> {
    pub host: &'a str,
    pub port: &'a str,

    pub sender: mpsc::Sender<Command>,
}

/// start attaches the api services to the server and starts the server on the host
/// and port given in the function parameter
pub async fn start(cfg: Config<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let service = HyperionAPIService::new(cfg.sender);
    let server = HyperionApiServiceServer::new(service);

    let addr = format!("{}:{}", cfg.host, cfg.port).parse()?;

    println!("server listening on {}", addr);

    Server::builder().add_service(server).serve(addr).await?;

    Ok(())
}
