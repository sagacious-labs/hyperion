use tokio::sync::mpsc;

pub struct Runner<T>
{
    rx: mpsc::Receiver<T>,
}

impl<T> Runner<T> {
    pub fn new(rx: mpsc::Receiver<T>) -> Self {
        Self { rx }
    }

    pub async fn run<U>(&mut self, handler: U)
    where
        U: Handler<RxData = T>
    {
        while let Some(msg) = self.rx.recv().await {
            handler.handle(msg);
        }
    }
}

pub trait Handler: Sync + Send {
    type RxData;

    fn handle(&self, msg: Self::RxData) {
        // Drop the message by default
    }
}
