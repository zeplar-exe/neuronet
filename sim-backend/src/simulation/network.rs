use std::ffi::c_uchar;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

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
    ffi_catch_ptr!(Box::into_raw(Box::new(Network::default())))
}

#[no_mangle]
pub extern "C" fn destroy_network(network: *mut Network) {
    ffi_catch_void!(unsafe { drop(Box::from_raw(network)) })
}

#[no_mangle]
pub extern "C" fn serialize_network(network: *const Network, json: bool) -> ByteBuffer {
    ffi_catch_struct!(
        ByteBuffer {
            data: std::ptr::null_mut(),
            len: 0
        },
        unsafe {
            let bytes = if json {
                serde_json::to_vec(&*network).unwrap()
            } else {
                bincode::serialize(&*network).unwrap()
            };
            let mut bytes = std::mem::ManuallyDrop::new(bytes);
            ByteBuffer {
                data: bytes.as_mut_ptr(),
                len: bytes.len(),
            }
        }
    )
}

#[no_mangle]
pub extern "C" fn deserialize_network(data: *const c_uchar, length: usize, json: bool) -> *mut Network {
    ffi_catch_ptr!(unsafe {
        let slice = std::slice::from_raw_parts(data, length);
        let network: Network = if json {
            serde_json::from_slice(slice).unwrap()
        } else {
            bincode::deserialize(slice).unwrap()
        };
        Box::into_raw(Box::new(network))
    })
}

#[no_mangle]
pub extern "C" fn get_neurons(network: *const Network) -> NeuronBuffer {
    ffi_catch_struct!(
        NeuronBuffer {
            data: std::ptr::null(),
            len: 0
        },
        unsafe {
            let neurons = (*network).neurons.keys().copied().collect::<Vec<_>>();
            let leaked = std::mem::ManuallyDrop::new(neurons);
            NeuronBuffer {
                data: leaked.as_ptr(),
                len: (*network).neurons.len(),
            }
        }
    )
}

#[no_mangle]
pub extern "C" fn get_synapses(network: *const Network) -> SynapseBuffer {
    ffi_catch_struct!(
        SynapseBuffer {
            data: std::ptr::null(),
            len: 0
        },
        unsafe {
            let synapses = (*network).synapses.keys().copied().collect::<Vec<_>>();
            let leaked = std::mem::ManuallyDrop::new(synapses);
            SynapseBuffer {
                data: leaked.as_ptr(),
                len: (*network).synapses.len(),
            }
        }
    )
}

#[no_mangle]
pub extern "C" fn add_synapse(network: *mut Network, source: NeuronId, target: NeuronId) -> SynapseId {
    ffi_catch_num!(unsafe {
        let idx = (*network).next_synapse_id();
        (*network).synapses.insert(idx, Synapse { source, target });
        (*network).neurons.get_mut(&source).unwrap().outgoing.push(idx);
        idx
    })
}

#[no_mangle]
pub extern "C" fn add_integrate_fire_neuron(network: *mut Network) -> NeuronId {
    ffi_catch_num!(unsafe {
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
    })
}

#[no_mangle]
pub extern "C" fn add_lif_neuron(network: *mut Network) -> NeuronId {
    ffi_catch_num!(unsafe {
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
    })
}

#[no_mangle]
pub extern "C" fn add_izhikevich_neuron(network: *mut Network) -> NeuronId {
    ffi_catch_num!(unsafe {
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
    })
}

#[no_mangle]
pub extern "C" fn network_remove_neuron(network: *mut Network, id: NeuronId) {
    ffi_catch_void!(unsafe {
        if let Some(_) = (*network).neurons.remove(&id) {
            (*network).synapses.retain(|_, e| {
                if e.source == id {
                    return false;
                } else if e.target == id {
                    if let Some(source) = (*network).neurons.get_mut(&e.source) {
                        source.outgoing.retain(|e| *e != id);
                    }
                    return false;
                }
                return true;
            });
        }
    })
}

#[no_mangle]
pub extern "C" fn network_remove_synapse(network: *mut Network, id: SynapseId) {
    ffi_catch_void!(unsafe {
        if let Some(synapse) = (*network).synapses.remove(&id) {
            if let Some(source) = (*network).neurons.get_mut(&synapse.source) {
                source.outgoing.retain(|e| *e != id);
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn get_neuron_model(network: *const Network, id: NeuronId) -> NeuronModelKind {
    ffi_catch_struct!(NeuronModelKind::IntegrateFire, unsafe {
        (*network).neurons.get(&id).unwrap().model.clone()
    })
}

#[no_mangle]
pub extern "C" fn network_has_neuron(network: *mut Network, id: NeuronId) -> bool {
    ffi_catch_num!(unsafe { (*network).neurons.contains_key(&id) })
}

#[no_mangle]
pub extern "C" fn network_has_synapse(network: *mut Network, id: SynapseId) -> bool {
    ffi_catch_num!(unsafe { (*network).synapses.contains_key(&id) })
}
