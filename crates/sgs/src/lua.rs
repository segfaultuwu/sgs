use anyhow::{Context, Result};
use mlua::{AnyUserData, Lua, Table, UserData, Value};
use std::cell::RefCell;
use std::rc::Rc;

use crate::widget::*;

#[derive(Default)]
pub struct LuaRuntimeState {
    pub windows: Vec<WindowConfig>,
}

#[derive(Debug, Clone)]
pub struct LuaWidget(pub WidgetNode);

impl UserData for LuaWidget {}

pub fn load_config(path: &str) -> Result<Vec<WindowConfig>> {
    let lua = Lua::new();
    let state = Rc::new(RefCell::new(LuaRuntimeState::default()));

    register_sgs_api(&lua, state.clone()).map_err(lua_err)?;

    let code = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read Lua config: {path}"))?;

    lua.load(&code).set_name(path).exec().map_err(lua_err)?;

    Ok(state.borrow().windows.clone())
}

fn register_sgs_api(lua: &Lua, state: Rc<RefCell<LuaRuntimeState>>) -> mlua::Result<()> {
    let sgs = lua.create_table()?;

    let label = lua.create_function(|_, opts: Table| {
        let text = get_string(&opts, "text", "");
        let class = get_class(&opts);

        Ok(LuaWidget(WidgetNode::Label(LabelNode { text, class })))
    })?;
    sgs.set("label", label)?;

    let button = lua.create_function(|_, opts: Table| {
        let text = get_string(&opts, "text", "");
        let class = get_class(&opts);

        let on_click = if let Ok(command) = opts.get::<String>("command") {
            Some(Action::Command(command))
        } else if let Ok(dispatch) = opts.get::<String>("hypr_dispatch") {
            Some(Action::HyprDispatch(dispatch))
        } else {
            None
        };

        Ok(LuaWidget(WidgetNode::Button(ButtonNode {
            text,
            class,
            on_click,
        })))
    })?;
    sgs.set("button", button)?;

    let clock = lua.create_function(|_, opts: Table| {
        let format = get_string(&opts, "format", "%H:%M:%S");
        let class = get_class(&opts);

        Ok(LuaWidget(WidgetNode::Clock(ClockNode { format, class })))
    })?;
    sgs.set("clock", clock)?;

    let battery = lua.create_function(|_, opts: Table| {
        let class = get_class(&opts);

        Ok(LuaWidget(WidgetNode::Battery(BatteryNode { class })))
    })?;
    sgs.set("battery", battery)?;

    let cpu = lua.create_function(|_, opts: Table| {
        let class = get_class(&opts);

        Ok(LuaWidget(WidgetNode::Cpu(CpuNode { class })))
    })?;
    sgs.set("cpu", cpu)?;

    let memory = lua.create_function(|_, opts: Table| {
        let class = get_class(&opts);

        Ok(LuaWidget(WidgetNode::Memory(MemoryNode { class })))
    })?;
    sgs.set("memory", memory)?;

    let volume = lua.create_function(|_, opts: Table| {
        let class = get_class(&opts);

        Ok(LuaWidget(WidgetNode::Volume(VolumeNode { class })))
    })?;
    sgs.set("volume", volume)?;

    let active_window = lua.create_function(|_, opts: Table| {
        let class = get_class(&opts);

        Ok(LuaWidget(WidgetNode::ActiveWindow(ActiveWindowNode {
            class,
        })))
    })?;
    sgs.set("active_window", active_window)?;

    let workspaces = lua.create_function(|_, opts: Table| {
        let class = get_class(&opts);
        let button_class = get_string_list(&opts, "button_class");
        let count = opts.get::<i32>("count").unwrap_or(5);

        Ok(LuaWidget(WidgetNode::Workspaces(WorkspacesNode {
            class,
            button_class,
            count,
        })))
    })?;
    sgs.set("workspaces", workspaces)?;

    let box_fn = lua.create_function(|_, opts: Table| {
        let class = get_class(&opts);

        let orientation = match get_string(&opts, "orientation", "horizontal").as_str() {
            "vertical" => Orientation::Vertical,
            _ => Orientation::Horizontal,
        };

        let children = get_children(&opts)?;

        Ok(LuaWidget(WidgetNode::Box(BoxNode {
            class,
            orientation,
            children,
        })))
    })?;
    sgs.set("box", box_fn)?;

    let centerbox = lua.create_function(|_, opts: Table| {
        let class = get_class(&opts);

        let start = get_optional_widget(&opts, "start")?.map(Box::new);
        let center = get_optional_widget(&opts, "center")?.map(Box::new);
        let end = get_optional_widget(&opts, "end")?.map(Box::new);

        Ok(LuaWidget(WidgetNode::CenterBox(CenterBoxNode {
            class,
            start,
            center,
            end,
        })))
    })?;
    sgs.set("centerbox", centerbox)?;

    let window_state = state.clone();

    let window = lua.create_function(move |_, (name, opts): (String, Table)| {
        let class = get_class(&opts);
        let child = get_required_widget(&opts, "child")?;

        let layer = parse_layer(&get_string(&opts, "layer", "top"));
        let anchor = parse_anchors(&opts);

        let win = WindowConfig {
            name,
            monitor: opts.get::<i32>("monitor").unwrap_or(0),
            layer,
            anchor,
            height: opts.get::<i32>("height").ok(),
            width: opts.get::<i32>("width").ok(),
            class,
            child,
        };

        window_state.borrow_mut().windows.push(win);

        Ok(())
    })?;
    sgs.set("window", window)?;

    let exec = lua.create_function(|_, cmd: String| {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .spawn()
            .map_err(mlua::Error::external)?;

        Ok(())
    })?;
    sgs.set("exec", exec)?;

    lua.globals().set("sgs", sgs)?;

    Ok(())
}

fn get_string(opts: &Table, key: &str, default: &str) -> String {
    opts.get::<String>(key)
        .unwrap_or_else(|_| default.to_string())
}

fn get_required_widget(opts: &Table, key: &str) -> mlua::Result<WidgetNode> {
    let userdata = opts.get::<AnyUserData>(key)?;
    let widget = userdata.borrow::<LuaWidget>()?;

    Ok(widget.0.clone())
}

fn get_optional_widget(opts: &Table, key: &str) -> mlua::Result<Option<WidgetNode>> {
    let value = opts.get::<Value>(key)?;

    match value {
        Value::Nil => Ok(None),
        Value::UserData(userdata) => {
            let widget = userdata.borrow::<LuaWidget>()?;
            Ok(Some(widget.0.clone()))
        }
        other => Err(mlua::Error::external(format!(
            "expected widget userdata or nil for key '{key}', got {other:?}"
        ))),
    }
}

fn get_children(opts: &Table) -> mlua::Result<Vec<WidgetNode>> {
    let value = opts.get::<Value>("children")?;

    let children = match value {
        Value::Nil => return Ok(Vec::new()),
        Value::Table(table) => table,
        other => {
            return Err(mlua::Error::external(format!(
                "expected children to be a table, got {other:?}"
            )));
        }
    };

    let mut out = Vec::new();

    for value in children.sequence_values::<AnyUserData>() {
        let userdata = value?;
        let widget = userdata.borrow::<LuaWidget>()?;
        out.push(widget.0.clone());
    }

    Ok(out)
}

fn get_class(opts: &Table) -> Vec<String> {
    match opts.get::<Value>("class") {
        Ok(Value::String(s)) => vec![s.to_string_lossy().to_string()],

        Ok(Value::Table(t)) => {
            let mut out = Vec::new();

            for item in t.sequence_values::<String>() {
                if let Ok(class) = item {
                    out.push(class);
                }
            }

            out
        }

        _ => Vec::new(),
    }
}

fn parse_layer(value: &str) -> Layer {
    match value {
        "background" => Layer::Background,
        "bottom" => Layer::Bottom,
        "overlay" => Layer::Overlay,
        _ => Layer::Top,
    }
}

fn parse_anchors(opts: &Table) -> Vec<Anchor> {
    let value = match opts.get::<Value>("anchor") {
        Ok(value) => value,
        Err(_) => {
            return vec![Anchor::Top, Anchor::Left, Anchor::Right];
        }
    };

    match value {
        Value::String(s) => match s.to_string_lossy().as_ref() {
            "top" => vec![Anchor::Top, Anchor::Left, Anchor::Right],
            "bottom" => vec![Anchor::Bottom, Anchor::Left, Anchor::Right],
            "left" => vec![Anchor::Top, Anchor::Bottom, Anchor::Left],
            "right" => vec![Anchor::Top, Anchor::Bottom, Anchor::Right],
            _ => vec![Anchor::Top, Anchor::Left, Anchor::Right],
        },

        Value::Table(t) => {
            let mut out = Vec::new();

            for item in t.sequence_values::<String>() {
                match item.as_deref() {
                    Ok("top") => out.push(Anchor::Top),
                    Ok("bottom") => out.push(Anchor::Bottom),
                    Ok("left") => out.push(Anchor::Left),
                    Ok("right") => out.push(Anchor::Right),
                    _ => {}
                }
            }

            if out.is_empty() {
                vec![Anchor::Top, Anchor::Left, Anchor::Right]
            } else {
                out
            }
        }

        _ => vec![Anchor::Top, Anchor::Left, Anchor::Right],
    }
}

fn lua_err(err: mlua::Error) -> anyhow::Error {
    anyhow::anyhow!("lua error: {err}")
}

fn get_string_list(opts: &Table, key: &str) -> Vec<String> {
    match opts.get::<Value>(key) {
        Ok(Value::Table(t)) => {
            let mut out = Vec::new();

            for item in t.sequence_values::<String>() {
                if let Ok(value) = item {
                    out.push(value);
                }
            }

            out
        }

        Ok(Value::String(s)) => vec![s.to_string_lossy().to_string()],

        _ => Vec::new(),
    }
}
