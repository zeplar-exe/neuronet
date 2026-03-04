use dioxus::prelude::*;
use crate::components::explorer::Explorer;
use crate::components::execute_properties::ExecuteProperties;
use crate::components::circle::CircleNode;
use crate::state::{AppStore, ViewType};
use crate::tools::{tool_by_name, ToolContext};

#[component]
pub fn ExecuteView() -> Element {
    let mut store = use_context::<Signal<AppStore>>();
    let tool_ctx = ToolContext { store: store.clone() };
    
    if store.read().set_view_state(ViewType::EXECUTE).selected_tool != "select" {
        store.write().set_tool(ViewType::EXECUTE, "select");
    }
    let mut active_tool: Signal<Box<dyn crate::tools::Tool>> = use_signal(|| tool_by_name("select"));

    let is_dragging = use_signal(|| false);

    let onclick_canvas = {
        let tool_ctx = tool_ctx.clone();
        let mut active_tool = active_tool.clone();
        move |evt: MouseEvent| {
            let mut ctx = tool_ctx.clone();
            active_tool.write().on_canvas_click(&mut ctx, &evt);
        }
    };
    let onmousedown_canvas = {
        let tool_ctx = tool_ctx.clone();
        let mut is_dragging = is_dragging.clone();
        move |evt: MouseEvent| {
            is_dragging.set(true);
            let mut ctx = tool_ctx.clone();
            active_tool.write().on_drag_start(&mut ctx, &evt);
        }
    };
    let onmousemove_canvas = {
        let tool_ctx = tool_ctx.clone();
        move |evt: MouseEvent| {
            if is_dragging.read().to_owned() {
                let mut ctx = tool_ctx.clone();
                active_tool.write().on_drag(&mut ctx, &evt);
            }
        }
    };
    let onmouseup_canvas = {
        let tool_ctx = tool_ctx.clone();
        let mut is_dragging = is_dragging.clone();
        move |evt: MouseEvent| {
            if is_dragging.read().to_owned() {
                let mut ctx = tool_ctx.clone();
                active_tool.write().on_drag_end(&mut ctx, &evt);
                is_dragging.set(false);
            }
        }
    };

    let canvas_class = "nn-canvas";
    let sidebar_base_class = "nn-sidebar";

    rsx! {
        div { class: canvas_class,
            tabindex: 0,
            onclick: onclick_canvas,
            onmousedown: onmousedown_canvas,
            onmousemove: onmousemove_canvas,
            onmouseup: onmouseup_canvas,

            for node in store.read().network.neurons.clone().into_iter() {
                RenderNode { node: node, store: store.clone(), active_tool: active_tool.clone(), is_dragging: is_dragging.clone() }
            }
        }

        {
            let left_sidebar_class = format!("{} nn-sidebar-left", sidebar_base_class);
            let right_sidebar_class = format!("{} nn-sidebar-right", sidebar_base_class);
            rsx! {
                aside { class: left_sidebar_class, Explorer {} }
                aside { class: right_sidebar_class, ExecuteProperties {} }
            }
        }
    }
}

#[component]
fn RenderNode(
    node: crate::state::Neuron,
    store: Signal<crate::state::AppStore>,
    active_tool: Signal<Box<dyn crate::tools::Tool>>,
    is_dragging: Signal<bool>,
) -> Element {
    let radius = 20.0;
    let selected = store.read().selected.iter().any(|sid| sid == &node.id);
    let node_id = node.id;
    let left = node.position.0;
    let top = node.position.1;
    let node_style = format!("position: absolute; left: {left}px; top: {top}px;");
    
    // Clone node for use in multiple move closures
    let node_for_down = node.clone();
    let node_for_click = node.clone();
    
    rsx! {
        div { style: node_style, class: "nn-select-none",
            onmousedown: move |e| {
                e.stop_propagation();
                is_dragging.set(true);
                let this_node = crate::state::Neuron { id: node_id, ..node_for_down.clone() };
                let mut tool_ctx2 = ToolContext { store: store.clone() };
                active_tool.write().on_node_drag_start(&mut tool_ctx2, &this_node, &e);
            },
            onclick: move |e| {
                e.stop_propagation();
                let this_node = crate::state::Neuron { id: node_id, ..node_for_click.clone() };
                let mut tool_ctx2 = ToolContext { store: store.clone() };
                active_tool.write().on_node_click(&mut tool_ctx2, &this_node, &e);
            },
            CircleNode { radius, selected }
        }
    }
}
