use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

const DEFAULT_SOCKET_PATH: &str = "/tmp/sgs.sock";

#[derive(Debug, Clone)]
pub struct IpcServer {
    socket_path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum IpcEvent {
    Reload,
    Quit,
    Open { window: String },
    Close { window: String },
    Toggle { window: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum IpcRequest {
    Ping,
    Reload,
    Quit,
    Open { window: String },
    Close { window: String },
    Toggle { window: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct IpcResponse {
    pub ok: bool,
    pub message: String,
}

impl IpcResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: message.into(),
        }
    }
}

impl IpcServer {
    pub fn new() -> Self {
        Self {
            socket_path: PathBuf::from(DEFAULT_SOCKET_PATH),
        }
    }

    pub fn with_socket_path(path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: path.into(),
        }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn start(self) -> Result<mpsc::Receiver<IpcEvent>> {
        remove_stale_socket(&self.socket_path)?;

        let listener = UnixListener::bind(&self.socket_path)
            .with_context(|| format!("failed to bind IPC socket {}", self.socket_path.display()))?;

        let (tx, rx) = mpsc::channel::<IpcEvent>();
        let socket_path = self.socket_path.clone();

        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let tx = tx.clone();

                        thread::spawn(move || {
                            if let Err(err) = handle_client(stream, tx) {
                                eprintln!("[sgs:ipc] client error: {err:#}");
                            }
                        });
                    }

                    Err(err) => {
                        eprintln!("[sgs:ipc] accept error: {err}");
                        break;
                    }
                }
            }

            let _ = fs::remove_file(socket_path);
        });

        Ok(rx)
    }
}

fn handle_client(mut stream: UnixStream, tx: mpsc::Sender<IpcEvent>) -> Result<()> {
    let cloned = stream.try_clone().context("failed to clone IPC stream")?;
    let mut reader = BufReader::new(cloned);

    let mut line = String::new();
    reader
        .read_line(&mut line)
        .context("failed to read IPC request")?;

    if line.trim().is_empty() {
        write_response(&mut stream, IpcResponse::err("empty request"))?;
        return Ok(());
    }

    let request: IpcRequest = match serde_json::from_str(line.trim()) {
        Ok(req) => req,
        Err(err) => {
            write_response(
                &mut stream,
                IpcResponse::err(format!("invalid JSON request: {err}")),
            )?;
            return Ok(());
        }
    };

    match request {
        IpcRequest::Ping => {
            write_response(&mut stream, IpcResponse::ok("pong"))?;
        }

        IpcRequest::Reload => {
            tx.send(IpcEvent::Reload)
                .context("failed to send reload IPC event")?;

            write_response(&mut stream, IpcResponse::ok("reload requested"))?;
        }

        IpcRequest::Quit => {
            tx.send(IpcEvent::Quit)
                .context("failed to send quit IPC event")?;

            write_response(&mut stream, IpcResponse::ok("quit requested"))?;
        }

        IpcRequest::Open { window } => {
            tx.send(IpcEvent::Open {
                window: window.clone(),
            })
            .context("failed to send open IPC event")?;

            write_response(
                &mut stream,
                IpcResponse::ok(format!("open requested: {window}")),
            )?;
        }

        IpcRequest::Close { window } => {
            tx.send(IpcEvent::Close {
                window: window.clone(),
            })
            .context("failed to send close IPC event")?;

            write_response(
                &mut stream,
                IpcResponse::ok(format!("close requested: {window}")),
            )?;
        }

        IpcRequest::Toggle { window } => {
            tx.send(IpcEvent::Toggle {
                window: window.clone(),
            })
            .context("failed to send toggle IPC event")?;

            write_response(
                &mut stream,
                IpcResponse::ok(format!("toggle requested: {window}")),
            )?;
        }
    }

    Ok(())
}

fn write_response(stream: &mut UnixStream, response: IpcResponse) -> Result<()> {
    let json = serde_json::to_string(&response).context("failed to serialize IPC response")?;
    stream
        .write_all(json.as_bytes())
        .context("failed to write IPC response")?;
    stream
        .write_all(b"\n")
        .context("failed to write IPC response newline")?;
    stream.flush().context("failed to flush IPC response")?;

    Ok(())
}

fn remove_stale_socket(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    match UnixStream::connect(path) {
        Ok(_) => anyhow::bail!(
            "IPC socket {} already exists and is active; is SGS already running?",
            path.display()
        ),

        Err(_) => {
            fs::remove_file(path)
                .with_context(|| format!("failed to remove stale socket {}", path.display()))?;
        }
    }

    Ok(())
}
