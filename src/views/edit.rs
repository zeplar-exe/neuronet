use dioxus::prelude::*;
use crate::state::{AppStore, ViewType, PrimaryTool};
use crate::components::explorer::Explorer;
use crate::components::properties::Properties;
use crate::components::circle::CircleNode;
use crate::models::NeuronModelKind;
use crate::tools::{tool_by_name, ToolContext, ToolRequest};

#[component]
pub fn EditView() -> Element {
    let mut store = use_context::<Signal<AppStore>>();
    let mut tool_ctx = ToolContext { store: store.clone() };
    // Keep a stateful instance of the currently active tool
    let mut active_tool: Signal<Box<dyn crate::tools::Tool>> = use_signal(|| {
        let name = store.read().view_state(ViewType::EDIT).selected_tool.name;
        tool_by_name(name)
    });
    // View offset for focusing/centering (must exist before handlers)
    let canvas_offset: Signal<(f64, f64)> = use_signal(|| (0.0, 0.0));

    // Dioxus-native keyboard shortcuts (Desktop & Web)
    let onkeydown_canvas = {
        let mut store = store.clone();
        let mut active_tool = active_tool.clone();
        let mut canvas_offset_ref = canvas_offset.clone();
        move |e: KeyboardEvent| {
            // First handle global shortcuts
            match e.key() {
                Key::Escape => {
                    store.write().clear_selection();
                }
                Key::Character(k) if (k.eq_ignore_ascii_case("a")) && (e.modifiers().ctrl() || e.modifiers().meta()) => {
                    // Select all
                    e.prevent_default();
                    store.write().select_all();
                }
                Key::Character(k) if (k.eq_ignore_ascii_case("g")) && (e.modifiers().ctrl() || e.modifiers().meta()) => {
                    // Group selected nodes and groups
                    e.prevent_default();
                    store.write().group_selected(None);
                }
                Key::Character(k) if k.eq_ignore_ascii_case("f") => {
                    // Focus: center view on average position of selected nodes
                    let s = store.read();
                    if s.selected.is_empty() { return; }
                    let mut sum_x = 0.0f64; let mut sum_y = 0.0f64; let mut count = 0.0f64;
                    for id in s.selected.iter() {
                        if let Some(n) = s.nodes.iter().find(|n| &n.id == id) {
                            let r = 20.0;
                            sum_x += n.position.0 + r; sum_y += n.position.1 + r; count += 1.0;
                        }
                    }
                    if count > 0.0 {
                        let avg_x = sum_x / count;
                        let avg_y = sum_y / count;
                        // Approximate canvas center (account for sidebars/topbar). If actual size is unknown, use a sensible default 800x600 viewport
                        let canvas_center_x = 220.0 + 400.0; // left sidebar + half of 800
                        let canvas_center_y = 56.0 + 300.0;  // topbar + half of 600
                        let dx = canvas_center_x - avg_x;
                        let dy = canvas_center_y - avg_y;
                        canvas_offset_ref.set((dx, dy));
                    }
                }
                Key::Character(k) => {
                    let ku = k.to_uppercase();
                    match ku.as_str() {
                        "S" => { let mut s = store.write(); s.set_tool(ViewType::EDIT, PrimaryTool { name: "select" }); active_tool.set(tool_by_name("select")); },
                        "A" => { let mut s = store.write(); s.set_tool(ViewType::EDIT, PrimaryTool { name: "add" }); active_tool.set(tool_by_name("add")); },
                        "E" => { let mut s = store.write(); s.set_tool(ViewType::EDIT, PrimaryTool { name: "edge" }); active_tool.set(tool_by_name("edge")); },
                        "R" => { let mut s = store.write(); s.set_tool(ViewType::EDIT, PrimaryTool { name: "region" }); active_tool.set(tool_by_name("region")); },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    };

    // Classes moved to Tailwind CSS (tailwind.css)
    let canvas_class = "nn-canvas";
    let toolbar_class = "nn-toolbar";
    let sidebar_base_class = "nn-sidebar";

    // Drag state for tools
    let is_dragging = use_signal(|| false);

    // Precompute node centers for edge rendering
    let node_centers: std::collections::HashMap<i64, (f64, f64)> = {
        let s = store.read();
        let mut map = std::collections::HashMap::new();
        for n in s.nodes.iter() {
            let r = 20.0;
            let cx = n.position.0 + r;
            let cy = n.position.1 + r;
            map.insert(n.id, (cx, cy));
        }
        map
    };

    // canvas_offset already defined above

    let onclick_canvas = {
        let tool_ctx = tool_ctx.clone();
        let mut active_tool = active_tool.clone();
        move |evt: MouseEvent| {
            let mut ctx = tool_ctx.clone();
            active_tool.write().on_canvas_click(&mut ctx, &evt);
        }
    };
    let onmousedown_canvas = {
        let tool_ctx = tool_ctx.clone();
        let mut is_dragging = is_dragging.clone();
        let mut active_tool = active_tool.clone();
        move |evt: MouseEvent| {
            is_dragging.set(true);
            let mut ctx = tool_ctx.clone();
            active_tool.write().on_drag_start(&mut ctx, &evt);
        }
    };
    let onmousemove_canvas = {
        let tool_ctx = tool_ctx.clone();
        let is_dragging = is_dragging.clone();
        let mut active_tool = active_tool.clone();
        move |evt: MouseEvent| {
            if *is_dragging.read() {
                let mut ctx = tool_ctx.clone();
                active_tool.write().on_drag(&mut ctx, &evt);
            }
        }
    };
    let onmouseup_canvas = {
        let tool_ctx = tool_ctx.clone();
        let mut is_dragging = is_dragging.clone();
        let mut active_tool = active_tool.clone();
        move |evt: MouseEvent| {
            if *is_dragging.read() {
                let mut ctx = tool_ctx.clone();
                active_tool.write().on_drag_end(&mut ctx, &evt);
                is_dragging.set(false);
            }
        }
    };

    rsx! {
        // Canvas area
        div { class: canvas_class, tabindex: 0, onkeydown: onkeydown_canvas, onclick: onclick_canvas, onmousedown: onmousedown_canvas, onmousemove: onmousemove_canvas, onmouseup: onmouseup_canvas,
            // Inner translated layer for panning/focusing
            {
                let (ox, oy) = *canvas_offset.read();
                let layer_style = format!("position: absolute; inset: 0; transform: translate({ox}px, {oy}px);");
                rsx! {
                    div { style: layer_style,
                        // Edge rendering (behind nodes)
                        svg { style: "position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none;",
                            for e in store.read().edges.iter() {
                                if let (Some(&(x1, y1)), Some(&(x2, y2))) = (node_centers.get(&e.source), node_centers.get(&e.target)) {
                                    line { x1: "{x1}", y1: "{y1}", x2: "{x2}", y2: "{y2}", stroke: "#aab", stroke_width: "2" }
                                }
                            }
                        }
                        for n in store.read().nodes.iter() {
                            RenderNode { node: n.clone(), store: store.clone(), active_tool: active_tool.clone(), is_dragging: is_dragging.clone() }
                        }
                        // AddTool selection box overlay
                        if let Some((x, y, w, h)) = active_tool.read().select_rect() {
                            div { class: "nn-selectbox", style: format!("position: absolute; left: {x}px; top: {y}px; width: {w}px; height: {h}px;") }
                        }
                    }
                }
            }
        }

        // Toolbar
        div { class: toolbar_class,
            ToolButton { label: "SEL [S]", active: store.read().view_state(ViewType::EDIT).selected_tool.name == "select", onclick: move |_| { store.write().set_tool(ViewType::EDIT, PrimaryTool { name: "select" }); let mut t = active_tool.clone(); t.set(tool_by_name("select")); } }
            ToolButton { label: "ADD [A]", active: store.read().view_state(ViewType::EDIT).selected_tool.name == "add", onclick: move |_| { store.write().set_tool(ViewType::EDIT, PrimaryTool { name: "add" }); let mut t = active_tool.clone(); t.set(tool_by_name("add")); } }
            ToolButton { label: "EDG [E]", active: store.read().view_state(ViewType::EDIT).selected_tool.name == "edge", onclick: move |_| { store.write().set_tool(ViewType::EDIT, PrimaryTool { name: "edge" }); let mut t = active_tool.clone(); t.set(tool_by_name("edge")); } }
            ToolButton { label: "REG [R]", active: store.read().view_state(ViewType::EDIT).selected_tool.name == "region", onclick: move |_| { store.write().set_tool(ViewType::EDIT, PrimaryTool { name: "region" }); let mut t = active_tool.clone(); t.set(tool_by_name("region")); } }
        }

        // Sidebars
        {
            let left_sidebar_class = format!("{} left-0", sidebar_base_class);
            let right_sidebar_class = format!("{} right-0", sidebar_base_class);
            rsx! {
                // Properties panel (left) – keep empty for now or future properties UI
                aside { class: left_sidebar_class, Properties {} }
                aside { class: right_sidebar_class, Explorer {} }
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
    node: crate::state::Node,
    store: Signal<crate::state::AppStore>,
    active_tool: Signal<Box<dyn crate::tools::Tool>>, 
    is_dragging: Signal<bool>,
) -> Element {
    let radius = 20.0;
    let selected = store.read().selected.iter().any(|sid| sid == &node.id);
    let node_id = node.id;
    let left = node.position.0;
    let top = node.position.1;
    let node_style = format!("position: absolute; left: {left}px; top: {top}px;");
    // Clone node for use in multiple move closures
    let node_for_down = node.clone();
    let node_for_click = node.clone();
    rsx! {
        div { style: node_style, class: "nn-select-none",
            onmousedown: move |e| {
                // Start node drag; prevent canvas drag-start from firing first
                e.stop_propagation();
                is_dragging.set(true);
                let this_node = crate::state::Node { id: node_id, ..node_for_down.clone() };
                let mut tool_ctx2 = ToolContext { store: store.clone() };
                active_tool.write().on_node_drag_start(&mut tool_ctx2, &this_node, &e);
            },
            onclick: move |e| {
                e.stop_propagation();
                // Call tool-defined node click on the active tool instance
                let this_node = crate::state::Node { id: node_id, ..node_for_click.clone() };
                let mut tool_ctx2 = ToolContext { store: store.clone() };
                active_tool.write().on_node_click(&mut tool_ctx2, &this_node, &e);
            },
            CircleNode { radius, selected }
        }
    }
}

#[component]
fn AddPopulationModal(store: Signal<AppStore>, active_tool: Signal<Box<dyn crate::tools::Tool>>, rect: (f64, f64, f64, f64)) -> Element {
    let mut name = use_signal(|| String::from("Population"));
    let mut arrangement = use_signal(|| String::from("Poisson"));
    let mut count = use_signal(|| String::from("10"));

    let on_cancel = {
        let mut active_tool = active_tool.clone();
        move |_| {
            active_tool.write().clear_request();
        }
    };

    let on_add = {
        let mut store = store.clone();
        let mut active_tool = active_tool.clone();
        let name = name.clone();
        let arrangement = arrangement.clone();
        let count = count.clone();
        move |_| {
            let name = name.read().clone();
            let arrangement = arrangement.read().clone();
            let count_val: usize = count.read().parse().unwrap_or(0);
            if count_val == 0 { active_tool.write().clear_request(); return; }

            let mut store_signal = store.clone();
            let mut active_tool2 = active_tool.clone();
            let rect_local = rect;
            dioxus::core::spawn(async move {
                let (x, y, w, h) = rect_local;
                // Simple LCG for pseudo-random without external deps
                let mut seed: u64 = 0x853c49e6748fea9b;
                let mut next_rand = || {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                    ((seed >> 33) as f64) / ((1u64 << 31) as f64)
                };

                // Compute grid dims for grid arrangement
                let grid_cols = (count_val as f64).sqrt().ceil() as usize;
                let grid_rows = ((count_val + grid_cols - 1) / grid_cols).max(1);
                let cell_w = if grid_cols > 0 { w / grid_cols as f64 } else { w };
                let cell_h = if grid_rows > 0 { h / grid_rows as f64 } else { h };
                let radius = 20.0;

                // Create a group for this population, named after the population
                let group_id = {
                    let mut s = store_signal.write();
                    let next_id = s.groups.iter().map(|g| g.id).max().unwrap_or(0) + 1;
                    s.groups.push(crate::state::Group {
                        id: next_id,
                        name: name.clone(),
                        parent: 0,
                        children: vec![],
                    });
                    next_id
                };

                for i in 0..count_val {
                    let id = crate::create_node().await.unwrap_or(0);
                    let (px, py) = match arrangement.as_str() {
                        "Grid" => {
                            let r = i / grid_cols;
                            let c = i % grid_cols;
                            let cx = x + c as f64 * cell_w + cell_w * 0.5;
                            let cy = y + r as f64 * cell_h + cell_h * 0.5;
                            (cx - radius, cy - radius)
                        }
                        "Gaussian" => {
                            // Box-Muller using our LCG
                            let u1 = (next_rand().max(1e-6)).min(0.999999);
                            let u2 = next_rand();
                            let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                            let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).sin();
                            let cx = x + w * 0.5 + z0 * (w * 0.2);
                            let cy = y + h * 0.5 + z1 * (h * 0.2);
                            (cx.clamp(x, x + w) - radius, cy.clamp(y, y + h) - radius)
                        }
                        "Poisson" | "Random" | _ => {
                            let rx = x + next_rand() * w;
                            let ry = y + next_rand() * h;
                            (rx - radius, ry - radius)
                        }
                    };

                    let mut s = store_signal.write();
                    s.push_node(crate::state::Node {
                        id: id,
                        model: NeuronModelKind::default(),
                        position: (px, py),
                        parent: group_id,
                    });
                    if let Some(gr) = s.groups.iter_mut().find(|g| g.id == group_id) {
                        gr.children.push(id);
                    }
                }
                // Clear tool request/overlay after adding
                active_tool2.write().clear_request();
            });
        }
    };

    rsx! {
        // Backdrop
        div { class: "nn-modal-backdrop", onclick: on_cancel.clone() }
        // Modal
        div { class: "nn-modal",
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
                button { class: "nn-toolbtn nn-toolbtn--active", onclick: on_add, "Add" }
            }
        }
    }
}
