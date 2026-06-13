#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub name: String,
    pub monitor: i32,
    pub layer: Layer,
    pub anchor: Vec<Anchor>,
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub class: Vec<String>,
    pub child: WidgetNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum WidgetNode {
    Label(LabelNode),
    Button(ButtonNode),
    Clock(ClockNode),
    Box(BoxNode),
    CenterBox(CenterBoxNode),
    Volume(VolumeNode),
    Battery(BatteryNode),
    Cpu(CpuNode),
    Memory(MemoryNode),
    Workspaces(WorkspacesNode),
    ActiveWindow(ActiveWindowNode),
}

#[derive(Debug, Clone)]
pub struct BoxNode {
    pub class: Vec<String>,
    pub orientation: Orientation,
    pub children: Vec<WidgetNode>,
}

#[derive(Debug, Clone)]
pub struct CenterBoxNode {
    pub class: Vec<String>,
    pub start: Option<Box<WidgetNode>>,
    pub center: Option<Box<WidgetNode>>,
    pub end: Option<Box<WidgetNode>>,
}

#[derive(Debug, Clone)]
pub struct LabelNode {
    pub class: Vec<String>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ButtonNode {
    pub class: Vec<String>,
    pub text: String,
    pub on_click: Option<Action>,
}

#[derive(Debug, Clone)]
pub struct ClockNode {
    pub class: Vec<String>,
    pub format: String,
}

#[derive(Debug, Clone)]
pub struct VolumeNode {
    pub class: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ActiveWindowNode {
    pub class: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BatteryNode {
    pub class: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CpuNode {
    pub class: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MemoryNode {
    pub class: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspacesNode {
    pub class: Vec<String>,
    pub button_class: Vec<String>,
    pub count: i32,
}

#[derive(Debug, Clone)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub enum Action {
    Command(String),
    HyprDispatch(String),
}
