use crate::components::tooltip::{TooltipIndicator, TooltipKind};
use crate::models::NeuronModelKind;
use crate::simulation::id::NeuronId;
use crate::state::AppStore;
use crate::util::variant_eq;
use dioxus::prelude::*;
use std::collections::HashSet;

type NeuronPropertySnapshot = (NeuronId, f64, f64, f64, f64, f64, f64);

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
    let (selected_neurons_count, unique_model_count, model_name_string, editable_snapshot): (
        usize,
        usize,
        String,
        Option<NeuronPropertySnapshot>,
    ) = {
        let s = store.read();
        let selected_ids = s.selected.clone();
        let selected_neurons = s.get_selected_neurons();
        let selected_neurons_count = selected_neurons.len();

        let unique_model_names: HashSet<&'static str> = s
            .network
            .neurons
            .iter()
            .filter(|(nid, _)| selected_ids.contains(nid))
            .map(|(_, n)| n.model.name())
            .collect();
        let unique_model_count = unique_model_names.len();

        let mut model_name = "<mixed>".to_string();
        let mut snapshot: Option<NeuronPropertySnapshot> = None;

        if selected_neurons_count == 1 {
            let n = selected_neurons[0];
            model_name = n.model.name().to_string();
            if variant_eq(&n.model, &NeuronModelKind::integrate_fire()) {
                let idx = n.state_index as usize;
                let exec = &s.network.executor.integrate_fire;
                let voltage = *exec.voltage.get(idx).unwrap_or(&0.0);
                let reset = *exec.reset_potential.get(idx).unwrap_or(&-70.0);
                let threshold = *exec.threshold.get(idx).unwrap_or(&-55.0);
                let strength = *exec.strength.get(idx).unwrap_or(&30.0);
                let min_v = *exec.minimum_voltage.get(idx).unwrap_or(&-100.0);
                let max_v = *exec.maximum_voltage.get(idx).unwrap_or(&50.0);
                snapshot = Some((n.id, voltage, reset, threshold, strength, min_v, max_v));
            }
        } else if unique_model_count == 1 {
            model_name = unique_model_names.iter().next().copied().unwrap_or("").to_string();
        }
        (selected_neurons_count, unique_model_count, model_name, snapshot)
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
            ReadonlyStringProperty { label: "Model", value: "{model_name_string}" }
            ReadonlyStringProperty { label: "Tags", value: "" }
        };

        if let Some((nid, voltage_snapshot, reset, threshold, strength, min_v, max_v)) = editable_snapshot {
            // Handlers per field that write directly to the store with validation

            let oninput_voltage = {
                move |e: FormEvent| {
                    let mut s = store.write();
                    if let Ok(mut v) = e.value().parse::<f64>() {
                        if let Some((_, node)) = s.network.neurons.iter().find(|(i, _)| *i == nid) {
                            if variant_eq(&node.model, &NeuronModelKind::integrate_fire()) {
                                let idx = node.state_index as usize;
                                let min_v = s
                                    .network
                                    .executor
                                    .integrate_fire
                                    .minimum_voltage
                                    .get(idx)
                                    .copied()
                                    .unwrap_or(-1000.0);
                                let max_v = s
                                    .network
                                    .executor
                                    .integrate_fire
                                    .maximum_voltage
                                    .get(idx)
                                    .copied()
                                    .unwrap_or(1000.0);
                                v = v.clamp(min_v, max_v);
                                if let Some(slot) = s.network.executor.integrate_fire.voltage.get_mut(idx) {
                                    *slot = v;
                                }
                            }
                        }
                    }
                }
            };
            let oninput_reset = {
                move |e: FormEvent| {
                    let mut s = store.write();
                    if let Ok(mut v) = e.value().parse::<f64>() {
                        if let Some((_, node)) = s.network.neurons.iter().find(|(i, _)| *i == nid) {
                            if variant_eq(&node.model, &NeuronModelKind::integrate_fire()) {
                                let idx = node.state_index as usize;
                                let min_v = s
                                    .network
                                    .executor
                                    .integrate_fire
                                    .minimum_voltage
                                    .get(idx)
                                    .copied()
                                    .unwrap_or(-1000.0);
                                let mut max_v = s
                                    .network
                                    .executor
                                    .integrate_fire
                                    .maximum_voltage
                                    .get(idx)
                                    .copied()
                                    .unwrap_or(1000.0);
                                if min_v > max_v {
                                    max_v = min_v;
                                }
                                v = v.clamp(min_v, max_v);
                                if let Some(slot) = s.network.executor.integrate_fire.reset_potential.get_mut(idx) {
                                    *slot = v;
                                }
                                // ensure threshold >= reset
                                if let Some(th) = s.network.executor.integrate_fire.threshold.get_mut(idx) {
                                    if *th < v {
                                        *th = v;
                                    }
                                    if *th > max_v {
                                        *th = max_v;
                                    }
                                }
                                // clamp voltage
                                if let Some(vol) = s.network.executor.integrate_fire.voltage.get_mut(idx) {
                                    *vol = (*vol).clamp(min_v, max_v);
                                }
                            }
                        }
                    }
                }
            };
            let oninput_threshold = {
                move |e: FormEvent| {
                    let mut s = store.write();
                    if let Ok(mut v) = e.value().parse::<f64>() {
                        if let Some((_, node)) = s.network.neurons.iter().find(|(i, _)| *i == nid) {
                            if variant_eq(&node.model, &NeuronModelKind::integrate_fire()) {
                                let idx = node.state_index as usize;
                                let reset = s
                                    .network
                                    .executor
                                    .integrate_fire
                                    .reset_potential
                                    .get(idx)
                                    .copied()
                                    .unwrap_or(v);
                                let max_v = s
                                    .network
                                    .executor
                                    .integrate_fire
                                    .maximum_voltage
                                    .get(idx)
                                    .copied()
                                    .unwrap_or(v);
                                if reset > v {
                                    v = reset;
                                }
                                if v > max_v {
                                    v = max_v;
                                }
                                if let Some(th) = s.network.executor.integrate_fire.threshold.get_mut(idx) {
                                    *th = v;
                                }
                            }
                        }
                    }
                }
            };
            let oninput_strength = {
                move |e: FormEvent| {
                    let mut s = store.write();
                    if let Ok(v) = e.value().parse::<f64>() {
                        if let Some((_, node)) = s.network.neurons.iter().find(|(i, _)| *i == nid) {
                            if variant_eq(&node.model, &NeuronModelKind::integrate_fire()) {
                                let idx = node.state_index as usize;
                                if let Some(st) = s.network.executor.integrate_fire.strength.get_mut(idx) {
                                    *st = v;
                                }
                            }
                        }
                    }
                }
            };
            let oninput_min = {
                move |e: FormEvent| {
                    let mut s = store.write();
                    if let Ok(vmin) = e.value().parse::<f64>() {
                        if let Some((_, node)) = s.network.neurons.iter().find(|(i, _)| *i == nid) {
                            if variant_eq(&node.model, &NeuronModelKind::integrate_fire()) {
                                let idx = node.state_index as usize;
                                let mut vmax = s
                                    .network
                                    .executor
                                    .integrate_fire
                                    .maximum_voltage
                                    .get(idx)
                                    .copied()
                                    .unwrap_or(vmin);
                                if vmin > vmax {
                                    vmax = vmin;
                                }
                                if let Some(min_slot) = s.network.executor.integrate_fire.minimum_voltage.get_mut(idx) {
                                    *min_slot = vmin;
                                }
                                if let Some(max_slot) = s.network.executor.integrate_fire.maximum_voltage.get_mut(idx) {
                                    *max_slot = vmax;
                                }
                                // adjust reset/threshold/voltage to range
                                if let Some(reset_slot) = s.network.executor.integrate_fire.reset_potential.get_mut(idx)
                                {
                                    *reset_slot = (*reset_slot).clamp(vmin, vmax);
                                }
                                let reset_now = s
                                    .network
                                    .executor
                                    .integrate_fire
                                    .reset_potential
                                    .get(idx)
                                    .copied()
                                    .unwrap_or(vmin);
                                if let Some(th_slot) = s.network.executor.integrate_fire.threshold.get_mut(idx) {
                                    if *th_slot < reset_now {
                                        *th_slot = reset_now;
                                    }
                                    if *th_slot > vmax {
                                        *th_slot = vmax;
                                    }
                                }
                                if let Some(vol_slot) = s.network.executor.integrate_fire.voltage.get_mut(idx) {
                                    *vol_slot = (*vol_slot).clamp(vmin, vmax);
                                }
                            }
                        }
                    }
                }
            };
            let oninput_max = {
                move |e: FormEvent| {
                    let mut s = store.write();
                    if let Ok(mut vmax) = e.value().parse::<f64>() {
                        if let Some((_, node)) = s.network.neurons.iter().find(|(i, _)| *i == nid) {
                            if variant_eq(&node.model, &NeuronModelKind::integrate_fire()) {
                                let idx = node.state_index as usize;
                                let vmin = s
                                    .network
                                    .executor
                                    .integrate_fire
                                    .minimum_voltage
                                    .get(idx)
                                    .copied()
                                    .unwrap_or(vmax);
                                if vmax < vmin {
                                    vmax = vmin;
                                }
                                if let Some(max_slot) = s.network.executor.integrate_fire.maximum_voltage.get_mut(idx) {
                                    *max_slot = vmax;
                                }
                                // clamp others
                                if let Some(reset_slot) = s.network.executor.integrate_fire.reset_potential.get_mut(idx)
                                {
                                    if *reset_slot > vmax {
                                        *reset_slot = vmax;
                                    }
                                }
                                let reset_now = s
                                    .network
                                    .executor
                                    .integrate_fire
                                    .reset_potential
                                    .get(idx)
                                    .copied()
                                    .unwrap_or(vmin);
                                if let Some(th_slot) = s.network.executor.integrate_fire.threshold.get_mut(idx) {
                                    if *th_slot > vmax {
                                        *th_slot = vmax;
                                    }
                                    if *th_slot < reset_now {
                                        *th_slot = reset_now;
                                    }
                                }
                                if let Some(vol_slot) = s.network.executor.integrate_fire.voltage.get_mut(idx) {
                                    *vol_slot = (*vol_slot).clamp(vmin, vmax);
                                }
                            }
                        }
                    }
                }
            };

            section = rsx! {
                div {
                    {base}
                    // Editable fields bound to current snapshot values
                    NumberRow { label: "Membrane Potential (mV)", value: voltage_snapshot, oninput: oninput_voltage }
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
