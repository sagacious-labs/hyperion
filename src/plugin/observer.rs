use std::{collections::{HashMap, HashSet}, sync::Arc};

use tokio::sync::{mpsc, Mutex};

use crate::proto::base;

type observee_list = HashSet<mpsc::Sender<()>>;

pub struct Observer {
    observees: Arc<Mutex<HashMap<String, observee_list>>>,
}

// Public interface of the Observer
impl Observer {
    pub fn new(data_rx: mpsc::Receiver<()>) -> Self {
        let mut s = Self {
            observees: Arc::new(Mutex::new(HashMap::new())),
        };

		s.observe(data_rx);

		s
    }

    pub fn start(&mut self, mut cmd_rx: mpsc::Receiver<(base::ModuleCore, mpsc::Sender<()>)>) {
		let observants = Arc::clone(&self.observees);

		tokio::spawn(async move {
			while let Some((core, tx)) = cmd_rx.recv().await {
				let key = Observer::generate_key(&core);

				
			}
		});
	}
}

// private interface of the Observer
impl Observer {
	fn observe(&mut self, mut data_rx: mpsc::Receiver<()>) {
		let observants = Arc::clone(&self.observees);

		tokio::spawn(async move {
			while let Some(data) = data_rx.recv().await {
				let mut locked = observants.lock().await;
				if locked.is_empty() {
					// drop(data);
				}
			}
		});
	}

    fn generate_key(core: &base::ModuleCore) -> String {
        format!("{}/{}/{}", core.namespace, core.name, core.version)
    }
}