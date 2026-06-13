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
    }
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
