use dioxus::prelude::*;
use crate::state::Node;
use crate::tools::{Tool, ToolContext};

pub struct EdgeTool {
    dragging: bool,
    drag_start: (f64, f64),
    moved: bool,
    suppress_click: bool,
    drag_ids: Vec<i64>,
    start_positions: std::collections::HashMap<i64, (f64, f64)>,
}
impl Default for EdgeTool {
    fn default() -> Self {
        Self {
            dragging: false, drag_start: (0.0, 0.0),
            moved: false, suppress_click: false,
            drag_ids: vec![], start_positions: Default::default() }
    }
}
impl Tool for EdgeTool {
    fn name(&self) -> &'static str { "edge" }
    fn on_node_click(&mut self, ctx: &mut ToolContext, node: &Node, evt: &MouseEvent) {
        if self.suppress_click { self.suppress_click = false; return; }
        let mods = evt.modifiers();
        let shift = mods.shift();
        let multi = mods.ctrl() || mods.meta();

        // If Shift or Ctrl/Cmd: create edges from all currently selected nodes to this node
        if shift || multi {
            let mut s = ctx.store.write();
            let target_id = node.id;
            let sources: Vec<i64> = s.selected.clone();
            for src in sources {
                if src == target_id { continue; }
                if !s.has_edge(src, target_id) {
                    s.push_edge(src, target_id);
                }
            }
            if shift {
                // Replace selection with just the clicked node
                s.select_only(target_id);
            } else {
                // Ctrl/Cmd: add the clicked node to the multiselect (ensure selected)
                s.ensure_selected(target_id);
            }
        } else {
            // No modifiers: behave like select-only
            ctx.store.write().select_only(node.id);
        }
    }
    fn on_node_drag_start(&mut self, ctx: &mut ToolContext, node: &Node, evt: &MouseEvent) {
        // Same drag behavior as SelectTool: allow moving currently selected or the clicked node
        let mut s = ctx.store.write();
        let selected_contains = s.selected.iter().any(|&id| id == node.id);
        let mods = evt.modifiers();
        if !selected_contains {
            if mods.ctrl() || mods.meta() { s.ensure_selected(node.id); }
            else { s.select_only(node.id); }
        }
        self.drag_ids = s.selected.clone();
        self.start_positions = s.nodes.iter().filter(|n| self.drag_ids.contains(&n.id)).map(|n| (n.id, n.position)).collect();
        self.dragging = true;
        self.moved = false;
        self.suppress_click = false;
        self.drag_start = (evt.client_coordinates().x as f64, evt.client_coordinates().y as f64);
    }
    fn on_drag(&mut self, ctx: &mut ToolContext, evt: &MouseEvent) {
        if !self.dragging { return; }
        let x = evt.client_coordinates().x as f64;
        let y = evt.client_coordinates().y as f64;
        let dx = x - self.drag_start.0;
        let dy = y - self.drag_start.1;
        if dx.abs() + dy.abs() > 0.5 { self.moved = true; self.suppress_click = true; }
        if self.drag_ids.is_empty() { return; }
        let mut s = ctx.store.write();
        for n in s.nodes.iter_mut() {
            if let Some(&(sx, sy)) = self.start_positions.get(&n.id) {
                if self.drag_ids.contains(&n.id) {
                    n.position = (sx + dx, sy + dy);
                }
            }
        }
    }
    fn on_drag_end(&mut self, _ctx: &mut ToolContext, _evt: &MouseEvent) {
        self.dragging = false;
        self.drag_ids.clear();
        self.start_positions.clear();
        self.drag_start = (0.0, 0.0);
        self.moved = false;
    }
}
