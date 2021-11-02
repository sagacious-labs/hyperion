use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

use crate::{proto::base, woduler::process::mail::Mail};

use super::bus::Bus;

#[derive(Clone)]
pub struct Manager {
    bus: Bus,
}

impl Manager {
    pub fn new() -> Self {
        Self { bus: Bus::new() }
    }

    /// register_module takes in reference to a module config
    /// and will return an instance of ModuleEventBus which will
    /// provide helper functions to the caller for streaming
    /// log, data, input to the event bus
    pub fn register_module(&self, md: &base::Module) -> ModuleEventBus {
        ModuleEventBus::new(
            Manager::create_log_topics(md),
            Manager::create_data_topics(md),
            Manager::create_input_topics(md),
            self.bus.clone(),
        )
    }

    fn create_log_topics(md: &base::Module) -> Vec<String> {
        match &md.metadata {
            Some(metadata) => metadata
                .labels
                .iter()
                .map(|(k, v)| format!("{}={}.log", k, v))
                .collect(),
            None => Vec::new(),
        }
    }

    fn create_data_topics(md: &base::Module) -> Vec<String> {
        match &md.metadata {
            Some(metadata) => metadata
                .labels
                .iter()
                .map(|(k, v)| format!("{}={}.data", k, v))
                .collect(),
            None => Vec::new(),
        }
    }

    fn create_input_topics(md: &base::Module) -> Vec<String> {
        match &md.spec {
            Some(spec) => {
                if let Some(base::module_spec::DataSource::Label(base::LabelSelector {
                    selector,
                })) = &spec.data_source
                {
                    selector
                        .iter()
                        .map(|(k, v)| format!("{}={}.data", k, v))
                        .collect()
                } else {
                    Vec::new()
                }
            }
            None => Vec::new(),
        }
    }
}

pub struct ModuleEventBus {
    log_topics: Option<Vec<String>>,
    data_topics: Option<Vec<String>>,
    input_topics: Option<Vec<String>>,
    bus: Bus,

    sids: Arc<Mutex<Vec<(u128, String)>>>,
}

impl ModuleEventBus {
    pub fn new(
        log_topics: Vec<String>,
        data_topics: Vec<String>,
        input_topics: Vec<String>,
        bus: Bus,
    ) -> Self {
        Self {
            log_topics: Some(log_topics),
            data_topics: Some(data_topics),
            input_topics: Some(input_topics),
            bus,

            sids: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn stream_logs(&mut self, rx: mpsc::Receiver<Mail>) {
        if let Some(topics) = self.log_topics.take() {
            Self::stream(topics, rx, self.bus.clone());
        }
    }

    pub fn stream_data(&mut self, rx: mpsc::Receiver<Mail>) {
        if let Some(topics) = self.data_topics.take() {
            Self::stream(topics, rx, self.bus.clone());
        }
    }

    pub fn recv_data(&mut self, tx: mpsc::Sender<Mail>) {
        let topics = self.input_topics.take();
        let mut bus = self.bus.clone();
        let sids = Arc::clone(&self.sids);

        tokio::spawn(async move {
            let mut rxs = Vec::new();

            if let Some(topics) = topics {
                for topic in topics.iter() {
                    let (sid, rx) = bus.subscribe(topic.to_string()).await;
                    sids.lock().await.push((sid, topic.to_string()));
                    rxs.push(rx);
                }

                for mut rx in rxs {
                    let tx = tx.clone();

                    tokio::spawn(async move {
                        while let Some(mail) = rx.recv().await {
                            tx.send(mail).await;
                        }
                    });
                }
            }
        });
    }

    pub async fn cleanup(&mut self) {
        for (sid, topic) in self.sids.lock().await.iter() {
            self.bus.unsubscribe(topic, *sid).await;
        }

        self.sids.lock().await.clear();
    }

    fn stream(topics: Vec<String>, mut rx: mpsc::Receiver<Mail>, mut bus: Bus) {
        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                for topic in topics.iter() {
                    bus.publish(topic, data.clone()).await;
                }
            }
        });
    }
}
