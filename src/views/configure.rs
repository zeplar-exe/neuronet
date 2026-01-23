use dioxus::prelude::*;

#[component]
pub fn ConfigureView() -> Element {
    let canvas_style = "position: absolute; top: 56px; left: 220px; right: 220px; bottom: 0; overflow: hidden; display: flex; align-items: center; justify-content: center; color: #888; font-size: 24px;".to_string();

    let sidebar_style = "position: fixed; top: 56px; bottom: 0; width: 220px; background: rgba(26,26,26,0.98); box-shadow: 0 2px 8px rgba(0,0,0,0.35); z-index: 30; padding: 12px; overflow: auto; color: #f6f6f6;".to_string();

    rsx! {
        div { style: canvas_style, "Configure View - Coming Soon" }
        {
            let left_sidebar_style = format!("{} left: 0;", sidebar_style);
            let right_sidebar_style = format!("{} right: 0;", sidebar_style);
            rsx! {
                aside { style: left_sidebar_style, div { "Configuration" } }
                aside { style: right_sidebar_style, div { "Settings" } }
            }
        }
    }
}
