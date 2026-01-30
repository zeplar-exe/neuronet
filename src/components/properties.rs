use dioxus::prelude::*;
use std::collections::HashSet;
use crate::components::tooltip::{TooltipIndicator, TooltipKind};
use crate::models::{NeuronModelKind, NeuronModel};
use crate::state::AppStore;

#[component]
pub fn Properties() -> Element {
    let store = use_context::<Signal<AppStore>>();
    let expanded: Signal<HashSet<String>> = use_signal(Default::default);

    rsx! {
        div { class: "nn-props relative", tabindex: 0,
            div { class: "flex items-center justify-between mb-2",
                h3 { "Properties" }
            }
            RenderNeuronProperties { store: store.clone() }
        }
    }
}

#[component]
fn RenderNeuronProperties(store: Signal<AppStore>) -> Element {
    let (selected_neurons_count, unique_model_count, id_string, model_name_string, editable_snapshot): (usize, usize, String, String, Option<(i64, f64, f64, f64, f64, f64, f64)>) = {
        let s = store.read();
        let selected_ids = s.selected.clone();
        let selected_neurons_ids: Vec<i64> = s.nodes.iter().filter(|n| selected_ids.contains(&n.id)).map(|n| n.id).collect();
        let selected_neurons_count = selected_neurons_ids.len();

        let unique_model_names: HashSet<&'static str> = s.nodes.iter()
            .filter(|n| selected_ids.contains(&n.id))
            .map(|n| n.model.name())
            .collect();
        let unique_model_count = unique_model_names.len();

        let mut id = "<mixed>".to_string();
        let mut model_name = "<mixed>".to_string();
        let mut snapshot: Option<(i64, f64, f64, f64, f64, f64, f64)> = None;

        if selected_neurons_count == 1 {
            let nid = selected_neurons_ids[0];
            id = nid.to_string();
            if let Some(node) = s.nodes.iter().find(|n| n.id == nid) {
                model_name = node.model.name().to_string();
                if let NeuronModelKind::IntegrateFire(m) = &node.model {
                    snapshot = Some((
                        nid,
                        m.state.excitation,
                        m.config.reset_potential,
                        m.config.threshold,
                        m.config.strength,
                        m.config.minimum_voltage,
                        m.config.maximum_voltage,
                    ));
                }
            }
        } else if unique_model_count == 1 {
            model_name = unique_model_names.iter().next().copied().unwrap_or("").to_string();
        }
        (selected_neurons_count, unique_model_count, id, model_name, snapshot)
    };

    let mut section = rsx! { div {
        div { "{selected_neurons_count} neurons selected" }
        div { "{unique_model_count} model kinds selected" }
    }};

    if unique_model_count > 1 {
        section = rsx! { div { {section} TooltipIndicator {
            kind: TooltipKind::Warning,
            text: "Multiple model kinds selected. Cannot display unified property view."
        } } };
    } else if unique_model_count == 1 {
        let base = rsx! {
            ReadonlyStringProperty { label: "ID", value: "{id_string}" }
            ReadonlyStringProperty { label: "Model", value: "{model_name_string}" }
            ReadonlyStringProperty { label: "Tags", value: "" }
        };

        // If editable snapshot is present (IntegrateFire, single selection), show fields
        if let Some((nid, excitation, reset, threshold, strength, min_v, max_v)) = editable_snapshot {
            // Handlers per field that write directly to the store with validation
            let oninput_excitation = {
                let mut store = store.clone();
                move |e: FormEvent| {
                    if let Ok(mut v) = e.value().parse::<f64>() {
                        let mut s = store.write();
                        if let Some(n) = s.nodes.iter_mut().find(|n| n.id == nid) {
                            if let NeuronModelKind::IntegrateFire(m) = &mut n.model {
                                let min_v = m.config.minimum_voltage;
                                let max_v = m.config.maximum_voltage;
                                v = v.clamp(min_v, max_v);
                                m.state.excitation = v;
                            }
                        }
                    }
                }
            };
            let oninput_reset = {
                let mut store = store.clone();
                move |e: FormEvent| {
                    if let Ok(mut v) = e.value().parse::<f64>() {
                        let mut s = store.write();
                        if let Some(n) = s.nodes.iter_mut().find(|n| n.id == nid) {
                            if let NeuronModelKind::IntegrateFire(m) = &mut n.model {
                                let min_v = m.config.minimum_voltage;
                                let mut max_v = m.config.maximum_voltage;
                                if min_v > max_v { max_v = min_v; }
                                v = v.clamp(min_v, max_v);
                                m.config.reset_potential = v;
                                if m.config.threshold < v { m.config.threshold = v; }
                                m.state.excitation = m.state.excitation.clamp(min_v, max_v);
                            }
                        }
                    }
                }
            };
            let oninput_threshold = {
                let mut store = store.clone();
                move |e: FormEvent| {
                    if let Ok(mut v) = e.value().parse::<f64>() {
                        let mut s = store.write();
                        if let Some(n) = s.nodes.iter_mut().find(|n| n.id == nid) {
                            if let NeuronModelKind::IntegrateFire(m) = &mut n.model {
                                let reset = m.config.reset_potential;
                                let max_v = m.config.maximum_voltage;
                                if reset > v { v = reset; }
                                if v > max_v { v = max_v; }
                                m.config.threshold = v;
                            }
                        }
                    }
                }
            };
            let oninput_strength = {
                let mut store = store.clone();
                move |e: FormEvent| {
                    if let Ok(v) = e.value().parse::<f64>() {
                        let mut s = store.write();
                        if let Some(n) = s.nodes.iter_mut().find(|n| n.id == nid) {
                            if let NeuronModelKind::IntegrateFire(m) = &mut n.model { m.config.strength = v; }
                        }
                    }
                }
            };
            let oninput_min = {
                let mut store = store.clone();
                move |e: FormEvent| {
                    if let Ok(mut vmin) = e.value().parse::<f64>() {
                        let mut s = store.write();
                        if let Some(n) = s.nodes.iter_mut().find(|n| n.id == nid) {
                            if let NeuronModelKind::IntegrateFire(m) = &mut n.model {
                                let mut vmax = m.config.maximum_voltage;
                                if vmin > vmax { vmax = vmin; }
                                m.config.minimum_voltage = vmin;
                                m.config.maximum_voltage = vmax;
                                // adjust reset/threshold/excitation to range
                                let reset = m.config.reset_potential.clamp(vmin, vmax);
                                m.config.reset_potential = reset;
                                if m.config.threshold < reset { m.config.threshold = reset; }
                                if m.config.threshold > vmax { m.config.threshold = vmax; }
                                m.state.excitation = m.state.excitation.clamp(vmin, vmax);
                            }
                        }
                    }
                }
            };
            let oninput_max = {
                let mut store = store.clone();
                move |e: FormEvent| {
                    if let Ok(mut vmax) = e.value().parse::<f64>() {
                        let mut s = store.write();
                        if let Some(n) = s.nodes.iter_mut().find(|n| n.id == nid) {
                            if let NeuronModelKind::IntegrateFire(m) = &mut n.model {
                                let vmin = m.config.minimum_voltage;
                                if vmax < vmin { vmax = vmin; }
                                m.config.maximum_voltage = vmax;
                                // clamp others
                                if m.config.reset_potential > vmax { m.config.reset_potential = vmax; }
                                if m.config.threshold > vmax { m.config.threshold = vmax; }
                                if m.config.threshold < m.config.reset_potential { m.config.threshold = m.config.reset_potential; }
                                m.state.excitation = m.state.excitation.clamp(vmin, vmax);
                            }
                        }
                    }
                }
            };

            section = rsx! {
                div {
                    {base}
                    // Editable fields bound to current snapshot values
                    NumberRow { label: "Reset (mV)", value: reset, oninput: oninput_reset }
                    NumberRow { label: "Threshold (mV)", value: threshold, oninput: oninput_threshold }
                    NumberRow { label: "Strength", value: strength, oninput: oninput_strength }
                    NumberRow { label: "Minimum Potential (mV)", value: min_v, oninput: oninput_min }
                    NumberRow { label: "Maximum Potential (mV)", value: max_v, oninput: oninput_max }
                }
            };
        } else {
            // Single-kind but not editable (either mixed selection or different model kind)
            section = rsx! { div { {base} } };
        }
    }

    rsx! { div { {section} } }
}

#[component]
fn ReadonlyStringProperty(label: String, value: String) -> Element {
    rsx! { div { class: "nn-prop-row",
        div { class: "nn-prop-label", "{label}" }
        input { class: "nn-prop-input", value: "{value}", readonly: true }
    }}
}

#[component]
fn NumberRow(label: String, value: f64, oninput: EventHandler<FormEvent>) -> Element {
    rsx! { div { class: "nn-prop-row",
        div { class: "nn-prop-label", "{label}" }
        input { class: "nn-prop-input", r#type: "number", value: format!("{}", value), oninput: move |e| oninput.call(e) }
    }}
}

#[component]
fn MutableStringProperty(label: String, value: Signal<String>) -> Element {
    rsx! { div {
        div { "{label}" }
        input { value: "{value}" }
    }}
}