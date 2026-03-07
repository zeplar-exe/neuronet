use dioxus::html::geometry::ClientPoint;
use dioxus::prelude::*;
use crate::state::{AppStore, ViewType, Neuron, Color, Group, Rect};
use crate::components::explorer::Explorer;
use crate::components::canvas::GlobalCanvas;
use crate::components::properties::Properties;
use crate::components::circle::CircleNode;
use crate::models::NeuronModelKind;
use crate::tools::{tool_by_name, ToolContext, ToolRequest};

#[component]
pub fn EditView() -> Element {
    let mut store = use_context::<Signal<AppStore>>();
    let tool_ctx = ToolContext { store: store.clone() };
    let mut active_tool: Signal<Box<dyn crate::tools::Tool>> = use_signal(|| {
        let name = store.read().set_view_state(ViewType::EDIT).selected_tool;
        tool_by_name(name)
    });

    let mut last_mouse_position: Signal<ClientPoint> = use_signal(|| ClientPoint::default());

    let onkeydown_canvas = {
        move |e: KeyboardEvent| {
            let m = e.modifiers();
            match e.key() {
                Key::Escape => {
                    store.write().clear_selection();
                }
                Key::Character(k) if (k.eq_ignore_ascii_case("a")) && (m.ctrl() || m.meta()) => {
                    e.prevent_default();
                    store.write().select_all();
                }
                Key::Character(k) if (k.eq_ignore_ascii_case("g")) && (m.ctrl() || m.meta()) => {
                    e.prevent_default();
                    store.write().group_selected(&"Group".to_string());
                }
                Key::Character(k) if k.eq_ignore_ascii_case("f") => {
                    e.prevent_default();

                    let s = store.read();
                    if s.selected.is_empty() { return; }

                    let mut sum_x = 0.0f64;
                    let mut sum_y = 0.0f64;
                    let mut count = 0.0f64;
                    
                    for id in s.selected.iter() {
                        if let Some(n) = s.network.neurons.iter().find(|n| &n.id == id) {
                            sum_x += n.position.0; sum_y += n.position.1; 
                            count += 1.0;
                        }
                    }
                    
                    drop(s);

                    if count > 0.0 {
                        let avg_x = sum_x / count;
                        let avg_y = sum_y / count;

                        store.write().pan_to((avg_x, avg_y), true);
                    }

                }
                Key::Character(k) => {
                    let ku = k.to_uppercase();
                    match ku.as_str() {
                        "S" => { let mut s = store.write(); s.set_tool(ViewType::EDIT, "select"); active_tool.set(tool_by_name("select")); },
                        "A" => { let mut s = store.write(); s.set_tool(ViewType::EDIT, "add"); active_tool.set(tool_by_name("add")); },
                        "E" => { let mut s = store.write(); s.set_tool(ViewType::EDIT, "edge"); active_tool.set(tool_by_name("edge")); }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    };

    let mut is_dragging = use_signal(|| false);
    let mut is_panning = use_signal(|| false);

    let node_centers: std::collections::HashMap<i64, (f64, f64)> = {
        let s = store.read();
        let mut map = std::collections::HashMap::new();
        for n in s.network.neurons.iter() {
            let r = 20.0;
            let cx = n.position.0 + r;
            let cy = n.position.1 + r;
            map.insert(n.id, (cx, cy));
        }
        map
    };

    let onclick_canvas = {
        let mut ctx = tool_ctx.clone();
        move |evt: MouseEvent| {
            active_tool.write().on_canvas_click(&mut ctx, &evt);
        }
    };
    let onmousedown_canvas = {
        let mut ctx = tool_ctx.clone();
        move |evt: MouseEvent| {
            let m = evt.modifiers();
            if m.alt() {
                is_panning.set(true);
                // Interrupt any ongoing pan animation when manual panning starts
                store.write().pan_anim = None;
            } else {
                is_dragging.set(true);
                active_tool.write().on_drag_start(&mut ctx, &evt);
            }
        }
    };
    let onmousemove_canvas = {
        let mut ctx = tool_ctx.clone();
        move |evt: MouseEvent| {
            let coords = evt.client_coordinates();
            
            if *is_dragging.read() {
                active_tool.write().on_drag(&mut ctx, &evt);
            } else if *is_panning.read() {
                let delta = coords - *last_mouse_position.read();
                let current_offset = store.read().canvas_offset.clone();
                let new_offset = (current_offset.0 + delta.x, current_offset.1 + delta.y);
                store.write().canvas_offset = new_offset;
            }

            last_mouse_position.set(coords);
        }
    };
    let onmouseup_canvas = {
        let mut ctx = tool_ctx.clone();
        move |evt: MouseEvent| {
            if *is_dragging.read() {
                active_tool.write().on_drag_end(&mut ctx, &evt);
                is_dragging.set(false);
            } else if *is_panning.read() {
                is_panning.set(false);
            }
        }
    };

    rsx! {
        GlobalCanvas {
            onkeydown: onkeydown_canvas,
            onclick: onclick_canvas,
            onmousedown: onmousedown_canvas,
            onmousemove: onmousemove_canvas,
            onmouseup: onmouseup_canvas,

            // Edge rendering (behind nodes)
            svg { style: "position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none;",
                for e in store.read().network.edges.iter() {
                    if let (Some(&(x1, y1)), Some(&(x2, y2))) = (node_centers.get(&e.source), node_centers.get(&e.target)) {
                        line { x1: "{x1}", y1: "{y1}", x2: "{x2}", y2: "{y2}", stroke: "#aab", stroke_width: "2" }
                    }
                }
            }
            for n in store.read().network.neurons.iter() {
                RenderNode { neuron: n.clone(), store: store, active_tool: active_tool, is_dragging: is_dragging }
            }

            for g in store.read().network.groups.iter() {
                RenderGroup { group: g.clone(), store: store }
            }

            if let Some(r) = active_tool.read().select_rect() {
                div { class: "nn-selectbox", style: format!("position: absolute; left: {0}px; top: {1}px; width: {2}px; height: {3}px;", r.left, r.top, r.width(), r.height()) }
            }
        }

        div { class: "nn-toolbar",
            ToolButton { label: "SEL [S]", active: &*active_tool.read().name() == "select", onclick: move |_| { store.write().set_tool(ViewType::EDIT, "select"); active_tool.set(tool_by_name("select")); } }
            ToolButton { label: "ADD [A]", active: &*active_tool.read().name() == "add", onclick: move |_| { store.write().set_tool(ViewType::EDIT, "add"); active_tool.set(tool_by_name("add")); } }
            ToolButton { label: "EDG [E]", active: &*active_tool.read().name() == "edge", onclick: move |_| { store.write().set_tool(ViewType::EDIT, "edge"); active_tool.set(tool_by_name("edge")); } }
        }

        {
            let left_sidebar_class = format!("{} nn-sidebar-left", "nn-sidebar");
            let right_sidebar_class = format!("{} nn-sidebar-right", "nn-sidebar");

            rsx! {
                aside { class: left_sidebar_class, Explorer {} }
                aside { class: right_sidebar_class, Properties {} }
            }
        }

        if let Some(ToolRequest::AddPopulation{ rect }) = active_tool.read().request().clone() {
            AddPopulationModal { store: store.clone(), active_tool: active_tool.clone(), rect: rect }
        }
    }
}

#[component]
fn ToolButton(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let class = if active { "nn-toolbtn nn-toolbtn--active" } else { "nn-toolbtn" };
    rsx! { button { class: class, onclick: move |e| onclick.call(e), "{label}" } }
}

#[component]
fn RenderNode(
    neuron: Neuron,
    store: Signal<AppStore>,
    active_tool: Signal<Box<dyn crate::tools::Tool>>,
    is_dragging: Signal<bool>,
) -> Element {
    let radius = 20.0;
    let selected = store.read().selected.iter().any(|sid| sid == &neuron.id);
    let left = neuron.position.0;
    let top = neuron.position.1;
    let node_style = format!("position: absolute; left: {left}px; top: {top}px");

    let node_for_down = neuron.clone();
    let node_for_click = neuron.clone();

    rsx! {
        div { style: node_style, class: "nn-select-none",
            onmousedown: move |e| {
                e.stop_propagation();
                is_dragging.set(true);
                let mut tool_ctx = ToolContext { store };
                active_tool.write().on_node_drag_start(&mut tool_ctx, &node_for_down, &e);
            },
            onclick: move |e| {
                e.stop_propagation();
                let mut tool_ctx = ToolContext { store };
                active_tool.write().on_node_click(&mut tool_ctx, &node_for_click, &e);
            },
            CircleNode { radius, selected }
        }
    }
}

#[component]
fn RenderGroup(group: Group, store: Signal<AppStore>) -> Element {
    let (x, y, w, h) = group.rect.as_tuple_wh();
    let hex = group.color.hex();
    let group_style = format!("position: absolute; left: {x}px; top: {y}px; width: {w}px; height: {h}px; background-color: {hex};");

    rsx! {
        div { style: group_style,

        }
    }
}

#[component]
fn AddPopulationModal(store: Signal<AppStore>, active_tool: Signal<Box<dyn crate::tools::Tool>>, rect: Rect) -> Element {
    let mut name = use_signal(|| String::from("Population"));
    let mut arrangement = use_signal(|| String::from("Poisson"));
    let mut count = use_signal(|| String::from("10"));

    let on_cancel = {
        move |_| {
            active_tool.write().clear_request();
        }
    };

    let add_population = {
        move || {
            let name = name.read().clone();
            let arrangement = arrangement.read().clone();
            let count_val: usize = count.read().parse().unwrap_or(0);
            if count_val == 0 { let mut at = active_tool.clone(); at.write().clear_request(); return; }

            let rect = rect.clone();

            spawn(async move {
                let (x, y, w, h) = rect.as_tuple_wh();

                let grid_cols = (count_val as f64).sqrt().ceil() as usize;
                let grid_rows = ((count_val + grid_cols - 1) / grid_cols).max(1);
                let cell_w = if grid_cols > 0 { w / grid_cols as f64 } else { w };
                let cell_h = if grid_rows > 0 { h / grid_rows as f64 } else { h };
                let radius = 20.0;
                
                let mut group = Group {
                    id: store.read().network.get_next_group_id(),
                    name: name.clone(),
                    parent: 0,
                    neurons: vec![],
                    groups: vec![],
                    rect,
                    color: Color::default()
                };

                for i in 0..count_val {
                    let (px, py) = match arrangement.as_str() {
                        "Grid" => {
                            let r = i / grid_cols;
                            let c = i % grid_cols;
                            let cx = x + c as f64 * cell_w + cell_w * 0.5;
                            let cy = y + r as f64 * cell_h + cell_h * 0.5;
                            (cx - radius, cy - radius)
                        }
                        "Gaussian" => {
                            let u1 = rand::random::<f64>().max(1e-6).min(0.999999);
                            let u2 = rand::random::<f64>();
                            let _ln = u1.ln();
                            let z0 = (-2.0 * u1).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                            let z1 = (-2.0 * u1).sqrt() * (2.0 * std::f64::consts::PI * u2).sin();
                            let cx = x + w * 0.5 + z0 * (w * 0.2);
                            let cy = y + h * 0.5 + z1 * (h * 0.2);
                            (cx.clamp(x, x + w), cy.clamp(y, y + h))
                        }
                        "Poisson" | "Random" | _ => {
                            let rx = x + rand::random::<f64>() * w;
                            let ry = y + rand::random::<f64>() * h;
                            (rx - radius, ry - radius)
                        }
                    };

                    let mut s = store.write();
                    let kind = NeuronModelKind::default();
                    let state_index = s.get_next_state_index(&kind);
                    let id = s.network.get_next_neuron_id();

                    s.push_neuron(Neuron {
                        id,
                        model: kind,
                        state_index,
                        position: (px, py),
                        parent: group.id
                    });

                    group.neurons.push(id);
                }

                store.write().network.groups.push(group);
                active_tool.write().clear_request();
            });
        }
    };

    let on_add = {
        let add = add_population.clone();

        move || {
            add();
        }
    };

    let on_keydown_modal = {
        let add = add_population.clone();

        move |e: KeyboardEvent| {
            if e.key() == Key::Escape {
                e.stop_propagation();
                e.prevent_default();
                active_tool.write().clear_request();
            } else if e.key() == Key::Enter {
                e.stop_propagation();
                add();
            }
        }
    };

    rsx! {
        div { class: "nn-modal-backdrop", onclick: on_cancel.clone() }
        div { class: "nn-modal", onkeydown: on_keydown_modal,
            h3 { class: "text-lg font-semibold mb-2", "Add Population" }
            div { class: "mb-2",
                label { class: "block mb-1 text-sm", "Name" }
                input {
                    class: "w-full px-2 py-1 rounded bg-[#1e1e1e] text-[#f6f6f6] border border-[rgba(255,255,255,0.12)]",
                    value: name(),
                    oninput: move |e| name.set(e.value().to_string())
                }
            }
            div { class: "mb-2",
                label { class: "block mb-1 text-sm", "Arrangement" }
                select {
                    class: "w-full px-2 py-1 rounded bg-[#1e1e1e] text-[#f6f6f6] border border-[rgba(255,255,255,0.12)]",
                    value: arrangement(),
                    oninput: move |e| arrangement.set(e.value().to_string()),
                    option { value: "Poisson", "Poisson" }
                    option { value: "Gaussian", "Gaussian" }
                    option { value: "Random", "Random" }
                    option { value: "Grid", "Grid" }
                }
            }
            div { class: "mb-4",
                label { class: "block mb-1 text-sm", "Count" }
                input {
                    r#type: "number",
                    min: "1",
                    class: "w-full px-2 py-1 rounded bg-[#1e1e1e] text-[#f6f6f6] border border-[rgba(255,255,255,0.12)]",
                    value: count(),
                    oninput: move |e| count.set(e.value().to_string())
                }
            }
            div { class: "flex justify-end gap-2",
                button { class: "nn-toolbtn", onclick: on_cancel, "Cancel" }
                button { class: "nn-toolbtn nn-toolbtn--active", onclick: move |_| on_add(), "Add" }
            }
        }
    }
}
