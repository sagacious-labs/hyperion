use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::actor::Actor;
use crate::proto::base;
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
    fn handle_apply(&self, md: base::Module) -> Result<()> {
        let key = utility::module_core_key(&md)?;
        let modules = Arc::clone(&self.modules);
        let em = self.event_manager.clone();

        tokio::spawn(async move {
            let mut locked = modules.lock().await;
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
                    let meb = em.register_module(&md);

                    // Start the process controller with given module event bus
                    pc.run(&md, meb);

                    // Save this controller
                    locked.insert(key, (md, pc));
                }
                None => {
                    // Create a new process controller
                    let mut pc = ProcessController::new();

                    // Create new module event bus for the controller
                    let meb = em.register_module(&md);

                    // Start the process controller with given module event bus
                    pc.run(&md, meb);

                    // Save this controller
                    locked.insert(key, (md, pc));
                }
            }
        });

        Ok(())
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
        match msg {
            command::Command::Apply(md, res) => {
                let m = self.clone();
                tokio::spawn(async move {
                    m.handle_apply(md);
                    res.send("done".to_owned());
                });
            }
            command::Command::Delete(md, res) => {
                todo!()
            }
            _ => {}
        }
    }
}

pub mod command {
    use tokio::sync::{mpsc, oneshot};

    pub enum Command {
        Apply(super::base::Module, oneshot::Sender<String>),
        Delete(super::base::ModuleCore, oneshot::Sender<String>),
        List(Filter, mpsc::Sender<super::base::Module>),
        Get(
            super::base::ModuleCore,
            oneshot::Sender<super::base::Module>,
        ),
        WatchData(Filter, mpsc::Sender<Vec<u8>>),
        WatchLog(Filter, mpsc::Sender<Vec<u8>>),
    }

    #[derive(Clone)]
    pub enum Filter {
        Label(super::base::LabelSelector),
        Core(super::base::ModuleCore),
    }
}
