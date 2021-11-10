use tokio::sync::{mpsc, oneshot};
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
        WatchDataRequest, WatchDataResponse, WatchLogRequest, WatchLogResponse,
    },
    woduler::manager::command::{self, Command},
};

pub struct HyperionAPIService {
    mailbox: actor::MailBox<Command>,
}

#[tonic::async_trait]
impl HyperionAPI for HyperionAPIService {
    async fn apply(
        &self,
        request: Request<ApplyRequest>,
    ) -> Result<Response<ApplyResponse>, Status> {
        let req = request.into_inner();

        if let Some(module) = req.module {
            let (tx, rx) = oneshot::channel();
            if let Err(e) = self.mailbox.mail(command::Command::Apply(module, tx)).await {
                return Err(tonic::Status::new(
                    tonic::Code::Internal,
                    "failed to proceess the request",
                ));
            }

            match rx.await {
                Ok(res) => match res {
                    Ok(v) => {
                        return Ok(Response::new(ApplyResponse { msg: v }));
                    }
                    Err(err) => {
                        return Err(tonic::Status::new(tonic::Code::Internal, err.to_string()))
                    }
                },
                Err(e) => {
                    return Err(tonic::Status::new(
                        tonic::Code::Internal,
                        "failed to process the request",
                    ));
                }
            }
        }

        Err(tonic::Status::new(
            tonic::Code::FailedPrecondition,
            "invalid request",
        ))
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let req = request.into_inner();

        if let Some(core) = req.core {
            let (tx, rx) = oneshot::channel();
            if let Err(e) = self.mailbox.mail(command::Command::Delete(core, tx)).await {
                return Err(tonic::Status::new(
                    tonic::Code::Internal,
                    "failed to proceess the request",
                ));
            }

            match rx.await {
                Ok(v) => match v {
                    Ok(msg) => {
                        return Ok(Response::new(DeleteResponse { msg }));
                    }
                    Err(err) => {
                        return Err(tonic::Status::new(tonic::Code::Internal, err.to_string()));
                    }
                },
                Err(e) => {
                    return Err(tonic::Status::new(
                        tonic::Code::Internal,
                        "failed to process the request",
                    ));
                }
            }
        }

        Err(tonic::Status::new(
            tonic::Code::FailedPrecondition,
            "invalid request",
        ))
    }

    type ListStream = ReceiverStream<Result<GetResponse, Status>>;

    async fn list(
        &self,
        request: Request<ListRequest>,
    ) -> Result<Response<Self::ListStream>, Status> {
        let req = request.into_inner();

        if let Some(filter) = req.filter {
            let (tx, mut rx) = mpsc::channel(8);
            if let Err(e) = self.mailbox.mail(command::Command::List(filter, tx)).await {
                return Err(tonic::Status::new(
                    tonic::Code::Internal,
                    "failed to proceess the request",
                ));
            }

            let (rtx, rrx) = mpsc::channel(8);
            tokio::spawn(async move {
                while let Some(res) = rx.recv().await {
                    if let Err(err) = rtx.send(Ok(GetResponse { module: Some(res) })).await {
                        println!("failed to pipe data to the output stream: {}", err);
                        break;
                    }
                }
            });

            return Ok(Response::new(ReceiverStream::new(rrx)));
        }

        Err(tonic::Status::new(
            tonic::Code::FailedPrecondition,
            "invalid request",
        ))
    }

    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let req = request.into_inner();

        if let Some(core) = req.core {
            let (tx, rx) = oneshot::channel();
            if let Err(e) = self.mailbox.mail(command::Command::Get(core, tx)).await {
                return Err(tonic::Status::new(
                    tonic::Code::Internal,
                    "failed to proceess the request",
                ));
            }

            match rx.await {
                Ok(v) => match v {
                    Ok(res) => {
                        return Ok(Response::new(GetResponse { module: Some(res) }));
                    }
                    Err(err) => {
                        return Err(tonic::Status::new(tonic::Code::Internal, err.to_string()));
                    }
                },
                Err(e) => {
                    return Err(tonic::Status::new(
                        tonic::Code::Internal,
                        "failed to process the request",
                    ));
                }
            }
        }

        Err(tonic::Status::new(
            tonic::Code::FailedPrecondition,
            "invalid request",
        ))
    }

    type WatchDataStream = ReceiverStream<Result<WatchDataResponse, Status>>;

    async fn watch_data(
        &self,
        request: Request<WatchDataRequest>,
    ) -> Result<Response<Self::WatchDataStream>, Status> {
        let req = request.into_inner();

        if let Some(filter) = req.filter {
            let (tx, mut rx) = mpsc::channel(8);
            if let Err(e) = self
                .mailbox
                .mail(command::Command::WatchData(filter, tx))
                .await
            {
                return Err(tonic::Status::new(
                    tonic::Code::Internal,
                    "failed to proceess the request",
                ));
            }

            let (rtx, rrx) = mpsc::channel(8);
            tokio::spawn(async move {
                while let Some(res) = rx.recv().await {
                    if let Err(err) = rtx.send(Ok(WatchDataResponse { data: res })).await {
                        println!("failed to pipe data to the output stream: {}", err);
                        break;
                    }
                }
            });

            return Ok(Response::new(ReceiverStream::new(rrx)));
        }

        Err(tonic::Status::new(
            tonic::Code::FailedPrecondition,
            "invalid request",
        ))
    }

    type WatchLogStream = ReceiverStream<Result<WatchLogResponse, Status>>;

    async fn watch_log(
        &self,
        request: Request<WatchLogRequest>,
    ) -> Result<Response<Self::WatchLogStream>, Status> {
        let req = request.into_inner();

        if let Some(filter) = req.filter {
            let (tx, mut rx) = mpsc::channel(8);
            if let Err(e) = self
                .mailbox
                .mail(command::Command::WatchLog(filter, tx))
                .await
            {
                return Err(tonic::Status::new(
                    tonic::Code::Internal,
                    "failed to proceess the request",
                ));
            }

            let (rtx, rrx) = mpsc::channel(8);
            tokio::spawn(async move {
                while let Some(res) = rx.recv().await {
                    if let Err(err) = rtx.send(Ok(WatchLogResponse { data: res })).await {
                        println!("failed to pipe data to the output stream: {}", err);
                        break;
                    }
                }
            });

            return Ok(Response::new(ReceiverStream::new(rrx)));
        }

        Err(tonic::Status::new(
            tonic::Code::FailedPrecondition,
            "invalid request",
        ))
    }
}

impl HyperionAPIService {
    pub fn new(mailbox: actor::MailBox<Command>) -> Self {
        HyperionAPIService { mailbox }
    }
}
