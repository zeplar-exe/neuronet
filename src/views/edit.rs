use dioxus::prelude::*;
use crate::state::{AppStore, ViewType, PrimaryTool, Node};
use crate::components::explorer::Explorer;
use crate::flow::circle_node::CircleNode;
use crate::tauri_api::create_node;

#[component]
pub fn EditView() -> Element {
    let mut store = use_context::<Signal<AppStore>>();

    // Keyboard shortcuts S/A/E
    #[cfg(feature = "web")]
    {
        use dioxus::prelude::use_effect;
        use wasm_bindgen::JsCast;
        use web_sys::{window, KeyboardEvent};
        let mut store = store.clone();
        use_effect(move || {
            let window = window().unwrap();
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |e: KeyboardEvent| {
                let key = e.key().to_uppercase();
                let mut s = store.write();
                match key.as_str() {
                    "S" => s.set_tool(ViewType::EDIT, PrimaryTool { name: "select" }),
                    "A" => s.set_tool(ViewType::EDIT, PrimaryTool { name: "add" }),
                    "E" => s.set_tool(ViewType::EDIT, PrimaryTool { name: "edge" }),
                    _ => {}
                }
            }) as Box<dyn FnMut(_)>);
            window
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .ok();
            move || {
                let _ = window.remove_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
                drop(closure);
            }
        });
    }
    #[cfg(not(feature = "web"))]
    {
        // No-op on desktop/mobile; keyboard shortcuts not wired
    }

    let canvas_style = "position: absolute; top: 56px; left: 220px; right: 220px; bottom: 0; overflow: hidden; background: transparent;".to_string();

    let toolbar_style = "position: fixed; left: 50%; transform: translateX(-50%); bottom: 12px; display: flex; gap: 8px; padding: 8px 12px; background: rgba(26,26,26,0.95); border-radius: 10px; box-shadow: 0 6px 20px rgba(0,0,0,0.4); z-index: 50; color: #f6f6f6;".to_string();

    let sidebar_style = "position: fixed; top: 56px; bottom: 0; width: 220px; background: rgba(26,26,26,0.98); box-shadow: 0 2px 8px rgba(0,0,0,0.35); z-index: 30; padding: 12px; overflow: auto; color: #f6f6f6;".to_string();

    // Click handling similar to tools
    let onclick_canvas = move |evt: MouseEvent| {
        let tool = store.read().view_state(ViewType::EDIT).selected_tool.name;
        match tool {
            "select" => {
                store.write().clear_selection();
            }
            "add" => {
                let client_x = evt.client_coordinates().x as f64;
                let client_y = evt.client_coordinates().y as f64;
                // naive screen coords used as canvas coords (no transform)
                let radius = 20.0;
                let mut store2 = store.clone();
                spawn(async move {
                    let id = create_node().await.unwrap_or(0);
                    let mut s = store2.write();
                    s.push_node(Node {
                        id: id.to_string(),
                        r#type: Some("circle".to_string()),
                        data: serde_json::json!({"label": id, "radius": radius }),
                        position: (client_x - radius, client_y - radius),
                        parent: 0,
                    });
                });
            }
            "edge" => {
                // placeholder
                eprintln!("Edge tool clicked");
            }
            _ => {}
        }
    };

    rsx! {
        // Canvas area
        div { style: canvas_style, onclick: onclick_canvas,
            for n in store.read().nodes.iter() {
                RenderNode { node: n.clone(), store: store.clone() }
            }
        }

        // Toolbar
        div { style: toolbar_style,
            ToolButton { label: "SEL [S]", active: store.read().view_state(ViewType::EDIT).selected_tool.name == "select", onclick: move |_| store.write().set_tool(ViewType::EDIT, PrimaryTool { name: "select" }) }
            ToolButton { label: "ADD [A]", active: store.read().view_state(ViewType::EDIT).selected_tool.name == "add", onclick: move |_| store.write().set_tool(ViewType::EDIT, PrimaryTool { name: "add" }) }
            ToolButton { label: "EDG [E]", active: store.read().view_state(ViewType::EDIT).selected_tool.name == "edge", onclick: move |_| store.write().set_tool(ViewType::EDIT, PrimaryTool { name: "edge" }) }
        }

        // Sidebars
        {
            let left_sidebar_style = format!("{} left: 0;", sidebar_style);
            let right_sidebar_style = format!("{} right: 0;", sidebar_style);
            rsx! {
                aside { style: left_sidebar_style, Explorer {} }
                aside { style: right_sidebar_style, }
            }
        }
    }
}

#[component]
fn ToolButton(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let (bg, color, border) = if active {
        ("#396cd8", "#fff", "1px solid #2f5fc0")
    } else {
        ("transparent", "#f6f6f6", "1px solid rgba(255,255,255,0.1)")
    };
    let style = format!(
        "background: {bg}; color: {color}; border: {border}; padding: 8px 12px; border-radius: 6px; cursor: pointer;"
    );
    rsx! { button { style: style, onclick: move |e| onclick.call(e), "{label}" } }
}

#[component]
fn RenderNode(node: crate::state::Node, store: Signal<crate::state::AppStore>) -> Element {
    let label = if let Some(l) = node.data.get("label").and_then(|v| v.as_i64()) { format!("{}", l) } else { node.id.clone() };
    let radius = node.data.get("radius").and_then(|v| v.as_f64()).unwrap_or(20.0);
    let selected = store.read().selected.iter().any(|sid| sid == &node.id);
    let node_id = node.id.clone();
    let left = node.position.0;
    let top = node.position.1;
    let node_style = format!("position: absolute; left: {left}px; top: {top}px;");
    rsx! {
        div { style: node_style,
            onclick: move |e| {
                e.stop_propagation();
                let ctrl = e.modifiers().ctrl();
                if ctrl {
                    store.write().toggle_selected(&node_id);
                } else {
                    store.write().select_only(&node_id);
                }
            },
            CircleNode { label, radius, selected }
        }
    }
}
