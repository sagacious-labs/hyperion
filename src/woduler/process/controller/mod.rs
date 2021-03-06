mod state;

use std::{env, io::Cursor, path::Path, sync::Arc};

use anyhow::{anyhow, Result};
use tokio::{
    select,
    sync::{mpsc, Mutex, Notify},
};
use uuid::Uuid;

use super::{mail, Mail, Process};
use crate::{proto::base, woduler::event::ModuleEventBus};
use state::*;

pub struct Controller {
    process_state: Arc<Mutex<ProcessState>>,
    cancel: Arc<Notify>,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            process_state: Arc::new(Mutex::new(ProcessState::new())),
            cancel: Arc::new(Notify::new()),
        }
    }

    /// run takes in a module definition and a module event bus and spins up a
    /// process as defined in the module definition
    ///
    /// Module Event Bus is used to connect process streams to the main event bus
    pub fn run(&mut self, md: &base::Module, mut eb: ModuleEventBus) {
        let state = Arc::clone(&self.process_state);
        let cancel = self.cancel.clone();
        let md = md.to_owned();

        tokio::spawn(async move {
            let mut timeout = 1u64;
            let mut is_ok = true;

            while is_ok {
                log::debug!("starting process");

                let bin = Self::setup_binary(&md).await;
                if let Err(err) = &bin {
                    let mut state = state.lock().await;
                    state.set(State::Error(err.to_string()));

                    log::error!("failed to setup process binary: {}", err);

                    // Exponential backoff
                    tokio::time::sleep(std::time::Duration::from_secs(timeout)).await;
                    timeout *= 2;

                    continue;
                }

                let bin = bin.ok().unwrap();

                log::debug!("setup process binary at: {}", bin);

                let (stdout_tx, stdout_rx) = mpsc::channel(8);
                let (stdin_tx, stdin_rx) = mpsc::channel(8);

                let (data_rx, log_rx) = Self::split_stdout(stdout_rx);

                if let Ok(mut process) = Process::new(bin, stdout_tx, stdin_rx) {
                    {
                        let mut state = state.lock().await;
                        state.set(State::Running);
                    }

                    log::debug!("Process started");

                    // Wire the process channels with the event bus
                    eb.stream_data(data_rx);
                    eb.stream_logs(log_rx);
                    eb.recv_data(stdin_tx);

                    select! {
                        status = process.wait_on_child() => {
                            match status {
                                Ok(status) => {
                                    let mut state = state.lock().await;
                                    state.set(State::Exit(status));
                                }
                                Err(err) => {
                                    let mut state = state.lock().await;
                                    state.set(State::Error(err.to_string()));
                                }
                            }
                        }
                        _ = cancel.notified() => {
                            is_ok = false;

                            log::debug!("received process termination");

                            match process.terminate().await {
                                Ok(status) => {
                                    let mut state = state.lock().await;
                                    state.set(State::Exit(status));
                                }
                                Err(err) => {
                                    let mut state = state.lock().await;
                                    state.set(State::Error(err.to_string()));
                                }
                            }
                        }
                    }

                    // Cleanup the module event bus
                    eb.cleanup().await;
                } else {
                    log::error!("failed to startup process - init crashed");
                    let mut state = state.lock().await;
                    state.set(State::InitCrashLoopBackOff);
                }

                // Exponential backoff
                tokio::time::sleep(std::time::Duration::from_secs(timeout)).await;
                timeout *= 2;
            }
        });
    }

    /// stop will submit a stop request to the process controller but does not guarantee
    /// immediate stoppage
    pub fn stop(&self) {
        self.cancel.notify_one();
    }

    /// get_status returns status of the running process
    pub async fn get_status(&self) -> String {
        self.process_state.lock().await.to_string()
    }

    fn split_stdout(
        mut stdout: mpsc::Receiver<Mail>,
    ) -> (mpsc::Receiver<Mail>, mpsc::Receiver<Mail>) {
        let (data_tx, data_rx) = mpsc::channel(8);
        let (log_tx, log_rx) = mpsc::channel(8);

        tokio::spawn(async move {
            while let Some(mail) = stdout.recv().await {
                match mail.typ {
                    mail::data_type::LOG => {
                        if log_tx.send(mail).await.is_err() {
                            log::error!(
                                "failed to direct message of type: \"log\" to the listener"
                            );
                        }
                    }
                    mail::data_type::DATA => {
                        if data_tx.send(mail).await.is_err() {
                            log::error!(
                                "failed to direct message of type: \"data\" to the listener"
                            );
                        }
                    }
                    _ => {}
                }
            }
        });

        (data_rx, log_rx)
    }

    async fn setup_binary(md: &base::Module) -> Result<String> {
        let release = Self::get_binary_location(md)?;
        let location = release.location.as_str();

        if location.starts_with("file://") {
            return Ok(location.strip_prefix("file://").unwrap().to_string());
        }

        if location.starts_with("http") {
            return Self::dowload_binary(Self::get_binary_location(md)?).await;
        }

        Err(anyhow!(
            "unsupported protocol - supported protocols for importing modules: \"file://\", \"http://\", \"https://\""
        ))
    }

    fn get_binary_location(
        md: &base::Module,
    ) -> Result<&base::module_metadata::releases::ModuleRelease> {
        if let Some(metadata) = &md.metadata {
            match &metadata.release {
                Some(base::module_metadata::Releases {
                    linux_amd64,
                    linux_arm64,
                }) => {
                    if env::consts::OS == "linux" && env::consts::ARCH == "x86_64" {
                        match linux_amd64 {
                            Some(module) => {
                                return Ok(module);
                            }
                            None => {
                                return Err(anyhow!("no release info found for LinuxAmd64"));
                            }
                        }
                    } else if env::consts::OS == "linux" && env::consts::ARCH == "aarch64" {
                        match linux_arm64 {
                            Some(module) => {
                                return Ok(module);
                            }
                            None => {
                                return Err(anyhow!("no release info found for LinuxArm64"));
                            }
                        }
                    } else {
                        return Err(anyhow!(
                            "OS: \"{}\" Arch: \"{}\" is not supported",
                            env::consts::OS,
                            env::consts::ARCH
                        ));
                    }
                }
                None => {
                    return Err(anyhow!("module releases is a required property"));
                }
            }
        }

        Err(anyhow!("module metadata not found"))
    }

    async fn dowload_binary(
        remote: &base::module_metadata::releases::ModuleRelease,
    ) -> Result<String> {
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
