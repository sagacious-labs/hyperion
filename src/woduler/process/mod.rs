pub mod mail;

use std::process::ExitStatus;
use tokio::{
    io::{AsyncRead, AsyncWriteExt, BufReader},
    process,
    sync::mpsc,
};

use mail::Mail;

pub struct Process {
    bin: String,
    child: process::Child,
}

impl Process {
    /// `new` expects absolute path to the binary and `mailbox` of the parent and it returns
    /// an instance of the process
    ///
    /// Process instances are meant to be used mostly via the "Actor" interface
    pub fn new(
        bin: String,
        stdout: mpsc::Sender<Mail>,
        stderr: mpsc::Sender<Mail>,
        mut stdin: mpsc::Receiver<Mail>,
    ) -> anyhow::Result<Self> {
        let mut process = process::Command::new(bin.clone())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let cstdout = process.stdout.take().unwrap();
        tokio::spawn(async move {
            Process::observe(cstdout, stdout).await;
        });

        let cstderr = process.stderr.take().unwrap();
        tokio::spawn(async move {
            Process::observe(cstderr, stderr).await;
        });

        let mut cstdin = process.stdin.take().unwrap();
        tokio::spawn(async move {
            while let Some(mail) = stdin.recv().await {
                let _ = cstdin.write(&mail.as_bytes_vec()).await;
            }
        });

        Ok(Self {
            bin,
            child: process,
        })
    }

    /// wait_on_child will lock the child process instance and will wait for the
    /// child process to exit
    ///
    /// # Caveats
    /// - `wait_on_child` will drop the `stdin` of the child process
    pub async fn wait_on_child(&mut self) -> std::io::Result<ExitStatus> {
        self.child.wait().await
    }

    /// kill will kill the child process
    pub async fn kill(&mut self) -> std::io::Result<()> {
        Ok(self.child.kill().await?)
    }

    /// observe takes in a `pipe` which is an object must implement `AsyncRead` and `Unpin` trait
    /// and takes in a `mailbox` which will be used to send the messages that are coming through
    /// the pipe
    async fn observe<T: AsyncRead + Unpin>(pipe: T, mailbox: mpsc::Sender<Mail>) {
        let mut reader = BufReader::new(pipe);

        loop {
            let mail = Mail::from_stream(&mut reader).await;

            match mail {
                Ok(mail) => {
                    let _ = mailbox.send(mail).await;
                }
                Err(e) => (),
            }
        }
    }
}
