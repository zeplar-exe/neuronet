use std::collections::HashSet;

use crate::state::{Neuron, NeuronId};
use crate::tools::{Tool, ToolContext};
use dioxus::prelude::*;

pub struct EdgeTool {
    dragging: bool,
    drag_start: (f64, f64),
    moved: bool,
    suppress_click: bool,
    drag_ids: HashSet<NeuronId>,
    start_positions: std::collections::HashMap<NeuronId, (f64, f64)>,
}
impl Default for EdgeTool {
    fn default() -> Self {
        Self {
            dragging: false,
            drag_start: (0.0, 0.0),
            moved: false,
            suppress_click: false,
            drag_ids: HashSet::default(),
            start_positions: Default::default(),
        }
    }
}
impl Tool for EdgeTool {
    fn name(&self) -> &'static str {
        "edge"
    }
    fn on_node_click(&mut self, ctx: &mut ToolContext, id: &NeuronId, evt: &MouseEvent) {
        if self.suppress_click {
            self.suppress_click = false;
            return;
        }
        let mods = evt.modifiers();
        let shift = mods.shift();
        let multi = mods.ctrl() || mods.meta();

        if shift || multi {
            let mut s = ctx.store.write();
            let sources: HashSet<NeuronId> = s.selected.clone();

            for src in sources {
                if src == *id {
                    continue;
                }
                if !s.has_edge(src, *id) {
                    s.push_edge(src, *id);
                }
            }
            if shift {
                s.select_only(*id);
            } else {
                s.ensure_selected(*id);
            }
        } else {
            ctx.store.write().select_only(*id);
        }
    }
    fn on_node_drag_start(&mut self, ctx: &mut ToolContext, id: &NeuronId, evt: &MouseEvent) {
        let mut s = ctx.store.write();
        let selected_contains = s.selected.iter().any(|nid| *nid == *id);
        let mods = evt.modifiers();

        if !selected_contains {
            if mods.ctrl() || mods.meta() {
                s.ensure_selected(*id);
            } else {
                s.select_only(*id);
            }
        }
        self.drag_ids = s.selected.clone();
        self.start_positions = s
            .network
            .neurons
            .iter()
            .filter(|(nid, _)| self.drag_ids.contains(&nid))
            .map(|(nid, n)| (nid, n.position))
            .collect();
        self.dragging = true;
        self.moved = false;
        self.suppress_click = false;
        self.drag_start = (evt.client_coordinates().x as f64, evt.client_coordinates().y as f64);
    }
    fn on_drag(&mut self, ctx: &mut ToolContext, evt: &MouseEvent) {
        if !self.dragging {
            return;
        }
        let x = evt.client_coordinates().x as f64;
        let y = evt.client_coordinates().y as f64;
        let dx = x - self.drag_start.0;
        let dy = y - self.drag_start.1;
        if dx.abs() + dy.abs() > 0.5 {
            self.moved = true;
            self.suppress_click = true;
        }
        if self.drag_ids.is_empty() {
            return;
        }
        let mut s = ctx.store.write();
        for (nid, n) in s.network.neurons.iter_mut() {
            if let Some(&(sx, sy)) = self.start_positions.get(&nid) {
                if self.drag_ids.contains(&nid) {
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
