use dioxus::prelude::*;

mod state;
mod tauri_api;
mod views;
mod components;
mod flow;
mod hooks;

use state::{AppStore, ViewType};
use views::{edit::EditView, configure::ConfigureView, execute::ExecuteView};
use hooks::error_listener::use_error_listener;
use tauri_api::{get_node, get_nodes};

pub fn App() -> Element {
    // Provide global store
    let mut store = use_signal(AppStore::default);
    provide_context(store.clone());

    rsx! { FlowCanvas {} }
}

#[component]
fn FlowCanvas() -> Element {
    use_error_listener();

    let mut store = use_context::<Signal<AppStore>>();
    let current_view = store.read().current_view;

    // Initial data load (equivalent to useEffect in React)
    let _loader = {
        let mut store = store.clone();
        use_future(move || async move {
            let ids = get_nodes().await.unwrap_or_default();
            let mut nodes = Vec::new();
            for id in ids {
                if let Some(n) = get_node(id).await {
                    nodes.push(state::Node {
                        id: n.id.to_string(),
                        r#type: Some("circle".to_string()),
                        data: serde_json::json!({
                            "label": n.id,
                            "radius": 20
                        }),
                        position: (n.pos_x, n.pos_y),
                        parent: n.parent,
                    });
                }
            }
            store.write().set_nodes(nodes);
        })
    };

    let topbar_style = "position: fixed;
        top: 0;
        left: 0;
        right: 0;
        height: 56px;
        background: rgba(26,26,26,0.95);
        display: flex;
        align-items: center;
        box-shadow: 0 2px 8px rgba(0,0,0,0.35);
        z-index: 40;
        padding: 0 16px;
        color: #f6f6f6;
        gap: 16px;".to_string();

    rsx! {
        match current_view {
            ViewType::EDIT => rsx!( EditView {} ),
            ViewType::CONFIGURE => rsx!( ConfigureView {} ),
            ViewType::EXECUTE => rsx!( ExecuteView {} ),
        }

        div { style: topbar_style,
            TopbarButton { label: "EDIT", active: matches!(current_view, ViewType::EDIT), on_click: move |_| store.write().current_view = ViewType::EDIT }
            TopbarButton { label: "CONFIGURE", active: matches!(current_view, ViewType::CONFIGURE), on_click: move |_| store.write().current_view = ViewType::CONFIGURE }
            TopbarButton { label: "EXECUTE", active: matches!(current_view, ViewType::EXECUTE), on_click: move |_| store.write().current_view = ViewType::EXECUTE }
        }
    }
}

#[component]
fn TopbarButton(label: String, active: bool, on_click: EventHandler<MouseEvent>) -> Element {
    let font_weight = if active { "700" } else { "400" };
    let style = format!("cursor: pointer; font-weight: {font_weight}; margin-right: 16px;");

    rsx! { div { style: style, onclick: move |e| on_click.call(e), "{label}" } }
}
