use std::collections::HashMap;
use std::ffi::c_uchar;

use crate::models::NeuronModelKind;
use crate::simulation::id::{NeuronId, SynapseId};
use crate::util::ByteBuffer;

pub type Voltage = f64;
pub type Current = f64;
pub type Time = u32;

#[derive(Clone, PartialEq)]
pub struct Neuron {
    pub id: NeuronId,
    pub model: NeuronModelKind,
    pub refractory_period: Time,
    pub outgoing: Vec<SynapseId>,
}

#[derive(Clone, PartialEq)]
pub struct Synapse {
    pub strength: Current,
    pub source: NeuronId,
    pub target: NeuronId,
    pub conduction_time: Time,
}

#[derive(Clone, Default)]
pub struct Network {
    pub neurons: HashMap<NeuronId, Neuron>,
    pub synapses: HashMap<SynapseId, Synapse>,
}

impl Network {
    pub fn next_neuron_id(&self) -> NeuronId {
        self.neurons.len()
    }

    pub fn next_synapse_id(&self) -> SynapseId {
        self.synapses.len()
    }
}

#[no_mangle]
pub extern "C" fn create_network() -> *mut Network {
    return Box::into_raw(Box::new(Network::default()));
}

#[no_mangle]
pub extern "C" fn destroy_network(network: *mut Network) {
    unsafe {
        drop(Box::from_raw(network));
    }
}

#[no_mangle]
pub extern "C" fn serialize_network(network: *const Network) -> ByteBuffer {
    return ByteBuffer {
        data: std::ptr::null_mut(),
        len: 0,
    };
}

#[no_mangle]
pub extern "C" fn deserialize_network(data: *const c_uchar, length: usize) -> *mut Network {
    return Box::into_raw(Box::new(Network::default()));
}

#[no_mangle]
pub extern "C" fn add_synapse(
    network: *mut Network,
    source: NeuronId,
    target: NeuronId,
    strength: Current,
    conduction_time: Time,
) -> SynapseId {
    unsafe {
        let idx = (*network).next_synapse_id();
        (*network).synapses.insert(
            idx,
            Synapse {
                strength,
                source,
                target,
                conduction_time,
            },
        );
        (*network).neurons.get_mut(&source).unwrap().outgoing.push(idx);
        idx
    }
}

#[no_mangle]
pub extern "C" fn add_integrate_fire_neuron(network: *mut Network, refractory_period: Time) -> NeuronId {
    unsafe {
        let idx = (*network).next_neuron_id();
        (*network).neurons.insert(
            idx,
            Neuron {
                id: idx,
                model: NeuronModelKind::IntegrateFire,
                refractory_period,
                outgoing: Vec::new(),
            },
        );
        idx
    }
}

#[no_mangle]
pub extern "C" fn add_lif_neuron(network: *mut Network, refractory_period: Time) -> NeuronId {
    unsafe {
        let idx = (*network).next_neuron_id();
        (*network).neurons.insert(
            idx,
            Neuron {
                id: idx,
                model: NeuronModelKind::LIF,
                refractory_period,
                outgoing: Vec::new(),
            },
        );
        idx
    }
}

#[no_mangle]
pub extern "C" fn add_izhikevich_neuron(network: *mut Network, refractory_period: Time) -> NeuronId {
    unsafe {
        let idx = (*network).next_neuron_id();
        (*network).neurons.insert(
            idx,
            Neuron {
                id: idx,
                model: NeuronModelKind::Izhikevich,
                refractory_period,
                outgoing: Vec::new(),
            },
        );
        idx
    }
}

#[no_mangle]
pub extern "C" fn network_remove_neuron(network: *mut Network, id: NeuronId) {
    unsafe {
        (*network).neurons.remove(&id);
        (*network).synapses.retain(|_, e| e.target != id);
    }
}

#[no_mangle]
pub extern "C" fn remove_synapse(network: *mut Network, id: SynapseId) {
    unsafe {
        let synapse = (*network).synapses.remove(&id);
        if let Some(synapse) = synapse {
            (*network)
                .neurons
                .get_mut(&synapse.source)
                .unwrap()
                .outgoing
                .retain(|e| *e != id);
        }
    }
}

#[no_mangle]
pub extern "C" fn set_neuron_refractory_period(network: *mut Network, id: NeuronId, refractory_period: Time) {
    unsafe {
        (*network).neurons.get_mut(&id).unwrap().refractory_period = refractory_period;
    }
}

#[no_mangle]
pub extern "C" fn set_synapse_strength(network: *mut Network, id: SynapseId, strength: Voltage) {
    unsafe {
        (*network).synapses.get_mut(&id).unwrap().strength = strength;
    }
}

#[no_mangle]
pub extern "C" fn set_synapse_conduction_time(network: *mut Network, id: SynapseId, conduction_time: Time) {
    unsafe {
        (*network).synapses.get_mut(&id).unwrap().conduction_time = conduction_time;
    }
}
