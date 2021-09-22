use std::{collections::HashMap, sync::Arc};

use tokio::sync::{mpsc, Mutex};

use super::{sandbox, Command, CommandResponse, CommandType};
use crate::{proto::base};

const PLUGIN_COMM_CHANNEL_SIZE: usize = 16;

pub struct Manager {
    cmd_rx: mpsc::Receiver<Command>,
    plugins: Arc<Mutex<HashMap<String, sandbox::Sandbox>>>,

    plugin_rx: mpsc::Receiver<()>,
    plugin_tx: mpsc::Sender<()>,
}

// Holds the public interface of the manager
impl Manager {
    pub fn new(receiver: mpsc::Receiver<Command>) -> Self {
        let (tx, rx) = mpsc::channel(PLUGIN_COMM_CHANNEL_SIZE);

        Manager {
            plugins: Arc::new(Mutex::new(HashMap::new())),
            cmd_rx: receiver,

            plugin_rx: rx,
            plugin_tx: tx,
        }
    }

    /// start will start put the manager in "recieve" mode which will
    /// allow the manager to listen for the commands and respond accordingly
    pub async fn start(&mut self) {
        while let Some(cmd) = self.cmd_rx.recv().await {
            let typ = cmd.typ;

            match typ {
                CommandType::Apply(module) => {}
                CommandType::Delete(core) => {}
                _ => (),
            }
        }
    }
}

// Holds the private interface of the Manager
impl Manager {
    async fn plugin_exists(&self, core: &base::ModuleCore) -> bool {
        let locked = self.plugins.lock().await;

        locked.contains_key(&Manager::generate_key(core))
    }

    /// add_plugin takes in a module and adds that plugin to the manager's plugin store
    async fn add_plugin(&self, module: base::Module) -> Result<(), sandbox::errors::InvalidModuleError> {
        // Create channel for the plugin unit and manager to communicate
        let tx = self.plugin_tx.clone();

        // Create sandbox from the module
        let sb = sandbox::Sandbox::new(module, tx)?;

        // Add the unit to the plugins store
        let mut locked = self.plugins.lock().await;
        locked.insert(Manager::generate_key(sb.info()), sb);

        Ok(())
    }

    /// get_plugin takes in the module care which is used to get the plugin data
    /// from the manager's store. The data returned is a copy of the manager's
    /// internal plugin which makes this method expensive
    async fn get_plugin(&self, core: &base::ModuleCore) -> Option<base::Module> {
        let key = Manager::generate_key(core);

        let locked = self.plugins.lock().await;
        if let Some(plugin) = locked.get(&key) {
            let module = plugin.to_module();

            return Some(module);
        }

        None
    }

    async fn delete_plugin(&self, core: &base::ModuleCore) -> Result<(), sandbox::errors::InvalidModuleError> {
        let key = Manager::generate_key(core);

        let mut locked = self.plugins.lock().await;
        let plugin = locked.remove(&key);

        Ok(())
    }

    async fn watch_plugin(&self, core: &base::ModuleCore, tx: mpsc::Sender<u8>) {

    }

    fn generate_key(core: &base::ModuleCore) -> String {
        format!("{}/{}/{}", core.namespace, core.name, core.version)
    }
}

pub mod errors {
}