use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, LayerShell};

use crate::render::render_widget;
use crate::widget::{Anchor, Layer, WindowConfig};

pub fn create_window(app: &gtk::Application, cfg: &WindowConfig) -> gtk::ApplicationWindow {
    let win = gtk::ApplicationWindow::builder()
        .application(app)
        .title(format!("sgs:{}", cfg.name))
        .build();

    setup_layer_shell(&win, cfg);
    setup_size(&win, cfg);
    setup_css_classes(&win, cfg);

    let child = render_widget(&cfg.child);
    win.set_child(Some(&child));

    win.present();

    win
}

fn setup_layer_shell(win: &gtk::ApplicationWindow, cfg: &WindowConfig) {
    win.init_layer_shell();
    win.set_namespace("sgs");
    win.set_layer(to_layer_shell_layer(&cfg.layer));

    win.set_anchor(Edge::Top, false);
    win.set_anchor(Edge::Bottom, false);
    win.set_anchor(Edge::Left, false);
    win.set_anchor(Edge::Right, false);

    for anchor in &cfg.anchor {
        win.set_anchor(to_layer_shell_edge(anchor), true);
    }

    if cfg.anchor.contains(&Anchor::Top) || cfg.anchor.contains(&Anchor::Bottom) {
        if let Some(height) = cfg.height {
            win.set_exclusive_zone(height);
        }
    } else if cfg.anchor.contains(&Anchor::Left) || cfg.anchor.contains(&Anchor::Right) {
        if let Some(width) = cfg.width {
            win.set_exclusive_zone(width);
        }
    }

    win.set_keyboard_mode(KeyboardMode::None);
}

fn setup_size(win: &gtk::ApplicationWindow, cfg: &WindowConfig) {
    if let Some(width) = cfg.width {
        win.set_default_width(width);
    }

    if let Some(height) = cfg.height {
        win.set_default_height(height);
    }
}

fn setup_css_classes(win: &gtk::ApplicationWindow, cfg: &WindowConfig) {
    win.add_css_class("sgs-window");

    for class in &cfg.class {
        win.add_css_class(class);
    }
}

fn to_layer_shell_layer(layer: &Layer) -> gtk4_layer_shell::Layer {
    match layer {
        Layer::Background => gtk4_layer_shell::Layer::Background,
        Layer::Bottom => gtk4_layer_shell::Layer::Bottom,
        Layer::Top => gtk4_layer_shell::Layer::Top,
        Layer::Overlay => gtk4_layer_shell::Layer::Overlay,
    }
}

fn to_layer_shell_edge(anchor: &Anchor) -> Edge {
    match anchor {
        Anchor::Top => Edge::Top,
        Anchor::Bottom => Edge::Bottom,
        Anchor::Left => Edge::Left,
        Anchor::Right => Edge::Right,
    }
}
