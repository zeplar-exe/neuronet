use crate::{
    r#gen::generate_group_name,
    state::{AppStore, ContextMenuTarget},
};
use dioxus::prelude::*;

#[component]
pub fn GlobalContextMenu() -> Element {
    let mut store = use_context::<Signal<AppStore>>();

    rsx! {
        if let Some(menu) = store.read().context_menu.clone() {
            {
                // Pre-capture data for handlers to satisfy borrow rules
                let target_for_select = menu.target.clone();
                let target_for_delete = menu.target.clone();
                let group_name = generate_group_name();
                rsx! {
                    // Click-away overlay (very high z-index to sit above everything). Use inline styles for reliability.
                    div { style: "position: fixed; inset: 0; z-index: 999999; background: transparent; pointer-events: auto;",
                        tabindex: 0,
                        onclick: move |_| store.write().close_context_menu(),
                        onmousedown: move |_| store.write().close_context_menu(),
                        oncontextmenu: move |e| { e.prevent_default(); store.write().close_context_menu(); },
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape { e.prevent_default(); store.write().close_context_menu(); }
                        }
                    }
                    // Menu popup
                    div { style: format!("position: absolute; left: {}px; top: {}px; z-index: 1000000;", menu.x, menu.y), class: "nn-menu",
                        tabindex: 0,
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape { e.prevent_default(); store.write().close_context_menu(); }
                        },

                        // Select
                        if let ContextMenuTarget::Neuron(_) | ContextMenuTarget::Group(_) = target_for_select.clone() {
                            div { class: "nn-menu-item",
                                onclick: move |_| {
                                    let mut s = store.write();
                                    match target_for_select {
                                        ContextMenuTarget::Neuron(nid) => s.select_only(nid),
                                        ContextMenuTarget::Group(gid) => s.select_group_only(gid),
                                        ContextMenuTarget::Canvas => {}
                                    }
                                    s.close_context_menu();
                                },
                                "Select"
                            }
                        }

                        // Rename
                        div { class: "nn-menu-item opacity-60", onclick: move |_| { store.write().close_context_menu(); }, "Rename (coming soon)" }

                        // Group selected
                        div { class: "nn-menu-item opacity-60",
                            onclick: move |_| {
                                let mut s = store.write();
                                s.group_selected(&generate_group_name());
                                s.close_context_menu();
                            },
                            div { "Group" }
                        }

                        // Delete
                        if let ContextMenuTarget::Neuron(_) | ContextMenuTarget::Group(_) = target_for_delete.clone() {
                            div { class: "nn-menu-item",
                                onclick: move |_| {
                                    let mut s = store.write();
                                    match target_for_delete {
                                        ContextMenuTarget::Neuron(nid) => s.delete_neuron(nid),
                                        ContextMenuTarget::Group(gid) => s.delete_group_recursive(gid),
                                        ContextMenuTarget::Canvas => {}
                                    }
                                    s.close_context_menu();
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}
