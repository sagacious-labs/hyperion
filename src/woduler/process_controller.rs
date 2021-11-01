use std::{env, io::Cursor, path::Path, process::ExitStatus, sync::Arc};

use anyhow::{anyhow, Result};
use tokio::{
    select,
    sync::{mpsc, Mutex, Notify},
};
use uuid::Uuid;

use crate::proto::base;

use super::{event::ModuleEventBus, process::Process};

pub struct ProcessController {
    process_state: Arc<Mutex<ProcessState>>,
    cancel: Arc<Notify>,
}

impl ProcessController {
    pub fn new() -> Self {
        Self {
            process_state: Arc::new(Mutex::new(ProcessState::new())),
            cancel: Arc::new(Notify::new()),
        }
    }

    pub fn run(&mut self, md: &base::Module, mut eb: ModuleEventBus) {
        let state = Arc::clone(&self.process_state);
        let cancel = self.cancel.clone();
        let md = md.to_owned();

        tokio::spawn(async move {
            let mut timeout = 1u64;
            let mut is_ok = true;

            while is_ok {
                // Setup binary
                let bin = Self::setup_binary(&md).await;
                if let Err(err) = &bin {}
                let bin = bin.ok().unwrap();

                let (stdout_tx, stdout_rx) = mpsc::channel(8);
                let (stderr_tx, stderr_rx) = mpsc::channel(8);
                let (stdin_tx, stdin_rx) = mpsc::channel(8);

                if let Ok(mut process) = Process::new(bin, stdout_tx, stderr_tx, stdin_rx) {
                    // Set the state of the process
                    {
                        let mut state = state.lock().await;
                        state.set(State::Running);
                    }

                    // Wire the process channels with the event bus
                    eb.stream_data(stdout_rx);
                    eb.recv_data(stdin_tx);

                    select! {
                        status = process.wait_on_child() => {
                            match status {
                                Ok(status) => {
                                    let mut state = state.lock().await;
                                    state.set(State::Exit(status));
                                }
                                Err(err) => {}
                            }
                        }
                        val = cancel.notified() => {
                            is_ok = false;
                        }
                    }

                    // Cleanup the module event bus
                    eb.cleanup().await;
                } else {
                    todo!()
                }

                // Exponential backoff
                tokio::time::sleep(std::time::Duration::from_secs(timeout)).await;
                timeout *= 2;
            }
        });
    }

    pub async fn stop(&self) {
        self.cancel.notify_one();
    }

    async fn setup_binary(md: &base::Module) -> Result<String> {
        Ok(Self::dowload_binary(Self::get_binary_remote_location(md)?).await?)
    }

    fn get_binary_remote_location(
        md: &base::Module,
    ) -> Result<&base::module_metadata::ModuleRelease> {
        if let Some(metadata) = &md.metadata {
            if env::consts::OS == "linux" && env::consts::ARCH == "x86_64" {
                if let Some(base::module_metadata::Release::LinuxAmd64(release)) = &metadata.release
                {
                    return Ok(release);
                } else {
                    return Err(anyhow!("no release info found for LinuxAmd64"));
                }
            }

            if env::consts::OS == "linux" && env::consts::ARCH == "aarch64" {
                if let Some(base::module_metadata::Release::LinuxArm64(release)) = &metadata.release
                {
                    return Ok(release);
                } else {
                    return Err(anyhow!("no release info found for LinuxArm64"));
                }
            }

            return Err(anyhow!(
                "OS: \"{}\" Arch: \"{}\" is not supported",
                env::consts::OS,
                env::consts::ARCH
            ));
        }

        Err(anyhow!("module metadata not found"))
    }

    async fn dowload_binary(remote: &base::module_metadata::ModuleRelease) -> Result<String> {
        let res = reqwest::get(&remote.location).await?;

        let path = Path::new(&env::temp_dir())
            .join(&remote.sha256)
            .join(Uuid::new_v4().to_string())
            .as_path()
            .display()
            .to_string();
        let mut file = tokio::fs::File::create(&path).await?;

        let mut content = Cursor::new(res.bytes().await?);
        tokio::io::copy(&mut content, &mut file).await?;

        Ok(path)
    }
}

#[derive(Clone, Copy)]
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

    pub fn get(&self) -> State {
        self.state
    }
}

#[derive(Clone, Copy)]
pub enum State {
    Init,
    Running,
    Exit(ExitStatus),
}
