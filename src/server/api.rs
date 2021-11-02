use tokio::sync::mpsc;
// use futures_core::Stream;
// use futures_util::StreamExt;
// use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::{
    actor,
    proto::api::{
        hyperion_api_service_server::HyperionApiService as HyperionAPI, ApplyRequest,
        ApplyResponse, DeleteRequest, DeleteResponse, GetRequest, GetResponse, ListRequest,
        WatchDataRequest, WatchDataResponse,
    },
    woduler::manager::command::Command,
};

pub struct HyperionAPIService {
    mailbox: actor::MailBox<Command>,
}

#[tonic::async_trait]
impl HyperionAPI for HyperionAPIService {
    async fn apply(
        &self,
        _request: Request<ApplyRequest>,
    ) -> Result<Response<ApplyResponse>, Status> {
        todo!()
    }

    async fn delete(
        &self,
        _request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        todo!()
    }

    type ListStream = ReceiverStream<Result<GetResponse, Status>>;

    async fn list(
        &self,
        _request: Request<ListRequest>,
    ) -> Result<Response<Self::ListStream>, Status> {
        todo!()
    }

    async fn get(&self, _request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        todo!()
    }

    type WatchDataStream = ReceiverStream<Result<WatchDataResponse, Status>>;

    async fn watch_data(
        &self,
        _request: Request<WatchDataRequest>,
    ) -> Result<Response<Self::WatchDataStream>, Status> {
        todo!()
    }
}

impl HyperionAPIService {
    pub fn new(mailbox: actor::MailBox<Command>) -> Self {
        HyperionAPIService { mailbox }
    }
}
