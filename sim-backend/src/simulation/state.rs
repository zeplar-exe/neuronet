use std::ffi::c_uchar;

use rustc_hash::FxHashMap;
use serde::{Serialize, Deserialize};

use crate::simulation::network::{Current, NeuronId, SynapseId, Time, Voltage};
use crate::util::ByteBuffer;

pub const CONDUCTION_WHEEL_SIZE: usize = 5001;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Runstate {
    pub timestamp: i32,
    pub synapse_conduction_time: FxHashMap<SynapseId, Time>,
    pub synapse_strength: FxHashMap<SynapseId, Current>,
    pub neuron_refractory_period: FxHashMap<NeuronId, Time>,
    pub neuron_refractory: FxHashMap<NeuronId, Time>,
    // Bucket i holds conduction due at timestamps where t % CONDUCTION_WHEEL_SIZE == i
    pub synapse_wheel: Vec<Vec<(NeuronId, Current)>>,
    pub integrate_fire: FxHashMap<NeuronId, IntegrateFireState>,
    pub lif: FxHashMap<NeuronId, LifState>,
    pub izhikevich: FxHashMap<NeuronId, IzhikevichState>,
}

#[derive(Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct IntegrateFireState {
    pub voltage: Voltage,
    pub reset_potential: Voltage,
    pub threshold: Voltage,
}

#[derive(Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct LifState {
    pub voltage: Voltage,
    pub reset_potential: Voltage,
    pub threshold: Voltage,
    pub leak_rate: f64,
    pub input_gain: f64,
}

#[derive(Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct IzhikevichState {
    pub voltage: Voltage,
    pub reset_potential: Voltage,
    pub threshold: Voltage,
    pub a_var: f64,
    pub b_var: f64,
    pub recovery_var: f64,
    pub d_var: f64,
}

impl Default for IntegrateFireState {
    fn default() -> Self {
        Self {
            voltage: 0 as Voltage,
            reset_potential: 0 as Voltage,
            threshold: 30 as Voltage,
        }
    }
}

impl Default for LifState {
    fn default() -> Self {
        Self {
            voltage: 0.0,
            reset_potential: 0.0,
            threshold: 20.0,
            leak_rate: 0.01,
            input_gain: 1.0,
        }
    }
}

impl Default for IzhikevichState {
    fn default() -> Self {
        Self {
            voltage: -65 as Voltage,
            reset_potential: -70 as Voltage,
            threshold: -55 as Voltage,
            a_var: 0.02,
            b_var: 0.2,
            recovery_var: 0.0,
            d_var: 8.0,
        }
    }
}

#[no_mangle]
pub extern "C" fn serialize_runstate(runstate: *const Runstate, json: bool) -> ByteBuffer {
    unsafe {
        let bytes = if json {
            serde_json::to_vec(&*runstate).unwrap()
        } else {
            bincode::serialize(&*runstate).unwrap()
        };
        let mut bytes = std::mem::ManuallyDrop::new(bytes);
        ByteBuffer { data: bytes.as_mut_ptr(), len: bytes.len() }
    }
}

#[no_mangle]
pub extern "C" fn deserialize_runstate(data: *const c_uchar, length: usize, json: bool) -> *mut Runstate {
    unsafe {
        let slice = std::slice::from_raw_parts(data, length);
        let runstate: Runstate = if json {
            serde_json::from_slice(slice).unwrap()
        } else {
            bincode::deserialize(slice).unwrap()
        };
        Box::into_raw(Box::new(runstate))
    }
}

#[no_mangle]
pub extern "C" fn create_runstate() -> *mut Runstate {
    let mut runstate = Runstate::default();
    runstate.synapse_wheel = vec![Vec::new(); CONDUCTION_WHEEL_SIZE];
    Box::into_raw(Box::new(runstate))
}

#[no_mangle]
pub extern "C" fn clone_runstate(runstate: *mut Runstate) -> *mut Runstate {
    unsafe {
        let runstate = (*runstate).clone();
        Box::into_raw(Box::new(runstate))
    }
}

#[no_mangle]
pub extern "C" fn destroy_runstate(runstate: *mut Runstate) {
    unsafe {
        drop(Box::from_raw(runstate));
    }
}

#[no_mangle]
pub extern "C" fn set_neuron_refractory_period(runstate: *mut Runstate, id: NeuronId, refractory_period: Time) {
    unsafe {
        (*runstate).neuron_refractory_period.insert(id, refractory_period);
    }
}

#[no_mangle]
pub extern "C" fn set_synapse_strength(runstate: *mut Runstate, id: SynapseId, strength: Voltage) {
    unsafe {
        (*runstate).synapse_strength.insert(id, strength);
    }
}

#[no_mangle]
pub extern "C" fn set_synapse_conduction_time(runstate: *mut Runstate, id: SynapseId, conduction_time: Time) {
    unsafe {
        (*runstate).synapse_conduction_time.insert(id, conduction_time);
    }
}

#[no_mangle]
pub extern "C" fn runstate_remove_neuron(runstate: *mut Runstate, id: NeuronId) {
    unsafe {
        (*runstate).integrate_fire.remove(&id);
        (*runstate).lif.remove(&id);
        (*runstate).izhikevich.remove(&id);
        (*runstate).neuron_refractory_period.remove(&id);
        (*runstate).neuron_refractory.remove(&id);
    }
}

#[no_mangle]
pub extern "C" fn get_integrate_fire_state(runstate: &mut Runstate, id: NeuronId) -> &mut IntegrateFireState {
    runstate.integrate_fire.get_mut(&id).unwrap()
}

#[no_mangle]
pub extern "C" fn get_lif_state(runstate: &mut Runstate, id: NeuronId) -> &mut LifState {
    runstate.lif.get_mut(&id).unwrap()
}

#[no_mangle]
pub extern "C" fn get_izhikevich_state(runstate: &mut Runstate, id: NeuronId) -> &mut IzhikevichState {
    runstate.izhikevich.get_mut(&id).unwrap()
}

#[no_mangle]
pub extern "C" fn set_integrate_fire_state(runstate: *mut Runstate, neuron_id: NeuronId, state: IntegrateFireState) {
    unsafe {
        (*runstate).integrate_fire.insert(neuron_id, state.clone());
    }
}

#[no_mangle]
pub extern "C" fn set_lif_state(runstate: *mut Runstate, neuron_id: NeuronId, state: LifState) {
    unsafe {
        (*runstate).lif.insert(neuron_id, state);
    }
}

#[no_mangle]
pub extern "C" fn set_izhikevich_state(runstate: *mut Runstate, neuron_id: NeuronId, state: IzhikevichState) {
    unsafe {
        (*runstate).izhikevich.insert(neuron_id, state);
    }
}
