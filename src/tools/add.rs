use crate::common::Rect;
use crate::state::ViewType;
use crate::tools::{Tool, ToolContext, ToolRequest};
use dioxus::prelude::*;

pub struct AddTool {
    drag_start: Option<(f64, f64)>,
    rect: Option<Rect>,
    pending: Option<ToolRequest>,
}
impl Default for AddTool {
    fn default() -> Self {
        Self {
            drag_start: None,
            rect: None,
            pending: None,
        }
    }
}
impl Tool for AddTool {
    fn name(&self) -> &'static str {
        "add"
    }
    fn on_canvas_click(&mut self, _ctx: &mut ToolContext, _evt: &MouseEvent) {}
    fn on_drag_start(&mut self, _ctx: &mut ToolContext, evt: &MouseEvent) {
        let x = evt.element_coordinates().x as f64;
        let y = evt.element_coordinates().y as f64;
        self.drag_start = Some((x, y));
        self.rect = Some(Rect::from_dimensions_f(x, y, 0.0, 0.0));
    }
    fn on_drag(&mut self, _ctx: &mut ToolContext, evt: &MouseEvent) {
        let x = evt.element_coordinates().x as f64;
        let y = evt.element_coordinates().y as f64;
        if let Some((sx, sy)) = self.drag_start {
            let left = sx.min(x);
            let top = sy.min(y);
            let w = (sx - x).abs();
            let h = (sy - y).abs();
            self.rect = Some(Rect::from_dimensions_f(left, top, w, h));
        }
    }
    fn on_drag_end(&mut self, ctx: &mut ToolContext, _evt: &MouseEvent) {
        let (x, y, w, h) = self.rect.clone().unwrap().as_tuple_wh();

        if w >= 2.0 && h >= 2.0 {
            self.pending = Some(ToolRequest::AddPopulation {
                rect: Rect::from_dimensions_f(x, y, w, h),
            });
        }

        self.drag_start = None;
        ctx.store.write().set_tool(ViewType::EDIT, "add");
    }
    fn select_rect(&self) -> Option<Rect> {
        self.rect.clone()
    }
    fn request(&self) -> Option<ToolRequest> {
        self.pending.clone()
    }
    fn clear_request(&mut self) {
        self.pending = None;
        self.rect = None;
        self.drag_start = None;
    }
}
