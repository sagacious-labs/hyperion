use tokio::sync::mpsc::{self, error::SendError};

use super::runner;

pub const BUFFER_SIZE: usize = 16;

#[derive(Clone)]
pub struct Actor<T>
where
    T: Send + Sync + 'static,
{
    tx: mpsc::Sender<T>,
}

impl<T> Actor<T>
where
    T: Send + Sync + 'static,
{
    /// `new` will create an actor "container" for the `entity`
    /// passed as the parameter.
    ///
    /// It will interally spin up an async task in a `runner`
    pub fn new<U>(entity: U) -> Self
    where
        U: runner::Handler<RxData = T> + 'static,
    {
        let (tx, rx) = mpsc::channel(BUFFER_SIZE);
        let mut runner = runner::Runner::<T>::new(rx);

        tokio::spawn(async move {
            runner.run(entity).await;
        });

        Self { tx }
    }

    /// `send` will take in a message and will send it to the entity running
    /// inside the actor's runner
    pub async fn send(&self, data: T) -> Result<(), SendError<T>> {
        self.tx.send(data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod basic {
        use tokio::sync::oneshot;

        use super::*;

        struct Temp;

        impl runner::Handler for Temp {
            type RxData = (String, oneshot::Sender<String>);

            fn handle(&self, msg: Self::RxData) {
                let (data, chan) = msg;
                chan.send(data).unwrap();
            }
        }

        #[tokio::test]
        async fn test_actor() {
            let item = Temp {};

            let item_actor = Actor::new(item);

            let (tx, rx) = oneshot::channel();

            item_actor.send(("echo".to_owned(), tx)).await.unwrap();

            assert_eq!(rx.await.unwrap(), "echo".to_owned());
        }
    }
}
