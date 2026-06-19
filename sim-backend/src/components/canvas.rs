use dioxus::prelude::*;
use crate::state::AppStore;

#[component]
pub fn GlobalCanvas(
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    onclick: Option<EventHandler<MouseEvent>>,
    onmousedown: Option<EventHandler<MouseEvent>>,
    onmousemove: Option<EventHandler<MouseEvent>>,
    onmouseup: Option<EventHandler<MouseEvent>>,
    onwheel: Option<EventHandler<WheelEvent>>,
    oncontextmenu: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let store = use_context::<Signal<AppStore>>();

    let (ox, oy) = store.read().canvas_offset;
    let zoom = store.read().canvas_zoom;

    let layer_style = format!(
        "position: absolute; inset: 0; transform: translate({ox}px, {oy}px) scale({zoom}); transform-origin: 0 0;"
    );

    rsx! {
        div {
            class: "nn-canvas",
            tabindex: 0,
            onkeydown: move |e| if let Some(h) = &onkeydown { h.call(e) },
            onclick: move |e| if let Some(h) = &onclick { h.call(e) },
            onmousedown: move |e| if let Some(h) = &onmousedown { h.call(e) },
            onmousemove: move |e| if let Some(h) = &onmousemove { h.call(e) },
            onmouseup: move |e| if let Some(h) = &onmouseup { h.call(e) },
            onwheel: move |e| if let Some(h) = &onwheel { h.call(e) },
            oncontextmenu: move |e| if let Some(h) = &oncontextmenu { h.call(e) },

            div { style: layer_style,
                {children}
            }
        }
    }
}
