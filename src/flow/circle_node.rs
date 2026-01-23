use dioxus::prelude::*;

#[component]
pub fn CircleNode(label: String, radius: f64, selected: bool) -> Element {
    let size = radius * 2.0;
    let bg = if selected { "#396cd8" } else { "#0077ff" };
    let style = format!(
        "width: {size}px; height: {size}px; border-radius: 50%; display: flex; justify-content: center; align-items: center; background-color: {bg}ff; color: white; font-size: 10px; user-select: none;"
    );
    rsx! { div { style: style, "{label}" } }
}
