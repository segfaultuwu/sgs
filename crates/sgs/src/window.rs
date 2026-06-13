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

#[derive(Debug, Clone)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

#[derive(Debug, Clone)]
pub enum Anchor {
    Top,
    Bottom,
    Left,
    Right,
}
