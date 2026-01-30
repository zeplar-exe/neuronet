use dioxus::prelude::*;
use itertools::Itertools;
use crate::components::tooltip::{TooltipIndicator, TooltipKind};
use crate::models::NeuronModelKind;
use crate::state::{AppStore, Node};

#[component]
pub fn Properties() -> Element {
    let mut store = use_context::<Signal<AppStore>>();
    // Track expanded groups
    let expanded: Signal<std::collections::HashSet<String>> = use_signal(Default::default);

    rsx! {
        div { class: "text-[#f6f6f6] relative", tabindex: 0,
            div { class: "flex items-center justify-between mb-2",
                h3 { "Properties" }
            }

        }
    }
}

#[component]
fn RenderNeuronProperties(store: Signal<AppStore>) -> Element {
    let selected_ids = store.read().selected.clone();
    let selected_neurons = store.read().get_selected_neurons();
    let mut unique_models = selected_neurons.iter().map(|n| &n.model).unique().collect();

    let selected_neurons_count = selected_ids.len();
    let unique_model_count = unique_models.len();

    let mut id = "<mixed>".to_string();
    let tags = Vec::default();

    if selected_neurons_count == 1 {
        id = selected_neurons[0].id.to_string();
    }

    rsx! { div {
        div { "{selected_neurons_count} neurons selected" }
        div { "{unique_model_count} model kinds selected" }
        if unique_models > 1 {
            TooltipIndicator {
                kind: TooltipKind::Warning,
                text: "Multiple model kinds selected. Cannot display unified property view." }
        } else if unique_models == 1 {
            ReadonlyStringProperty { label: "ID", value: "{id}" }
            ReadonlyStringProperty { label: "Model", value: "" }
            ReadonlyStringProperty { label: "Tags", value: "" }
            if selected_neurons_count == 1 {
                if let NeuronModelKind::IntegrateFire(ifmodel) = *selected_neurons[0].model.read() {
                    MutableNumberProperty { label: "Potential", value: ifmodel.excitation }
                    MutableNumberProperty { label: "Reset", value: ifmodel.reset_potential }
                    MutableNumberProperty { label: "Threshold", value: ifmodel.threshold }
                    MutableNumberProperty { label: "Strength", value: ifmodel.strength }
                    MutableNumberProperty { label: "Minimum Potential", value: ifmodel.minimum_voltage }
                    MutableNumberProperty { label: "Maximum Potential", value: ifmodel.maximum_voltage }
                }
            } else {
                ReadonlyStringProperty { label: "Potential", value: "<mixed>" }
                ReadonlyStringProperty { label: "Reset", value: "<mixed>" }
                ReadonlyStringProperty { label: "Threshold", value: "<mixed>" }
                ReadonlyStringProperty { label: "Strength", value: "<mixed>" }
                ReadonlyStringProperty { label: "Minimum Potential", value: "<mixed>" }
                ReadonlyStringProperty { label: "Maximum Potential", value: "<mixed>" }
            }
        }
    } }
}

#[component]
fn ReadonlyStringProperty(label: String, value: String) -> Element {
    rsx! { div {
        div { "{label}" }
        input { value: "{value}", readonly: true }
    }}
}

#[component]
fn MutableNumberProperty(label: String, value: Signal<f64>) -> Element {
    rsx! { div {
        div { "{label}" }
        input { value: "{value}" }
    }}
}

#[component]
fn MutableStringProperty(label: String, value: Signal<String>) -> Element {
    rsx! { div {
        div { "{label}" }
        input { value: "{value}" }
    }}
}