use crate::proto::base;

use super::structure;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct Store {
    internal: Arc<Mutex<HashMap<String, structure::structure>>>,
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Self { internal: Arc::clone(&self.internal) }
    }
}

impl Store {
    pub fn new() -> Self {
        Self {
            internal: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add(&mut self, module: base::Module) {
        let mut locked = self.internal.lock().await;

        locked.insert(
            Store::generate_key(&module),
            structure::structure::new(module),
        );
    }

    fn generate_key(module: &base::Module) -> String {
        let core = module.core.as_ref();
        if let Some(core) = core {
            return format!("{}/{}/{}", core.namespace, core.name, core.version);
        }

        String::default()
    }
}
