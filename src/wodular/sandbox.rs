use crate::{actor, store};

pub struct Sandbox {
	wasm_store: wasmtime::Store<()>,
	external_store: actor::MailBox<store::ExternalData>,
}

impl Sandbox {
	pub fn new(engine: &wasmtime::Engine, external_store: actor::MailBox<store::ExternalData>) -> Self {
		let mut config = wasmtime::Config::new();
		config.async_support(true);
		config.consume_fuel(true);

		let store = wasmtime::Store::new(engine, ());

		Self { wasm_store: store, external_store }
	}
}

pub mod wasm_hyperion_abi {
	pub struct DataAssociator {

	}

	impl DataAssociator {
		
	}
}