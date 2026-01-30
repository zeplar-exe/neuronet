pub mod select;
pub mod add;
pub mod edge;

use dioxus::prelude::*;
use crate::state::{AppStore, Neuron, ViewType};
use crate::tools::add::AddTool;
use crate::tools::edge::EdgeTool;
use crate::tools::select::SelectTool;

// ToolContext gives tools access to app state and helpers
#[derive(Clone)]
pub struct ToolContext {
    pub store: Signal<AppStore>,
}

pub trait Tool {
    fn name(&self) -> &'static str;
    fn on_canvas_click(&mut self, _ctx: &mut ToolContext, _evt: &MouseEvent) {}
    fn on_node_click(&mut self, _ctx: &mut ToolContext, _node: &Neuron, _evt: &MouseEvent) {}
    fn on_node_drag_start(&mut self, _ctx: &mut ToolContext, _node: &Neuron, _evt: &MouseEvent) {}
    fn on_drag_start(&mut self, _ctx: &mut ToolContext, _evt: &MouseEvent) {}
    fn on_drag(&mut self, _ctx: &mut ToolContext, _evt: &MouseEvent) {}
    fn on_drag_end(&mut self, _ctx: &mut ToolContext, _evt: &MouseEvent) {}

    // Optional UI exposure for overlays and requests
    fn select_rect(&self) -> Option<(f64, f64, f64, f64)> { None }
    fn request(&self) -> Option<ToolRequest> { None }
    fn clear_request(&mut self) {}
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToolRequest {
    AddPopulation { rect: (f64, f64, f64, f64) },
}

// Factory helper
pub fn tool_by_name(name: &str) -> Box<dyn Tool + 'static> {
    match name {
        "select" => Box::new(SelectTool::default()),
        "add" => Box::new(AddTool::default()),
        "edge" => Box::new(EdgeTool::default()),
        _ => Box::new(SelectTool::default()),
    }
}
