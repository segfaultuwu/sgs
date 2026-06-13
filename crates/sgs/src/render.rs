use gtk::prelude::*;

use crate::hyprland;
use crate::widget::*;

pub fn render_widget(node: &WidgetNode) -> gtk::Widget {
    match node {
        WidgetNode::Label(label) => {
            let w = gtk::Label::new(Some(&label.text));
            apply_classes(&w, &label.class);

            w.upcast()
        }
        WidgetNode::Button(button) => {
            let w = gtk::Button::with_label(&button.text);
            apply_classes(&w, &button.class);

            if let Some(action) = button.on_click.clone() {
                w.connect_clicked(move |_| {
                    run_action(&action);
                });
            }

            w.upcast()
        }
        WidgetNode::Box(b) => {
            let orientation = match b.orientation {
                Orientation::Horizontal => gtk::Orientation::Horizontal,
                Orientation::Vertical => gtk::Orientation::Vertical,
            };

            let w = gtk::Box::new(orientation, 4);
            apply_classes(&w, &b.class);

            for child in &b.children {
                let child_widget = render_widget(child);
                w.append(&child_widget);
            }

            w.upcast()
        }
        WidgetNode::Battery(bt) => {
            let w = gtk::Label::new(Some(""));
            apply_classes(&w, &bt.class);
            update_battery_label(&w);

            let label = w.clone();
            glib::timeout_add_seconds_local(10, move || {
                update_battery_label(&label);
                glib::ControlFlow::Continue
            });
            w.upcast()
        }
        WidgetNode::Cpu(cpu) => {
            let w = gtk::Label::new(Some(""));
            let state = std::rc::Rc::new(std::cell::RefCell::new(None));
            apply_classes(&w, &cpu.class);
            update_cpu_label(&w, &state);

            let label = w.clone();
            glib::timeout_add_seconds_local(1, move || {
                update_cpu_label(&label, &state);
                glib::ControlFlow::Continue
            });

            w.upcast()
        }
        WidgetNode::CenterBox(c) => {
            let w = gtk::CenterBox::new();
            apply_classes(&w, &c.class);

            if let Some(start) = &c.start {
                let widget = render_widget(start);
                w.set_start_widget(Some(&widget));
            }

            if let Some(center) = &c.center {
                let widget = render_widget(center);
                w.set_center_widget(Some(&widget));
            }

            if let Some(end) = &c.end {
                let widget = render_widget(end);
                w.set_end_widget(Some(&widget));
            }

            w.upcast()
        }
        WidgetNode::Clock(clock) => {
            let w = gtk::Label::new(Some(""));
            apply_classes(&w, &clock.class);

            let format = clock.format.clone();
            let label = w.clone();

            update_clock_label(&label, &format);

            glib::timeout_add_seconds_local(1, move || {
                update_clock_label(&label, &format);
                glib::ControlFlow::Continue
            });

            w.upcast()
        }
        WidgetNode::Workspaces(workspaces) => {
            let w = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            apply_classes(&w, &workspaces.class);

            render_workspaces_into(&w, workspaces);

            let box_clone = w.clone();
            let workspaces_cfg = workspaces.clone();

            match hyprland::start_event_listener() {
                Ok(rx) => {
                    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                        let mut should_refresh = false;

                        while let Ok(event) = rx.try_recv() {
                            if hyprland::event_should_refresh_ui(&event) {
                                should_refresh = true;
                            }
                        }

                        if should_refresh {
                            clear_box(&box_clone);
                            render_workspaces_into(&box_clone, &workspaces_cfg);
                        }

                        glib::ControlFlow::Continue
                    });
                }

                Err(err) => {
                    eprintln!("hyprland event listener error: {err}");

                    let box_clone = w.clone();
                    let workspaces_cfg = workspaces.clone();

                    glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
                        clear_box(&box_clone);
                        render_workspaces_into(&box_clone, &workspaces_cfg);
                        glib::ControlFlow::Continue
                    });
                }
            }

            w.upcast()
        }
        WidgetNode::Volume(v) => {
            let w = gtk::Label::new(None);
            update_volume_label(&w);
            apply_classes(&w, &v.class);

            let label = w.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(750), move || {
                update_volume_label(&label);
                glib::ControlFlow::Continue
            });
            w.upcast()
        }
        WidgetNode::Memory(m) => {
            let w = gtk::Label::new(None);
            update_memory_label(&w);
            apply_classes(&w, &m.class);
            let label = w.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(750), move || {
                update_memory_label(&label);
                glib::ControlFlow::Continue
            });
            w.upcast()
        }
        WidgetNode::ActiveWindow(a) => {
            let w = gtk::Label::new(None);
            update_active_window_label(&w);
            apply_classes(&w, &a.class);
            let label = w.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(750), move || {
                update_active_window_label(&label);
                glib::ControlFlow::Continue
            });
            w.upcast()
        }
    }
}

fn update_active_window_label(label: &gtk::Label) {
    match crate::system::read_active_window() {
        Ok(w) => {
            label.set_label(&w);
        }
        Err(_) => {
            label.set_label("??");
        }
    }
}

fn update_memory_label(label: &gtk::Label) {
    match crate::system::read_memory_info() {
        Ok(mem) => {
            let used = crate::system::format_kib(mem.used_kib);
            let total = crate::system::format_kib(mem.total_kib);

            label.set_label(&format!("RAM: {used}/{total}"));
        }

        Err(_) => {
            label.set_label("RAM: ??");
        }
    }
}

fn update_cpu_label(
    label: &gtk::Label,
    state: &std::rc::Rc<std::cell::RefCell<Option<crate::system::CpuSnapshot>>>,
) {
    let Ok(next) = crate::system::read_cpu_snapshot() else {
        label.set_label("CPU: ??");
        return;
    };

    let mut prev = state.borrow_mut();

    if let Some(old) = *prev {
        let usage = crate::system::cpu_usage_percent(old, next);
        label.set_label(&format!("CPU: {:.0}%", usage));
    } else {
        label.set_label("CPU: --");
    }

    *prev = Some(next);
}

fn update_clock_label(label: &gtk::Label, format: &str) {
    let now = chrono::Local::now().format(format).to_string();
    label.set_label(&now);
}

fn render_workspaces_into(container: &gtk::Box, cfg: &WorkspacesNode) {
    let active_id = hyprland::get_active_workspace().map(|w| w.id).unwrap_or(1);

    let mut ids = hyprland::get_workspaces()
        .map(|workspaces| {
            let mut ids = workspaces
                .into_iter()
                .map(|w| w.id)
                .filter(|id| *id > 0)
                .collect::<Vec<_>>();

            ids.sort();
            ids.dedup();
            ids
        })
        .unwrap_or_else(|_| vec![1, 2, 3, 4, 5]);

    for id in 1..=cfg.count {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }

    ids.sort();
    ids.dedup();

    for id in ids {
        let button = gtk::Button::with_label(&id.to_string());

        button.add_css_class("workspace");

        if id == active_id {
            button.add_css_class("workspace-active");
        }

        for class in &cfg.button_class {
            button.add_css_class(class);
        }

        let dispatch = format!("workspace {id}");

        button.connect_clicked(move |_| {
            hyprland::dispatch(&dispatch);
        });

        container.append(&button);
    }
}

fn clear_box(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn apply_classes<W: IsA<gtk::Widget>>(widget: &W, classes: &[String]) {
    for class in classes {
        widget.add_css_class(class);
    }
}

fn run_action(action: &Action) {
    match action {
        Action::Command(cmd) => {
            let _ = std::process::Command::new("sh").arg("-c").arg(cmd).spawn();
        }

        Action::HyprDispatch(dispatch) => hyprland::dispatch(dispatch),
    }
}

// Helpers

fn has_class(classes: &[String], wanted: &str) -> bool {
    classes.iter().any(|class| class == wanted)
}

fn update_volume_label(label: &gtk::Label) {
    let volume = get_volume_percent().unwrap_or_else(|| "??".to_string());
    label.set_label(&format!("VOL: {volume}"));
}

fn update_battery_label(label: &gtk::Label) {
    let battery = get_battery_percent().unwrap_or_else(|| "??".to_string());
    label.set_label(&format!("BAT: {battery}"));
}

fn get_volume_percent() -> Option<String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(
            r#"
            if command -v wpctl >/dev/null 2>&1; then
              wpctl get-volume @DEFAULT_AUDIO_SINK@ | awk '{
                vol = int($2 * 100)
                if ($0 ~ /MUTED/) print "MUTE"
                else print vol "%"
              }'
            elif command -v pamixer >/dev/null 2>&1; then
              if pamixer --get-mute | grep -q true; then
                echo "MUTE"
              else
                echo "$(pamixer --get-volume)%"
              fi
            else
              echo "N/A"
            fi
            "#,
        )
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if text.is_empty() { None } else { Some(text) }
}

fn get_battery_percent() -> Option<String> {
    let battery_path = find_battery_path()?;

    let capacity_path = battery_path.join("capacity");
    let status_path = battery_path.join("status");

    let capacity = std::fs::read_to_string(capacity_path).ok()?;
    let capacity = capacity.trim();

    let status = std::fs::read_to_string(status_path)
        .unwrap_or_default()
        .trim()
        .to_string();

    let icon = match status.as_str() {
        "Charging" => "+",
        "Discharging" => "-",
        "Full" => "=",
        _ => "",
    };

    Some(format!("{capacity}%{icon}"))
}

fn find_battery_path() -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir("/sys/class/power_supply").ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        let type_path = path.join("type");

        let Ok(kind) = std::fs::read_to_string(type_path) else {
            continue;
        };

        if kind.trim() == "Battery" {
            return Some(path);
        }
    }

    None
}
