use dioxus::events::MouseEvent;
use dioxus::prelude::*;
use crate::state::{Neuron, Rect};
use crate::tools::{Tool, ToolContext};

pub struct SelectTool {
    select_dragging: bool,
    node_dragging: bool,
    select_rect: Option<Rect>,
    drag_start: (f64, f64),
    moved: bool,
    suppress_node_click: bool, suppress_canvas_click: bool,
    drag_ids: Vec<i64>,
    start_positions: std::collections::HashMap<i64, (f64, f64)>,
}
impl Default for SelectTool {
    fn default() -> Self {
        Self {
            select_dragging: false, node_dragging: false,
            drag_start: (0.0, 0.0),
            select_rect: None,
            moved: false, suppress_node_click: false, suppress_canvas_click: false,
            drag_ids: vec![], start_positions: Default::default(),

        }
    }
}
impl Tool for SelectTool {
    fn name(&self) -> &'static str { "select" }
    fn on_canvas_click(&mut self, ctx: &mut ToolContext, _evt: &MouseEvent) {
        if self.suppress_canvas_click { self.suppress_canvas_click = false; return; }
    }
    fn on_node_click(&mut self, ctx: &mut ToolContext, node: &Neuron, evt: &MouseEvent) {
        if self.suppress_node_click { self.suppress_node_click = false; return; }
        self.suppress_canvas_click = true;

        let mods = evt.modifiers();
        let multi = mods.ctrl() || mods.meta();
        if multi { ctx.store.write().toggle_selected(node.id) }
        else { ctx.store.write().select_only(node.id) }
    }
    fn on_node_drag_start(&mut self, ctx: &mut ToolContext, node: &Neuron, evt: &MouseEvent) {
        let mut s = ctx.store.write();
        let mods = evt.modifiers();
        if mods.ctrl() || mods.meta() { s.ensure_selected(node.id); }
        else { s.select_only(node.id); }

        self.drag_ids = s.selected.clone();
        self.start_positions = s.network.neurons.iter().filter(|n| self.drag_ids.contains(&n.id)).map(|n| (n.id, n.position)).collect();
        self.node_dragging = true;
        self.select_dragging = false;
        self.moved = false;
        self.suppress_node_click = false;
        self.drag_start = (evt.client_coordinates().x, evt.client_coordinates().y);
    }
    fn on_drag_start(&mut self, ctx: &mut ToolContext, evt: &MouseEvent) {
        self.select_dragging = true;
        self.node_dragging = false;
        self.select_rect = None;
        self.drag_start = (evt.client_coordinates().x, evt.client_coordinates().y);
    }
    fn on_drag(&mut self, ctx: &mut ToolContext, evt: &MouseEvent) {
        let x = evt.client_coordinates().x;
        let y = evt.client_coordinates().y;

        if self.select_dragging {
            let left = self.drag_start.0.min(x);
            let top = self.drag_start.1.min(y);
            let w = (self.drag_start.0 - x).abs();
            let h = (self.drag_start.1 - y).abs();
            self.select_rect = Some(Rect::from_dimensions_f(left, top, w, h));

            return;
        }

        if !self.node_dragging { return; }

        let dx = x - self.drag_start.0;
        let dy = y - self.drag_start.1;

        if dx.abs() + dy.abs() > 0.5 {
            self.moved = true;
            self.suppress_node_click = true;
        }

        if self.drag_ids.is_empty() { return; }
        let mut s = ctx.store.write();

        for n in s.network.neurons.iter_mut() {
            if let Some(&(sx, sy)) = self.start_positions.get(&n.id) {
                if self.drag_ids.contains(&n.id) {
                    n.position = (sx + dx, sy + dy);
                }
            }
        }
    }
    fn on_drag_end(&mut self, ctx: &mut ToolContext, evt: &MouseEvent) {
        if self.select_dragging {
            let (left, top, width, height) = self.select_rect.clone().unwrap().as_tuple_wh();
            let mut store = ctx.store.write();
            store.clear_selection();

            let mut selected = vec![];

            for neuron in store.network.neurons.iter() {
                let (nx, ny) = neuron.position;
                if nx >= left && nx <= left + width &&
                    ny >= top && ny <= top + height {
                    selected.push(neuron.id);
                }
            }

            for id in selected { store.ensure_selected(id); }


            self.suppress_canvas_click = true;
        }

        self.select_dragging = false;
        self.node_dragging = false;
        self.select_rect = None;
        self.drag_ids.clear();
        self.start_positions.clear();
        self.drag_start = (0.0, 0.0);
        self.moved = false;
    }

    fn select_rect(&self) -> Option<Rect> {
        self.select_rect.clone()
    }
}
