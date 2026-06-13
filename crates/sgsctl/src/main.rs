use anyhow::{Context, Result};
use serde_json::json;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

fn main() -> Result<()> {
    let cmd = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "ping".to_string());

    let payload = match cmd.as_str() {
        "ping" => json!({ "cmd": "ping" }),
        "reload" => json!({ "cmd": "reload" }),
        "quit" => json!({ "cmd": "quit" }),
        other => {
            anyhow::bail!("unknown command: {other}");
        }
    };

    let socket = socket_path();
    let mut stream = UnixStream::connect(&socket)
        .with_context(|| format!("failed to connect to SGS socket: {}", socket.display()))?;

    writeln!(stream, "{payload}")?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    print!("{response}");

    Ok(())
}

fn socket_path() -> PathBuf {
    PathBuf::from("/tmp/sgs.sock")
}
