use anyhow::{Context, Result};
use gtk::AccessibleRole::Option;
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, Deserialize)]
pub struct HyprWorkspace {
    pub id: i32,
    pub name: String,

    #[serde(default)]
    pub monitor: String,

    #[serde(default)]
    pub windows: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HyprActiveWorkspace {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActiveWindow {
    pub address: String,
    pub class: String,
    pub title: String,
    pub workspace: HyprActiveWorkspace,
}

#[derive(Debug, Clone)]
pub enum HyprlandEvent {
    Workspace(String),
    FocusedMonitor(String),
    ActiveWindow(String),
    CreateWorkspace(String),
    DestroyWorkspace(String),
    MoveWorkspace(String),
    OpenWindow(String),
    CloseWindow(String),
    Unknown(String),
}

pub fn get_workspaces() -> Result<Vec<HyprWorkspace>> {
    let output = Command::new("hyprctl")
        .args(["workspaces", "-j"])
        .output()
        .context("failed to run hyprctl workspaces -j")?;

    if !output.status.success() {
        anyhow::bail!(
            "hyprctl workspaces failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let mut workspaces = serde_json::from_slice::<Vec<HyprWorkspace>>(&output.stdout)
        .context("failed to parse hyprctl workspaces json")?;

    workspaces.sort_by_key(|w| w.id);

    Ok(workspaces)
}

pub fn get_active_workspace() -> Result<HyprActiveWorkspace> {
    let output = Command::new("hyprctl")
        .args(["activeworkspace", "-j"])
        .output()
        .context("failed to run hyprctl activeworkspace -j")?;

    if !output.status.success() {
        anyhow::bail!(
            "hyprctl activeworkspace failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let workspace = serde_json::from_slice::<HyprActiveWorkspace>(&output.stdout)
        .context("failed to parse hyprctl activeworkspace json")?;

    Ok(workspace)
}

pub fn dispatch(args: &str) {
    let mut parts = args.split_whitespace();

    let Some(dispatcher) = parts.next() else {
        return;
    };

    let mut cmd = std::process::Command::new("hyprctl");
    cmd.arg("dispatch");
    cmd.arg(dispatcher);

    for arg in parts {
        cmd.arg(arg);
    }

    if let Err(err) = cmd.spawn() {
        eprintln!("hyprctl dispatch spawn error: {err}");
    }
}

pub fn start_event_listener() -> Result<mpsc::Receiver<HyprlandEvent>> {
    let socket_path = get_hyprland_event_socket_path()?;
    let stream = UnixStream::connect(&socket_path)
        .with_context(|| format!("failed to connect to Hyprland event socket: {socket_path:?}"))?;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let reader = BufReader::new(stream);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let event = parse_event(&line);

                    if tx.send(event).is_err() {
                        break;
                    }
                }

                Err(err) => {
                    eprintln!("hyprland event socket read error: {err}");
                    break;
                }
            }
        }
    });

    Ok(rx)
}

pub fn get_active_window_title() -> Result<String> {
    let output = Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .context("cannot run hyprctl activewindow -j")?;

    if !output.status.success() {
        anyhow::bail!(
            "hyprctl activewindow failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let active_window: ActiveWindow = serde_json::from_slice(&output.stdout)
        .context("failed to parse hyprctl activewindow json")?;

    let title = active_window.title.trim();

    if title.is_empty() {
        Ok("Desktop".to_string())
    } else {
        Ok(title.to_string())
    }
}

pub fn event_should_refresh_ui(event: &HyprlandEvent) -> bool {
    matches!(
        event,
        HyprlandEvent::Workspace(_)
            | HyprlandEvent::FocusedMonitor(_)
            | HyprlandEvent::CreateWorkspace(_)
            | HyprlandEvent::DestroyWorkspace(_)
            | HyprlandEvent::MoveWorkspace(_)
            | HyprlandEvent::OpenWindow(_)
            | HyprlandEvent::CloseWindow(_)
    )
}

fn parse_event(line: &str) -> HyprlandEvent {
    let Some((name, data)) = line.split_once(">>") else {
        return HyprlandEvent::Unknown(line.to_string());
    };

    match name {
        "workspace" => HyprlandEvent::Workspace(data.to_string()),
        "focusedmon" => HyprlandEvent::FocusedMonitor(data.to_string()),
        "activewindow" => HyprlandEvent::ActiveWindow(data.to_string()),
        "createworkspace" => HyprlandEvent::CreateWorkspace(data.to_string()),
        "destroyworkspace" => HyprlandEvent::DestroyWorkspace(data.to_string()),
        "moveworkspace" => HyprlandEvent::MoveWorkspace(data.to_string()),
        "openwindow" => HyprlandEvent::OpenWindow(data.to_string()),
        "closewindow" => HyprlandEvent::CloseWindow(data.to_string()),
        _ => HyprlandEvent::Unknown(line.to_string()),
    }
}

fn get_hyprland_event_socket_path() -> Result<PathBuf> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR is not set")?;

    let instance_signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("HYPRLAND_INSTANCE_SIGNATURE is not set. Are you running inside Hyprland?")?;

    Ok(PathBuf::from(runtime_dir)
        .join("hypr")
        .join(instance_signature)
        .join(".socket2.sock"))
}
