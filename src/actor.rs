use tokio::sync::mpsc;

pub const BUFFER_SIZE: usize = 16;

pub trait Actor: Sized + Send + 'static {
    type RxData: Send;

    fn start(self) -> MailBox<Self::RxData> {
        let (tx, rx) = mpsc::channel(BUFFER_SIZE);
        let mut runner = Runner::<Self::RxData>::new(rx);

        tokio::spawn(async move {
            runner.run(self).await;
        });

        MailBox::new(tx)
    }

    fn handle(&self, msg: Self::RxData) {
        // Drop the message by default
    }
}

pub struct Runner<T> {
    rx: mpsc::Receiver<T>,
}

impl<T> Runner<T> {
    pub fn new(rx: mpsc::Receiver<T>) -> Self {
        Self { rx }
    }

    pub async fn run<U>(&mut self, handler: U)
    where
        U: Actor<RxData = T>,
    {
        while let Some(msg) = self.rx.recv().await {
            handler.handle(msg);
        }
    }
}

#[derive(Clone)]
pub struct MailBox<T> {
    tx: mpsc::Sender<T>,
}

impl<T> MailBox<T> {
    pub fn new(tx: mpsc::Sender<T>) -> Self {
        Self { tx }
    }

    pub async fn mail(&self, msg: T) -> Result<(), error::MailError> {
        self.tx.send(msg).await.or(Err(error::MailError))?;

        Ok(())
    }
}

pub mod error {
    #[derive(Debug)]
    pub struct MailError;

    impl std::fmt::Display for MailError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "failed to send")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod basic {
        use tokio::sync::oneshot;

        use super::*;

        struct Temp;

        impl Actor for Temp {
            type RxData = (String, oneshot::Sender<String>);

            fn handle(&self, msg: Self::RxData) {
                let (data, chan) = msg;
                chan.send(data).unwrap();
            }
        }

        #[tokio::test]
        async fn test_actor() {
            let item = Temp {};

            let (tx, rx) = oneshot::channel();

            item.start().mail(("echo".to_owned(), tx)).await.unwrap();

            assert_eq!(rx.await.unwrap(), "echo".to_owned());
        }
    }
}