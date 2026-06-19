use dioxus::prelude::*;

mod common;
mod components;
mod gen;
mod models;
mod settings;
mod simulation;
mod state;
mod tools;
mod util;
mod views;

use state::{AppStore, ViewType};
use views::{edit::EditView, execute::ExecuteView};

pub fn App() -> Element {
    let mut store = use_signal(AppStore::default);
    provide_context(store.clone());

    let current_view = store.read().current_view;
    let topbar_class = "nn-topbar";

    rsx! {
        Stylesheet { href: asset!("/assets/main.css") }
        Stylesheet { href: asset!("/assets/view.css") }

        match current_view {
            ViewType::EDIT => rsx!( EditView {} ),
            ViewType::EXECUTE => rsx!( ExecuteView {} ),
        }

        div { class: topbar_class,
            // Left: view tabs
            TopbarButton { label: "EDIT", active: matches!(current_view, ViewType::EDIT), on_click: move |_| store.write().current_view = ViewType::EDIT }
            TopbarButton { label: "EXECUTE", active: matches!(current_view, ViewType::EXECUTE), on_click: move |_| store.write().current_view = ViewType::EXECUTE }

            // Spacer
            div { style: "flex: 1" }

            // Right: Run State switcher
            {

            }
        }
    }
}

#[component]
fn TopbarButton(label: String, active: bool, on_click: EventHandler<MouseEvent>) -> Element {
    let class = if active {
        "cursor-pointer mr-4 font-bold"
    } else {
        "cursor-pointer mr-4 font-normal"
    };
    rsx! { div { class: class, onclick: move |e| on_click.call(e), "{label}" } }
}
