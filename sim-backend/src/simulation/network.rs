use std::ffi::c_uchar;

use rustc_hash::FxHashMap;
use serde::{Serialize, Deserialize};

use crate::models::NeuronModelKind;
use crate::util::{ByteBuffer, NeuronBuffer, SynapseBuffer};

pub type NeuronId = u32;
pub type SynapseId = u32;

pub type Voltage = f64;
pub type Current = f64;
pub type Time = u32;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Neuron {
    pub id: NeuronId,
    pub model: NeuronModelKind,
    pub outgoing: Vec<SynapseId>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Synapse {
    pub source: NeuronId,
    pub target: NeuronId,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Network {
    pub neurons: FxHashMap<NeuronId, Neuron>,
    pub synapses: FxHashMap<SynapseId, Synapse>,
    next_neuron_id: NeuronId,
    next_synapse_id: SynapseId,
}

impl Network {
    pub fn next_neuron_id(&mut self) -> NeuronId {
        self.next_neuron_id += 1;
        self.next_neuron_id - 1
    }

    pub fn next_synapse_id(&mut self) -> SynapseId {
        self.next_synapse_id += 1;
        self.next_synapse_id - 1
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
pub extern "C" fn serialize_network(network: *const Network, json: bool) -> ByteBuffer {
    unsafe {
        let bytes = if json {
            serde_json::to_vec(&*network).unwrap()
        } else {
            bincode::serialize(&*network).unwrap()
        };
        let mut bytes = std::mem::ManuallyDrop::new(bytes);
        ByteBuffer { data: bytes.as_mut_ptr(), len: bytes.len() }
    }
}

#[no_mangle]
pub extern "C" fn deserialize_network(data: *const c_uchar, length: usize, json: bool) -> *mut Network {
    unsafe {
        let slice = std::slice::from_raw_parts(data, length);
        let network: Network = if json {
            serde_json::from_slice(slice).unwrap()
        } else {
            bincode::deserialize(slice).unwrap()
        };
        Box::into_raw(Box::new(network))
    }
}

#[no_mangle]
pub extern "C" fn get_neurons(network: *const Network) -> NeuronBuffer {
    unsafe {
        let neurons = (*network).neurons.keys().copied().collect::<Vec<_>>();
        let leaked = std::mem::ManuallyDrop::new(neurons);
        return NeuronBuffer {
            data: leaked.as_ptr(),
            len: (*network).neurons.len(),
        };
    }
}

#[no_mangle]
pub extern "C" fn get_synapses(network: *const Network) -> SynapseBuffer {
    unsafe {
        let synapses = (*network).synapses.keys().copied().collect::<Vec<_>>();
        let leaked = std::mem::ManuallyDrop::new(synapses);
        return SynapseBuffer {
            data: leaked.as_ptr(),
            len: (*network).synapses.len(),
        };
    }
}

#[no_mangle]
pub extern "C" fn add_synapse(network: *mut Network, source: NeuronId, target: NeuronId) -> SynapseId {
    unsafe {
        let idx = (*network).next_synapse_id();
        (*network).synapses.insert(idx, Synapse { source, target });
        (*network).neurons.get_mut(&source).unwrap().outgoing.push(idx);
        idx
    }
}

#[no_mangle]
pub extern "C" fn add_integrate_fire_neuron(network: *mut Network) -> NeuronId {
    unsafe {
        let idx = (*network).next_neuron_id();
        (*network).neurons.insert(
            idx,
            Neuron {
                id: idx,
                model: NeuronModelKind::IntegrateFire,
                outgoing: Vec::new(),
            },
        );
        idx
    }
}

#[no_mangle]
pub extern "C" fn add_lif_neuron(network: *mut Network) -> NeuronId {
    unsafe {
        let idx = (*network).next_neuron_id();
        (*network).neurons.insert(
            idx,
            Neuron {
                id: idx,
                model: NeuronModelKind::LIF,
                outgoing: Vec::new(),
            },
        );
        idx
    }
}

#[no_mangle]
pub extern "C" fn add_izhikevich_neuron(network: *mut Network) -> NeuronId {
    unsafe {
        let idx = (*network).next_neuron_id();
        (*network).neurons.insert(
            idx,
            Neuron {
                id: idx,
                model: NeuronModelKind::Izhikevich,
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
pub extern "C" fn network_remove_synapse(network: *mut Network, id: SynapseId) {
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
pub extern "C" fn get_neuron_model(network: *const Network, id: NeuronId) -> NeuronModelKind {
    unsafe { (*network).neurons.get(&id).unwrap().model.clone() }
}

#[no_mangle]
pub extern "C" fn network_has_neuron(network: *mut Network, id: NeuronId) -> bool {
    unsafe { (*network).neurons.contains_key(&id) }
}

#[no_mangle]
pub extern "C" fn network_has_synapse(network: *mut Network, id: SynapseId) -> bool {
    unsafe { (*network).synapses.contains_key(&id) }
}
