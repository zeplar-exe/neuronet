use std::collections::HashMap;

use crate::models::NeuronModelKind;
use crate::simulation::id::{NeuronId, SynapseId};
use crate::util::variant_eq;

pub type Voltage = i16;

#[derive(Clone, PartialEq)]
pub struct Neuron {
    pub id: NeuronId,
    pub model: NeuronModelKind,
    pub outgoing: Vec<SynapseId>,
    pub conduction_velocity: f32,
}

#[derive(Clone, PartialEq)]
pub struct Synapse {
    pub weight: i16,
    pub source: NeuronId,
    pub target: NeuronId,
    pub conduction_velocity: f32,
}

#[derive(Clone, Default)]
pub struct Network {
    pub neurons: HashMap<NeuronId, Neuron>,
    pub synapses: HashMap<SynapseId, Synapse>,
    pub runstate: Runstate,
}

impl Network {
    pub fn next_neuron_id(&self) -> NeuronId {
        self.neurons.len()
    }

    pub fn next_synapse_id(&self) -> SynapseId {
        self.synapses.len()
    }
}

#[derive(Clone, Default)]
pub struct Runstate {
    pub neuron_propagation: HashMap<NeuronId, Vec<f32>>,
    pub synapse_propagation: HashMap<NeuronId, Vec<f32>>,
    pub integrate_fire: HashMap<NeuronId, IntegrateFireState>,
}

#[derive(Clone)]
pub struct IntegrateFireState {
    pub voltage: Voltage,
    pub reset_potential: Voltage,
    pub strength: Voltage,
    pub threshold: Voltage,
    pub minimum_voltage: Voltage,
    pub maximum_voltage: Voltage,
}

impl Default for IntegrateFireState {
    fn default() -> Self {
        Self {
            voltage: 0 as Voltage,
            strength: 30 as Voltage,
            reset_potential: -70 as Voltage,
            threshold: -55 as Voltage,
            minimum_voltage: -100 as Voltage,
            maximum_voltage: 50 as Voltage,
        }
    }
}

pub fn serialize_network(network: *const Network) -> String {
    return "".to_string();
}

pub fn deserialize_network(data: &str) -> Network {
    return Network::default();
}

pub fn get_voltage(network: *const Network, id: NeuronId) -> Voltage {
    unsafe {
        let neuron = (*network).neurons.get(&id).unwrap();
        let if_state = (*network).runstate.integrate_fire.get(&id).unwrap();
        match &neuron.model {
            NeuronModelKind::IntegrateFire(_) => if_state.voltage,
            NeuronModelKind::LIF(_) => 0,
            NeuronModelKind::Izhikevich(_) => 0,
        }
    }
}

pub fn set_voltage(network: *mut Network, id: NeuronId, voltage: Voltage) {
    unsafe {
        let neuron = (*network).neurons.get(&id).unwrap();
        let if_state = (*network).runstate.integrate_fire.get_mut(&id).unwrap();
        match &neuron.model {
            NeuronModelKind::IntegrateFire(_) => if_state.voltage = voltage,
            NeuronModelKind::LIF(_) => {}
            NeuronModelKind::Izhikevich(_) => {}
        }
    }
}

pub fn add_synapse(
    network: *mut Network,
    source: NeuronId,
    target: NeuronId,
    strength: Voltage,
    propogation_speed: f32,
) -> SynapseId {
    unsafe {
        let idx = (*network).next_synapse_id();
        (*network).synapses.insert(
            idx,
            Synapse {
                weight: strength,
                source: source,
                target: target,
                conduction_velocity: propogation_speed,
            },
        );
        (*network).neurons.get_mut(&source).unwrap().outgoing.push(idx);
        idx
    }
}

pub fn add_integrate_fire_neuron(network: *mut Network) -> NeuronId {
    unsafe {
        let idx = (*network).next_neuron_id();
        (*network).neurons.insert(
            idx,
            Neuron {
                id: idx,
                model: NeuronModelKind::integrate_fire(),
                outgoing: Vec::new(),
                conduction_velocity: 1.0,
            },
        );
        idx
    }
}

pub fn remove_neuron(network: *mut Network, id: NeuronId) {
    unsafe {
        let model = (*network).neurons.get(&id).unwrap().model.clone();

        if variant_eq(&model, &NeuronModelKind::integrate_fire()) {
            (*network).runstate.integrate_fire.remove(&id);
        }

        (*network).neurons.remove(&id);
        (*network).synapses.retain(|_, e| e.target != id);
    }
}
