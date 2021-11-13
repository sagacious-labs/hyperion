use std::process::ExitStatus;

#[derive(Clone)]
pub struct ProcessState {
    state: State,
}

impl ProcessState {
    pub fn new() -> Self {
        Self { state: State::Init }
    }

    pub fn set(&mut self, state: State) {
        self.state = state;
    }
}

impl std::string::ToString for ProcessState {
    fn to_string(&self) -> String {
        self.state.to_string()
    }
}

#[derive(Clone)]
pub enum State {
    Init,
    InitCrashLoopBackOff,
    Running,
    Error(String),
    Exit(ExitStatus),
}

impl std::string::ToString for State {
    fn to_string(&self) -> String {
        match &self {
            Self::Init => "Init".to_string(),
            Self::InitCrashLoopBackOff => "InitCrashLoopBackoff".to_string(),
            Self::Running => "Running".to_string(),
            Self::Exit(status) => format!("Exit: {}", status),
            Self::Error(err) => err.clone(),
        }
    }
}
