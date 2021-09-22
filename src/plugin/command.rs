use crate::proto::base;

pub struct Command {
    pub typ: CommandType,
    pub chan: tokio::sync::oneshot::Sender<CommandResponse>,
}

#[derive(Clone)]
pub enum CommandType {
    Apply(base::Module),
    Delete(base::ModuleCore),
    List(Filter),
    Get(base::ModuleCore),
    WatchData(Filter),
}

#[derive(Clone)]
pub enum Filter {
    Label(base::LabelSelector),
    Core(base::ModuleCore),
}

#[derive(Clone)]
pub enum CommandResponse {
    Apply(String),
    Delete(String),
    Get(Box<base::Module>),
    WatchData(Box<prost_types::Any>),
}
