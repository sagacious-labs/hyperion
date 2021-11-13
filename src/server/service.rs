use tonic::transport::Server;

use super::api::HyperionAPIService;
use crate::{
    actor, proto::api::hyperion_api_service_server::HyperionApiServiceServer,
    woduler::manager::command::Command,
};

pub struct Config<'a> {
    pub host: &'a str,
    pub port: &'a str,

    pub mailbox: actor::MailBox<Command>,
}

/// start attaches the api services to the server and starts the server on the host
/// and port given in the function parameter
pub async fn start(cfg: Config<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let service = HyperionAPIService::new(cfg.mailbox);
    let server = HyperionApiServiceServer::new(service);

    let addr = format!("{}:{}", cfg.host, cfg.port).parse()?;

    log::info!("server listening on {}", addr);

    Server::builder().add_service(server).serve(addr).await?;

    Ok(())
}
