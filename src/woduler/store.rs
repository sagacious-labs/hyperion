use std::{collections::{HashMap, HashSet}, sync::Arc};

use tokio::sync::Mutex;

use crate::{proto::base, utility};

pub struct Store<'a> {
    data: Mutex<HashMap<String, base::Any>>,
	childs: Mutex<HashMap<String, ChildStore<'a>>>
}

impl<'a> Store<'a> {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
			childs: Mutex::new(HashMap::new()),
        }
    }

    pub async fn create_child(&'a mut self, md: &base::Module) -> Result<ChildStore<'a>, utility::error::ModuleCoreKeyErr> {
        let key = utility::module_core_key(md)?;

		let child = ChildStore::new(self);

		let mut locked = self.childs.lock().await;
		locked.insert(key, child.clone());

        Ok(child)
    }

	pub async fn get_child(&self, md: &base::Module) -> Result<ChildStore<'a>, utility::error::ModuleCoreKeyErr> {
        let key = utility::module_core_key(md)?;

		let locked = self.childs.lock().await;
		let child = locked.get(&key).ok_or(utility::error::ModuleCoreKeyErr)?;

		Ok(child.clone())
	}
}

#[derive(Clone)]
pub struct ChildStore<'a> {
    parent: &'a Store<'a>,
    data: Arc<Mutex<HashSet<String>>>,
}

impl<'a> ChildStore<'a> {
    pub fn new(parent: &'a Store<'a>) -> Self {
        Self {
            parent,
            data: Arc::new(Mutex::new(HashSet::new())),
        }
    }

	pub async fn insert(&mut self, value: &str) {
		let mut locked = self.data.lock().await;
		locked.insert(value.to_owned());
	}

	pub async fn get(&self, key: &str) {
		let locked = self.data.lock().await;
		locked.get(key);
	}
}
