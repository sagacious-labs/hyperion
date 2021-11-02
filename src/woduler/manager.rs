use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::actor::Actor;
use crate::proto::{api, base};
use crate::utility;

use super::event;
use super::process_controller::ProcessController;

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
    async fn handle_apply(&mut self, md: base::Module, ch: oneshot::Sender<String>) {
        let key = utility::module_core_key(&md);
        if let Err(err) = &key {
            ch.send(err.to_string());
            return;
        }
        let key = key.ok().take().unwrap();

        let mut locked = self.modules.lock().await;
        match locked.get(&key) {
            Some((module, controller)) => {
                // Dumb implementation for now - No matter what, delete older version and load another

                // Stop the previous controller
                controller.stop().await;

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

        ch.send(format!("applied {}", key));
    }

    async fn handle_delete(&mut self, md: base::ModuleCore, ch: oneshot::Sender<String>) {
        let key = md.name.clone();

        let mut modules = self.modules.lock().await;

        // Delete the module from the store
        if let Some((module, pc)) = modules.remove(&key) {
            // Instruct the process controller to shut down the process
            pc.stop().await;
        }

        ch.send(format!("deleted {}", md.name));
    }

    async fn handle_list(&self, filter: api::list_request::Filter, ch: mpsc::Sender<base::Module>) {
        match filter {
            api::list_request::Filter::Core(core) => {
                let key = core.name;

                let modules = self.modules.lock().await;
                if let Some((module, _)) = modules.get(&key) {
                    ch.send(module.clone()).await;
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

                    if matches == label.selector.len() {
                        ch.send(module.clone()).await;
                    }
                }
            }
        }
    }

    async fn handle_get(&self, core: base::ModuleCore, ch: oneshot::Sender<base::Module>) {
        let key = core.name;
        let modules = self.modules.lock().await;

        if let Some((module, _)) = modules.get(&key) {
            ch.send(module.clone());
        }
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
                _ => {}
            }
        });
    }
}

pub mod command {
    use tokio::sync::{mpsc, oneshot};

    pub enum Command {
        Apply(super::base::Module, oneshot::Sender<String>),
        Delete(super::base::ModuleCore, oneshot::Sender<String>),
        List(
            super::api::list_request::Filter,
            mpsc::Sender<super::base::Module>,
        ),
        Get(
            super::base::ModuleCore,
            oneshot::Sender<super::base::Module>,
        ),
        WatchData(super::api::list_request::Filter, mpsc::Sender<Vec<u8>>),
        WatchLog(super::api::list_request::Filter, mpsc::Sender<Vec<u8>>),
    }
}
