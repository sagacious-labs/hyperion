use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::actor::Actor;
use crate::proto::{api, base};
use crate::utility;

use super::event;
use super::process::Controller as ProcessController;

/// Manager is an actor and exposes the API of woduler
/// to other parts of Hyperion
///
/// Manager also has the responsibility of maintaing the
/// lifecycle of the modules and factitilating communication
/// between them by leveraging the Event Manager
pub struct Manager {
    event_manager: event::Manager,
    modules: Arc<Mutex<HashMap<String, (base::Module, ProcessController)>>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            event_manager: event::Manager::new(),
            modules: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn handle_apply(&mut self, mut md: base::Module, ch: oneshot::Sender<Result<String>>) {
        let key = utility::module_core_key(&md);
        if let Err(err) = &key {
            if ch.send(Err(anyhow::anyhow!("{}", err))).is_err() {
                println!("failed to send data to caller");
            }
            return;
        }
        let key = key.ok().take().unwrap();

        if let Err(err) = Self::setup_defaults(&mut md, key.clone()) {
            if let Err(err) = ch.send(Err(err)) {
                println!("failed to send data to caller");
            }
            return;
        }

        let mut locked = self.modules.lock().await;
        match locked.get(&key) {
            Some((module, controller)) => {
                // Dumb implementation for now - No matter what, delete older version and load another

                // Stop the previous controller
                controller.stop();

                // Drop the controller - hence clearing up the resources acquired by it
                locked.remove(&key);

                // Create a new process controller
                let mut pc = ProcessController::new();

                // Create new module event bus for the controller
                let meb = self.event_manager.register_module(&md);

                // Start the process controller with given module event bus
                pc.run(&md, meb);

                // Save this controller
                locked.insert(key.clone(), (md, pc));
            }
            None => {
                // Create a new process controller
                let mut pc = ProcessController::new();

                // Create new module event bus for the controller
                let meb = self.event_manager.register_module(&md);

                // Start the process controller with given module event bus
                pc.run(&md, meb);

                // Save this controller
                locked.insert(key.clone(), (md, pc));
            }
        }

        if ch.send(Ok(format!("applied {}", key))).is_err() {
            println!("failed to send data to caller");
        }
    }

    async fn handle_delete(&mut self, md: base::ModuleCore, ch: oneshot::Sender<Result<String>>) {
        let key = md.name.clone();

        let mut modules = self.modules.lock().await;

        // Delete the module from the store
        if let Some((module, pc)) = modules.remove(&key) {
            // Instruct the process controller to shut down the process
            pc.stop();

            if ch.send(Ok(format!("deleted {}", key))).is_err() {
                println!("failed to send data to caller")
            }

            return;
        }

        if ch.send(Err(anyhow!("{} not found", key))).is_err() {
            println!("failed to send data to caller")
        }
    }

    async fn handle_list(&self, filter: api::list_request::Filter, ch: mpsc::Sender<base::Module>) {
        match filter {
            api::list_request::Filter::Core(core) => {
                let key = core.name;

                let modules = self.modules.lock().await;
                if let Some((module, _)) = modules.get(&key) {
                    if let Err(err) = ch.send(module.clone()).await {
                        println!("failed to send data to caller: {}", err)
                    }
                }
            }
            api::list_request::Filter::Label(label) => {
                let modules = self.modules.lock().await;

                for (_, (module, _)) in modules.iter() {
                    let mut matches = 0;

                    for (k, v) in &label.selector {
                        if let Some(m) = &module.metadata {
                            if let Some(val) = m.labels.get(k) {
                                if val == v {
                                    matches += 1;
                                }
                            }
                        }
                    }

                    if matches == label.selector.len() && ch.send(module.clone()).await.is_err() {
                        println!("failed to send data to caller")
                    }
                }
            }
        }
    }

    async fn handle_get(&self, core: base::ModuleCore, ch: oneshot::Sender<Result<base::Module>>) {
        let key = core.name;
        let modules = self.modules.lock().await;

        if let Some((module, pc)) = modules.get(&key) {
            let mut module = module.clone();
            module.status = Some(base::ModuleStatus {
                msg: pc.get_status().await,
            });

            if ch.send(Ok(module)).is_err() {
                println!("failed to send data to the caller")
            }

            return;
        }

        if ch
            .send(Err(anyhow::anyhow!(
                "module with key \"{}\" not found",
                key
            )))
            .is_err()
        {
            println!("failed to send data to the caller");
        }
    }

    async fn handle_watch_data(&mut self, core: base::ModuleCore, ch: mpsc::Sender<Vec<u8>>) {
        let key = core.name;
        let topic = event::Manager::generate_topic("data", "core.hyperion.io/app", &key);
        let mut bus = self.event_manager.bus().clone();

        let (sid, mut recv) = bus.subscribe(topic.clone()).await;

        tokio::spawn(async move {
            // Keep listening to the data coming from the bus
            while let Some(data) = recv.recv().await {
                if ch.send(data.data).await.is_err() {
                    println!("failed to send data to the caller - will closing subscription");
                    break;
                }
            }

            // Close the subscription
            bus.unsubscribe(&topic, sid).await;
        });
    }

    async fn handle_watch_log(&mut self, core: base::ModuleCore, ch: mpsc::Sender<Vec<u8>>) {
        let key = core.name;
        let topic = event::Manager::generate_topic("log", "core.hyperion.io/app", &key);
        let mut bus = self.event_manager.bus().clone();

        let (sid, mut recv) = bus.subscribe(topic.clone()).await;

        tokio::spawn(async move {
            // Keep listening to the data coming from the bus
            while let Some(data) = recv.recv().await {
                if ch.send(data.data).await.is_err() {
                    println!("failed to send data to the caller - will closing subscription");
                    break;
                }
            }

            // Close the subscription
            bus.unsubscribe(&topic, sid).await;
        });
    }

    fn setup_defaults(md: &mut base::Module, name: String) -> Result<()> {
        let mdc = md.clone();

        match &mut md.metadata {
            Some(metadata) => {
                let (k, v) = Self::get_module_name_label(&mdc);
                metadata.labels.insert(k, v);
            }
            None => {
                return Err(anyhow!("invalid module - metadata cannot be empty"));
            }
        }

        Ok(())
    }

    fn get_module_name_label(md: &base::Module) -> (String, String) {
        let res = utility::module_core_key(md).unwrap();

        ("core.hyperion.io/app".to_string(), res)
    }
}

impl Clone for Manager {
    fn clone(&self) -> Self {
        Self {
            event_manager: self.event_manager.clone(),
            modules: Arc::clone(&self.modules),
        }
    }
}

impl Actor for Manager {
    type RxData = command::Command;

    fn handle(&self, msg: Self::RxData) {
        let mut m = self.clone();

        tokio::spawn(async move {
            match msg {
                command::Command::Apply(md, res) => {
                    m.handle_apply(md, res).await;
                }
                command::Command::Delete(md, res) => {
                    m.handle_delete(md, res).await;
                }
                command::Command::List(filter, res) => {
                    m.handle_list(filter, res).await;
                }
                command::Command::Get(core, res) => {
                    m.handle_get(core, res).await;
                }
                command::Command::WatchData(core, res) => {
                    m.handle_watch_data(core, res).await;
                }
                command::Command::WatchLog(core, res) => {
                    m.handle_watch_log(core, res).await;
                }
                _ => {}
            }
        });
    }
}

pub mod command {
    use tokio::sync::{mpsc, oneshot};

    pub enum Command {
        Apply(super::base::Module, oneshot::Sender<anyhow::Result<String>>),
        Delete(
            super::base::ModuleCore,
            oneshot::Sender<anyhow::Result<String>>,
        ),
        List(
            super::api::list_request::Filter,
            mpsc::Sender<super::base::Module>,
        ),
        Get(
            super::base::ModuleCore,
            oneshot::Sender<anyhow::Result<super::base::Module>>,
        ),
        WatchData(super::base::ModuleCore, mpsc::Sender<Vec<u8>>),
        WatchLog(super::base::ModuleCore, mpsc::Sender<Vec<u8>>),
    }
}
