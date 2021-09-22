use std::future::Future;
use tokio::sync::mpsc::error::SendError;
use tokio::task::JoinHandle;
use tokio::{spawn, sync::mpsc};

/// Actor struct is intended for implementing a generic actor
/// model in the project
pub struct Actor<T>
where
    T: Sync + Send,
{
    receiver: Option<mpsc::Receiver<T>>,
    sender: Option<mpsc::Sender<T>>,
}

impl<T> Actor<T>
where
    T: Sync + Send + 'static,
{
    /// new will return a new instance of Actor struct
    pub fn new(buffer: usize) -> Self {
        let (tx, rx) = mpsc::channel::<T>(buffer);

        Self {
            receiver: Some(rx),
            sender: Some(tx),
        }
    }

	/// rx is a private function it returns the internal receiver
	/// channel if it is not `None`
    fn rx(&mut self) -> Result<mpsc::Receiver<T>, errors::RxError> {
        match self.receiver.take() {
			Some(rx) => Ok(rx),
			None => Err(errors::RxError),
		}
    }

    /// act spawns an asynchronous task internally and returns a join handler
    /// to the task
    ///
    /// The async task listens for messages received on the mpsc channel and invokes
    /// the async function passed as a parameter to the act function
    pub fn act<U, F>(&mut self, func: F) -> Result<JoinHandle<()>, errors::RxError>
    where
        U: Future<Output = ()> + Send,
        F: Sync + Send + 'static + Fn(T) -> U,
    {
        let mut rx = self.rx()?;
        Ok(spawn(async move {
            while let Some(message) = rx.recv().await {
                func(message).await;
            }
        }))
    }

    /// handle takes in a message of generic type T and sends the message to
    /// the mpsc channel
    pub async fn handle(&self, msg: T) -> Result<(), SendError<T>> {
        if let Some(sender) = &self.sender {
            let sender = sender.clone();

            sender.send(msg).await?;
        }

        Ok(())
    }
}

pub mod errors {
    #[derive(Debug, Clone)]
    pub struct RxError;

    impl std::fmt::Display for RxError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "receiver does not exists, either it has already been taken or dropped")
        }
    }
}
