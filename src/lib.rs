use dioxus::prelude::*;

mod state;
mod views;
mod components;
mod hooks;
mod tools;
mod models;
mod settings;

use state::{AppStore, ViewType};
use views::{edit::EditView, configure::ConfigureView, execute::ExecuteView};
use hooks::error_listener::use_error_listener;

static TAILWIND: Asset = asset!("/assets/tailwind.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeId(pub i64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupId(pub i64);

pub async fn create_node() -> Option<i64> {
    static mut NEXT: i64 = 1000;
    unsafe {
        NEXT += 1;
        Some(NEXT)
    }
}

pub fn App() -> Element {
    let mut store = use_signal(AppStore::default);
    provide_context(store.clone());

    rsx! { FlowCanvas {} }
}

#[component]
fn FlowCanvas() -> Element {
    use_error_listener();

    let mut store = use_context::<Signal<AppStore>>();
    let current_view = store.read().current_view;

    let topbar_class = "nn-topbar";

    rsx! {
        document::Stylesheet { href: TAILWIND }

        match current_view {
            ViewType::EDIT => rsx!( EditView {} ),
            ViewType::CONFIGURE => rsx!( ConfigureView {} ),
            ViewType::EXECUTE => rsx!( ExecuteView {} ),
        }

        div { class: topbar_class,
            TopbarButton { label: "EDIT", active: matches!(current_view, ViewType::EDIT), on_click: move |_| store.write().current_view = ViewType::EDIT }
            TopbarButton { label: "CONFIGURE", active: matches!(current_view, ViewType::CONFIGURE), on_click: move |_| store.write().current_view = ViewType::CONFIGURE }
            TopbarButton { label: "EXECUTE", active: matches!(current_view, ViewType::EXECUTE), on_click: move |_| store.write().current_view = ViewType::EXECUTE }
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
