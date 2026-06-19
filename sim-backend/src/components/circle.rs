use dioxus::prelude::*;
use dioxus::core::Element;
use dioxus::core_macro::{component, rsx};

#[component]
pub fn CircleNode(radius: f64, selected: bool) -> Element {
    let size = radius * 2.0;
    let style = format!("width: {size}px; height: {size}px;");

    let class = if selected {
        "nn-circle nn-circle--selected nn-select-none"
    } else {
        "nn-circle nn-select-none"
    };
    rsx! { div { style: style, class: class } }
}
