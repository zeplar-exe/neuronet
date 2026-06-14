use crate::components::circle::CircleNode;
use crate::components::execute_properties::ExecuteProperties;
use crate::components::explorer::Explorer;
use crate::simulation::network::Neuron;
use crate::state::{AppStore, ViewType};
use crate::tools::{tool_by_name, ToolContext};
use dioxus::prelude::*;

#[component]
pub fn ExecuteView() -> Element {
    let mut store = use_context::<Signal<AppStore>>();
    let tool_ctx = ToolContext { store: store };

    if store.read().set_view_state(ViewType::EXECUTE).selected_tool != "select" {
        store.write().set_tool(ViewType::EXECUTE, "select");
    }
    let mut active_tool: Signal<Box<dyn crate::tools::Tool>> = use_signal(|| tool_by_name("select"));

    let is_dragging = use_signal(|| false);

    let onclick_canvas = {
        let mut tool_ctx = tool_ctx.clone();
        move |evt: MouseEvent| {
            active_tool.write().on_canvas_click(&mut tool_ctx, &evt);
        }
    };
    let onmousedown_canvas = {
        let mut tool_ctx = tool_ctx.clone();
        let mut is_dragging = is_dragging.clone();
        move |evt: MouseEvent| {
            is_dragging.set(true);
            active_tool.write().on_drag_start(&mut tool_ctx, &evt);
        }
    };
    let onmousemove_canvas = {
        let mut tool_ctx = tool_ctx.clone();
        move |evt: MouseEvent| {
            if is_dragging.read().to_owned() {
                active_tool.write().on_drag(&mut tool_ctx, &evt);
            }
        }
    };
    let onmouseup_canvas = {
        let mut tool_ctx = tool_ctx.clone();
        let mut is_dragging = is_dragging.clone();
        move |evt: MouseEvent| {
            if is_dragging.read().to_owned() {
                active_tool.write().on_drag_end(&mut tool_ctx, &evt);
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
    node: Neuron,
    store: Signal<AppStore>,
    active_tool: Signal<Box<dyn crate::tools::Tool>>,
    is_dragging: Signal<bool>,
) -> Element {
    let radius = 20.0;
    let selected = store.read().selected.iter().any(|sid| sid == &node.id);
    let left = node.position.0;
    let top = node.position.1;
    let node_style = format!("position: absolute; left: {left}px; top: {top}px;");

    rsx! {
        div { style: node_style, class: "nn-select-none",
            onmousedown: move |e| {
                e.stop_propagation();
                is_dragging.set(true);
                let mut tool_ctx = ToolContext { store };
                active_tool.write().on_node_drag_start(&mut tool_ctx, &node.id, &e);
            },
            onclick: move |e| {
                e.stop_propagation();
                let mut tool_ctx = ToolContext { store };
                active_tool.write().on_node_click(&mut tool_ctx, &node.id, &e);
            },
            CircleNode { radius, selected }
        }
    }
}
