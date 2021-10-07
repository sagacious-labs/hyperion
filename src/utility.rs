use crate::proto::base;

/// module_core_key takes in a modul and creates a key from it
///
/// if the creation of a valid key from the module is infeasible
/// then the function will return an error
pub fn module_core_key(md: &base::Module) -> Result<String, error::ModuleCoreKeyErr> {
    let core = md.core.as_ref().ok_or(error::ModuleCoreKeyErr)?;

	Ok(format!("{}/{}/{}", core.namespace, core.name, core.version))
}

pub mod error {
    #[derive(Debug)]
    pub struct ModuleCoreKeyErr;

    impl std::fmt::Display for ModuleCoreKeyErr {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "invalid module: failed to create a key")
        }
    }
}
