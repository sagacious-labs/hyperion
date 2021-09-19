mod api;

use tonic::transport::Server;

use crate::proto::api as hyperion_api;
use api::HyperionAPIService;
use hyperion_api::hyperion_api_service_server::HyperionApiServiceServer;

pub async fn start(host: &str, port: &str) -> Result<(), Box<dyn std::error::Error>> {
    let service = HyperionAPIService::new();
    let server = HyperionApiServiceServer::new(service);

    let addr = format!("{}:{}", host, port).parse()?;

    println!("server listening on {}", addr);

    Server::builder().add_service(server).serve(addr).await?;

    Ok(())
}