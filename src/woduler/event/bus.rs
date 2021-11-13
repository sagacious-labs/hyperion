use std::{collections::HashMap, sync::Arc};

use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::woduler::process::mail::Mail;

type DataChannel = mpsc::Sender<Mail>;
type TopicSubscriber = HashMap<u128, DataChannel>;

pub struct Bus {
    subscribers: Arc<Mutex<HashMap<String, TopicSubscriber>>>,
}

impl Bus {
    /// new returns a new instance of the event bus
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// subscribe takes in a topic and returns a subscription id and message receiver
    pub async fn subscribe(&mut self, topic: String) -> (u128, mpsc::Receiver<Mail>) {
        // Create ID for this subscription
        let id = Uuid::new_v4().as_u128();

        // Create channel for the subscription
        let (tx, rx) = mpsc::channel(8);

        let mut locked = self.subscribers.lock().await;
        match locked.get_mut(&topic) {
            Some(subs_grp) => {
                subs_grp.insert(id, tx);
            }
            None => {
                let mut map = HashMap::new();
                map.insert(id, tx);

                locked.insert(topic, map);
            }
        }

        (id, rx)
    }

    /// publish takes in a topic and data and sends the data to all of the subscribers of that topic
    pub async fn publish(&mut self, topic: &str, data: Mail) {
        let mut locked = self.subscribers.lock().await;
        if let Some(subs_grp) = locked.get_mut(topic) {
            for (_, v) in subs_grp.iter() {
                let tx = v.clone();
                let data = data.clone();

                // Don't block the publish because of a slow consumer
                tokio::spawn(async move {
                    if tx.send(data).await.is_err() {
                        log::warn!("failed to send message to subscriber");
                    }
                });
            }
        }
    }

    /// unsubscribe takes in a topic and the subscriber ID and removes that subscriber from the
    /// subscribed list
    pub async fn unsubscribe(&mut self, topic: &str, sub_id: u128) {
        let mut locked = self.subscribers.lock().await;
        let mut abandoned_topic = false;

        if let Some(subs_grp) = locked.get_mut(topic) {
            subs_grp.remove(&sub_id);
            abandoned_topic = subs_grp.is_empty();
        }

        // If there are no subscribers for this topic then delete the entry to prevent memory leakage
        if abandoned_topic {
            locked.remove(topic);
        }
    }
}

impl Clone for Bus {
    fn clone(&self) -> Self {
        Self {
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}
