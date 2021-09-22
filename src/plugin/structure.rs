use crate::proto::base;

pub struct structure {
    // structure is supposed to represent the structure of a
    // plugin - a plugin will be composed of
    // 	- base type module which will represent the config, name, namespace, version etc of the plugin
    // 	- wasm runtime where the plugin is supposed to run
    config: base::Module,
}

impl structure {
    pub fn new(module: base::Module) -> Self {
        Self { config: module }
    }
}
