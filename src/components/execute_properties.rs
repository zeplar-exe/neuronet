use dioxus::prelude::*;
use std::collections::HashSet;
use crate::components::tooltip::{TooltipIndicator, TooltipKind};
use crate::models::{NeuronModelKind};
use crate::util::variant_eq;
use crate::state::AppStore;

#[component]
pub fn ExecuteProperties() -> Element {
    let store = use_context::<Signal<AppStore>>();

    // Snapshot info for rendering without holding a write lock
    let (selected_count, unique_model_count, id_string, model_name_string, editable): (usize, usize, String, String, Option<(i64, f64, f64, f64)>) = {
        let s = store.read();
        let selected_ids = s.selected.clone();
        let selected_neurons_ids: Vec<i64> = s.network.neurons.iter().filter(|n| selected_ids.contains(&n.id)).map(|n| n.id).collect();
        let selected_count = selected_neurons_ids.len();

        let unique_model_names: HashSet<&'static str> = s.network.neurons.iter()
            .filter(|n| selected_ids.contains(&n.id))
            .map(|n| n.model.name())
            .collect();
        let unique_model_count = unique_model_names.len();

        let mut id = "<mixed>".to_string();
        let mut model_name = "<mixed>".to_string();
        let mut editable: Option<(i64, f64, f64, f64)> = None; // (id, voltage, min_v, max_v)

        if selected_count == 1 {
            let nid = selected_neurons_ids[0];
            id = nid.to_string();
            if let Some(node) = s.network.neurons.iter().find(|n| n.id == nid) {
                model_name = node.model.name().to_string();
                if variant_eq(&node.model, &NeuronModelKind::integrate_fire()) {
                    let idx = node.state_index as usize;
                    let exec = &s.network.executor.integrate_fire;
                    let voltage = *exec.voltage.get(idx).unwrap_or(&0.0);
                    let min_v = *exec.minimum_voltage.get(idx).unwrap_or(&-1000.0);
                    let max_v = *exec.maximum_voltage.get(idx).unwrap_or(&1000.0);
                    editable = Some((nid, voltage, min_v, max_v));
                }
            }
        } else if unique_model_count == 1 {
            model_name = unique_model_names.iter().next().copied().unwrap_or("").to_string();
        }

        (selected_count, unique_model_count, id, model_name, editable)
    };

    // Prepare optional editor
    let editor: Option<(f64, EventHandler<FormEvent>)> = if let Some((nid, voltage, min_v, max_v)) = editable {
        let mut store2 = store.clone();
        let handler = move |e: FormEvent| {
            if let Ok(v) = e.value().parse::<f64>() {
                let clamped = v.clamp(min_v, max_v);
                let mut s = store2.write();
                if let Some(node) = s.network.neurons.iter().find(|n| n.id == nid) {
                    let idx = node.state_index as usize;
                    if let Some(slot) = s.network.executor.integrate_fire.voltage.get_mut(idx) {
                        *slot = clamped;
                    }
                }
            }
        };
        Some((voltage, EventHandler::new(handler)))
    } else { None };

    rsx! {
        div { class: "nn-props", tabindex: 0,
            div { class: "flex items-center justify-between mb-2",
                h3 { "Run State Properties" }
            }
            div { class: "mb-2",
                div { "{selected_count} neurons selected" }
                div { "{unique_model_count} model kinds selected" }
            }
            if unique_model_count > 1 {
                TooltipIndicator { kind: TooltipKind::Warning, text: "Multiple model kinds selected. State editing disabled." }
            }
            if unique_model_count != 0 {
                ReadonlyStringRow { label: "ID", value: id_string.clone() }
                ReadonlyStringRow { label: "Model", value: model_name_string.clone() }

                if let Some((voltage, oninput_voltage)) = editor {
                    NumberRow { label: "Membrane Potential (mV)", value: voltage, oninput: oninput_voltage }
                }
            }
        }
    }
}

#[component]
fn ReadonlyStringRow(label: String, value: String) -> Element {
    rsx! { div { class: "nn-prop-row",
        div { class: "nn-prop-label", "{label}" }
        input { class: "nn-prop-input", value: "{value}", readonly: true }
    } }
}

#[component]
fn NumberRow(label: String, value: f64, oninput: EventHandler<FormEvent>) -> Element {
    rsx! { div { class: "nn-prop-row",
        div { class: "nn-prop-label", "{label}" }
        input { class: "nn-prop-input", r#type: "number", value: format!("{}", value), oninput: move |e| oninput.call(e) }
    } }
}
