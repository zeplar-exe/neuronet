use std::collections::HashMap;

use rustc_hash::FxHashMap;

use crate::{
    models::NeuronModelKind,
    simulation::network::{self, Current, Network, NeuronId, Time, Voltage},
};

pub const CONDUCTION_WHEEL_SIZE: usize = 5001;

#[derive(Clone, Default)]
pub struct Runstate {
    pub timestamp: i32,
    pub neuron_refractory: FxHashMap<NeuronId, Time>,
    // Bucket i holds conduction due at timestamps where t % CONDUCTION_WHEEL_SIZE == i
    pub synapse_wheel: Vec<Vec<(NeuronId, Current)>>,
    pub integrate_fire: FxHashMap<NeuronId, IntegrateFireState>,
    pub default_integrate_fire_state: IntegrateFireState,
    pub lif: FxHashMap<NeuronId, LifState>,
    pub default_lif_state: LifState,
    pub izhikevich: FxHashMap<NeuronId, IzhikevichState>,
    pub default_izhikevich_state: IzhikevichState,
}

#[derive(Clone)]
#[repr(C)]
pub struct IntegrateFireState {
    pub voltage: Voltage,
    pub reset_potential: Voltage,
    pub threshold: Voltage,
}

#[derive(Clone)]
#[repr(C)]
pub struct LifState {
    pub voltage: Voltage,
    pub resting_potential: Voltage,
    pub reset_potential: Voltage,
    pub threshold: Voltage,
    pub leak_constant: f64,
}

#[derive(Clone)]
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
            resting_potential: 0.0,
            reset_potential: 0.0,
            threshold: 20.0,
            leak_constant: 10.0,
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

pub fn get_integrate_fire_state(runstate: &mut Runstate, id: NeuronId) -> &mut IntegrateFireState {
    runstate.integrate_fire.get_mut(&id).unwrap()
}

pub fn get_lif_state(runstate: &mut Runstate, id: NeuronId) -> &mut LifState {
    runstate.lif.get_mut(&id).unwrap()
}

pub fn get_izhikevich_state(runstate: &mut Runstate, id: NeuronId) -> &mut IzhikevichState {
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

#[no_mangle]
pub extern "C" fn set_default_integrate_fire_state(runstate: *mut Runstate, state: IntegrateFireState) {
    unsafe {
        (*runstate).default_integrate_fire_state = state.clone();
    }
}

#[no_mangle]
pub extern "C" fn set_default_lif_state(runstate: *mut Runstate, state: LifState) {
    unsafe {
        (*runstate).default_lif_state = state.clone();
    }
}

#[no_mangle]
pub extern "C" fn set_default_izhikevich_state(runstate: *mut Runstate, state: IzhikevichState) {
    unsafe {
        (*runstate).default_izhikevich_state = state.clone();
    }
}

#[no_mangle]
pub extern "C" fn fill_defaults(network: *mut Network, runstate: *mut Runstate) {
    unsafe {
        for (neuron_id, neuron) in (*network).neurons.iter() {
            match neuron.model {
                NeuronModelKind::IntegrateFire => {
                    if !(*runstate).integrate_fire.contains_key(neuron_id) {
                        set_integrate_fire_state(
                            runstate,
                            *neuron_id,
                            (*runstate).default_integrate_fire_state.clone(),
                        );
                    }
                }
                NeuronModelKind::LIF => {
                    if !(*runstate).lif.contains_key(neuron_id) {
                        set_lif_state(runstate, *neuron_id, (*runstate).default_lif_state.clone());
                    }
                }
                NeuronModelKind::Izhikevich => {
                    if !(*runstate).izhikevich.contains_key(neuron_id) {
                        set_izhikevich_state(runstate, *neuron_id, (*runstate).default_izhikevich_state.clone());
                    }
                }
            }
        }
    }
}
