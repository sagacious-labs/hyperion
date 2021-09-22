use tokio::sync::mpsc;

use crate::proto::base;

use self::errors::InvalidModuleError;

/// Sandbox represents a plugin with its metadata
///
/// It includes the following:
/// - Plugin configuration
/// - Plugin information like name, namespace, etc
/// - Plugin WASM runtime
/// - Async MPSC channel for communication between
/// manager and the sandbox
pub struct Sandbox {
    info: base::ModuleCore,
    metadata: base::ModuleMetadata,
    config: base::ModuleSpec,
    // ... WASM runtime ...
    sender: mpsc::Sender<()>,
}

// public interface for Sandbox
impl Sandbox {
    pub fn new(module: base::Module, sender: mpsc::Sender<()>) -> Result<Self, InvalidModuleError> {
        let (core, meta, cfg) = Sandbox::derive_from_module(module)?;

        Ok(Self {
            info: core,
            metadata: meta,
            config: cfg,
            sender,
        })
    }

    pub fn start() {}

    /// Get a reference to the sandbox's info.
    pub fn info(&self) -> &base::ModuleCore {
        &self.info
    }

    /// Get a reference to the sandbox's metadata.
    pub fn metadata(&self) -> &base::ModuleMetadata {
        &self.metadata
    }

    /// Get a reference to the sandbox's config.
    pub fn config(&self) -> &base::ModuleSpec {
        &self.config
    }

    pub fn to_module(&self) -> base::Module {
        base::Module {
            core: Some(self.info.clone()),
            metadata: Some(self.metadata.clone()),
            spec: Some(self.config.clone()),
        }
    }
}

// private interface for Sandbox
impl Sandbox {
    /// derive_from_module extracts the module data into
    /// a tuple if it is feasible, if the process fails due to
    /// data being absent from the module then it returns `InvalidModuleError`
    fn derive_from_module(
        module: base::Module,
    ) -> Result<
        (base::ModuleCore, base::ModuleMetadata, base::ModuleSpec),
        errors::InvalidModuleError,
    > {
        let core = module
            .core
            .ok_or_else(|| errors::InvalidModuleError::new(errors::InvalidModuleSection::Core))?;

        let meta = module.metadata.ok_or_else(|| {
            errors::InvalidModuleError::new(errors::InvalidModuleSection::Metadata)
        })?;

        let cfg = module
            .spec
            .ok_or_else(|| errors::InvalidModuleError::new(errors::InvalidModuleSection::Spec))?;

        Ok((core, meta, cfg))
    }
}

pub mod errors {
    #[derive(Debug)]
    pub enum InvalidModuleSection {
        Core,
        Metadata,
        Spec,
    }

    pub struct InvalidModuleError {
        section: InvalidModuleSection,
    }
    impl InvalidModuleError {
        pub fn new(section: InvalidModuleSection) -> Self {
            Self { section }
        }
    }

    impl std::fmt::Display for InvalidModuleError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "module section '{:?}' is invalid", self.section)
        }
    }
}
