use anyhow::{Context, Result};
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;

mod app;
mod css;
mod hyprland;
mod ipc;
mod lua;
mod render;
mod system;
mod widget;

use crate::ipc::{IpcEvent, IpcServer};

#[derive(Default)]
struct Runtime {
    windows: HashMap<String, gtk::ApplicationWindow>,
}

impl Runtime {
    fn insert_window(&mut self, name: String, window: gtk::ApplicationWindow) {
        self.windows.insert(name, window);
    }

    fn show_window(&self, name: &str) {
        if let Some(window) = self.windows.get(name) {
            window.present();
        } else {
            eprintln!("[sgs] unknown window: {name}");
        }
    }

    fn hide_window(&self, name: &str) {
        if let Some(window) = self.windows.get(name) {
            window.hide();
        } else {
            eprintln!("[sgs] unknown window: {name}");
        }
    }

    fn toggle_window(&self, name: &str) {
        if let Some(window) = self.windows.get(name) {
            if window.is_visible() {
                window.hide();
            } else {
                window.present();
            }
        } else {
            eprintln!("[sgs] unknown window: {name}");
        }
    }
}

fn main() -> Result<()> {
    let config_path = config_path();
    let css_path = css_path();

    let windows = lua::load_config(
        config_path
            .to_str()
            .context("SGS config path is not valid UTF-8")?,
    )
    .with_context(|| format!("failed to load {}", config_path.display()))?;

    let ipc_rx = IpcServer::new()
        .start()
        .context("failed to start SGS IPC server")?;

    let hypr_rx = match hyprland::start_event_listener() {
        Ok(rx) => Some(rx),
        Err(err) => {
            eprintln!("[sgs:hyprland] disabled: {err:#}");
            None
        }
    };

    let gtk_app = gtk::Application::builder()
        .application_id("xyz.segfaultuwu.sgs")
        .build();

    let runtime = Rc::new(RefCell::new(Runtime::default()));

    {
        let css_path = css_path.clone();

        gtk_app.connect_startup(move |_| {
            if css_path.exists() {
                css::load_css(&css_path);
            } else {
                eprintln!("[sgs] css not found: {}", css_path.display());
            }
        });
    }

    {
        let runtime = runtime.clone();
        let windows = windows.clone();

        gtk_app.connect_activate(move |app_handle| {
            for cfg in &windows {
                let window = app::create_window(app_handle, cfg);
                runtime.borrow_mut().insert_window(cfg.name.clone(), window);
            }
        });
    }

    {
        let app_for_ipc = gtk_app.clone();
        let runtime = runtime.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            drain_ipc_events(&ipc_rx, &app_for_ipc, &runtime);
            glib::ControlFlow::Continue
        });
    }

    if let Some(hypr_rx) = hypr_rx {
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            drain_hyprland_events(&hypr_rx);
            glib::ControlFlow::Continue
        });
    }

    gtk_app.run();

    Ok(())
}

fn drain_ipc_events(
    rx: &mpsc::Receiver<IpcEvent>,
    app: &gtk::Application,
    runtime: &Rc<RefCell<Runtime>>,
) {
    loop {
        match rx.try_recv() {
            Ok(event) => handle_ipc_event(event, app, runtime),
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => break,
        }
    }
}

fn restart_self() -> ! {
    let exe = std::env::current_exe().expect("failed to get current exe");

    let args: Vec<String> = std::env::args().skip(1).collect();

    let err = std::os::unix::process::CommandExt::exec(std::process::Command::new(exe).args(args));

    panic!("failed to exec self: {err}");
}

fn handle_ipc_event(event: IpcEvent, app: &gtk::Application, runtime: &Rc<RefCell<Runtime>>) {
    match event {
        IpcEvent::Reload => {
            eprintln!("[sgs] reload requested");
            restart_self();
        }

        IpcEvent::Quit => {
            eprintln!("[sgs] quit requested");
            app.quit();
        }

        IpcEvent::Open { window } => {
            runtime.borrow().show_window(&window);
        }

        IpcEvent::Close { window } => {
            runtime.borrow().hide_window(&window);
        }

        IpcEvent::Toggle { window } => {
            runtime.borrow().toggle_window(&window);
        }
    }
}

fn drain_hyprland_events(rx: &mpsc::Receiver<hyprland::HyprlandEvent>) {
    loop {
        match rx.try_recv() {
            Ok(event) => {
                if hyprland::event_should_refresh_ui(&event) {
                    eprintln!("[sgs:hyprland] event: {event:?}");

                    // TODO: update state store, refresh widgets
                }
            }

            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => break,
        }
    }
}

fn config_path() -> PathBuf {
    std::env::var("SGS_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("sgs/init.lua")
        })
}

fn css_path() -> PathBuf {
    std::env::var("SGS_CSS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("sgs/style.css")
        })
}
