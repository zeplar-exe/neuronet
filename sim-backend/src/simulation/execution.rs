use std::collections::HashMap;

use rustc_hash::FxHashMap;

use crate::{
    models::NeuronModelKind,
    simulation::{
        events::{EventContainer, SpikeEvent},
        network::{Current, Network, NeuronId, SynapseId, Voltage},
        state::{get_integrate_fire_state, get_izhikevich_state, get_lif_state, Runstate, CONDUCTION_WHEEL_SIZE},
    },
    util::variant_eq,
};

#[derive(Clone, Default)]
pub struct StimulusContainer {
    pub current_stimuli: HashMap<NeuronId, Current>,
}

#[no_mangle]
pub extern "C" fn create_stimulus_container() -> *mut StimulusContainer {
    return Box::into_raw(Box::new(StimulusContainer::default()));
}

#[no_mangle]
pub extern "C" fn destroy_stimulus_container(container: *mut StimulusContainer) {
    unsafe {
        drop(Box::from_raw(container));
    }
}

#[no_mangle]
pub extern "C" fn get_voltage(network: *const Network, runstate: *mut Runstate, id: NeuronId) -> Voltage {
    unsafe {
        let neuron = (*network).neurons.get(&id).unwrap();

        match &neuron.model {
            NeuronModelKind::IntegrateFire => get_integrate_fire_state(&mut *runstate, id).voltage,
            NeuronModelKind::LIF => get_lif_state(&mut *runstate, id).voltage,
            NeuronModelKind::Izhikevich => get_izhikevich_state(&mut *runstate, id).voltage,
        }
    }
}

#[no_mangle]
pub extern "C" fn get_threshold(network: *const Network, runstate: *mut Runstate, id: NeuronId) -> Voltage {
    unsafe {
        let neuron = (*network).neurons.get(&id).unwrap();

        match &neuron.model {
            NeuronModelKind::IntegrateFire => get_integrate_fire_state(&mut *runstate, id).threshold,
            NeuronModelKind::LIF => get_lif_state(&mut *runstate, id).threshold,
            NeuronModelKind::Izhikevich => get_izhikevich_state(&mut *runstate, id).threshold,
        }
    }
}

#[no_mangle]
pub extern "C" fn set_voltage(network: *const Network, runstate: *mut Runstate, id: NeuronId, voltage: Voltage) {
    unsafe {
        let neuron = (*network).neurons.get(&id).unwrap();
        let if_state = (*runstate).integrate_fire.get_mut(&id).unwrap();
        match &neuron.model {
            NeuronModelKind::IntegrateFire => if_state.voltage = voltage,
            NeuronModelKind::LIF => get_lif_state(&mut *runstate, id).voltage = voltage,
            NeuronModelKind::Izhikevich => get_izhikevich_state(&mut *runstate, id).voltage = voltage,
        }
    }
}

#[no_mangle]
pub extern "C" fn runstate_remove_neuron(network: *const Network, runstate: *mut Runstate, id: NeuronId) {
    unsafe {
        let model = (*network).neurons.get(&id).unwrap().model.clone();

        if variant_eq(&model, &NeuronModelKind::IntegrateFire) {
            (*runstate).integrate_fire.remove(&id);
        } else if variant_eq(&model, &NeuronModelKind::LIF) {
            (*runstate).lif.remove(&id);
        } else if variant_eq(&model, &NeuronModelKind::Izhikevich) {
            (*runstate).izhikevich.remove(&id);
        }

        (*runstate).neuron_refractory.remove(&id);
    }
}

#[no_mangle]
pub extern "C" fn set_current_stimulus(stimuli: *mut StimulusContainer, neuron_id: NeuronId, current: Current) {
    unsafe {
        (*stimuli).current_stimuli.insert(neuron_id, current);
    }
}

#[no_mangle]
pub extern "C" fn step(
    network: *mut Network,
    runstate: *mut Runstate,
    stimuli: *const StimulusContainer,
    events: *mut EventContainer,
) -> *mut Runstate {
    // step in 1e-4 s

    unsafe {
        for (neuron_id, neuron) in (*network).neurons.iter() {
            let mut fired = false;

            match neuron.model {
                NeuronModelKind::IntegrateFire => {
                    let state = get_integrate_fire_state(&mut *runstate, *neuron_id);
                    if state.voltage >= state.threshold {
                        if !events.is_null() {
                            (*events).spikes.push(SpikeEvent {
                                timestamp: (*runstate).timestamp,
                                neuron_id: *neuron_id,
                                voltage: state.voltage,
                            });
                        }
                        state.voltage = state.reset_potential;

                        fired = true;
                    }
                }
                NeuronModelKind::LIF => {
                    let state = get_lif_state(&mut *runstate, *neuron_id);
                    if state.voltage >= state.threshold {
                        if !events.is_null() {
                            (*events).spikes.push(SpikeEvent {
                                timestamp: (*runstate).timestamp,
                                neuron_id: *neuron_id,
                                voltage: state.voltage,
                            });
                        }
                        state.voltage = state.reset_potential;

                        fired = true;
                    }
                }
                NeuronModelKind::Izhikevich => {
                    let state = get_izhikevich_state(&mut *runstate, *neuron_id);
                    if state.voltage >= state.threshold {
                        if !events.is_null() {
                            (*events).spikes.push(SpikeEvent {
                                timestamp: (*runstate).timestamp,
                                neuron_id: *neuron_id,
                                voltage: state.voltage,
                            });
                        }
                        state.voltage = state.reset_potential;
                        state.recovery_var = state.recovery_var + state.d_var;

                        fired = true;
                        // hmm.. well, we need to do the current additive stuff directly in here
                        // instead of globally.. so the question is how did izhikevich and co. do
                        // their simulation? as in, what determines the amount of current that
                        // comes from an incoming synapse (this can be found in stdp.pdf)
                        // apparently dI(syn)/dt = -I(syn)/tau for exponential decay... how get tau? how store I(syn)
                    }
                }
            }

            if fired {
                let t = (*runstate).timestamp as usize;
                let wheel = &mut (*runstate).synapse_wheel;
                for synapse_id in (*neuron).outgoing.iter() {
                    let synapse = (*network).synapses.get(synapse_id).unwrap();
                    let delay = (synapse.conduction_time as usize).saturating_sub(1);
                    debug_assert!(delay < CONDUCTION_WHEEL_SIZE, "conduction delay exceeds wheel");
                    wheel[(t + delay) % CONDUCTION_WHEEL_SIZE].push((synapse.target, synapse.strength));
                }
            }
        }

        let mut input: FxHashMap<NeuronId, Current> = FxHashMap::default();

        for (neuron_id, stimulus) in (*stimuli).current_stimuli.iter() {
            if !(*runstate).neuron_refractory.contains_key(neuron_id) {
                *input.entry(*neuron_id).or_insert(0.0) += *stimulus;
            }
        }

        let bucket = (*runstate).timestamp as usize % CONDUCTION_WHEEL_SIZE;
        let wheel = &mut (*runstate).synapse_wheel;
        let due = std::mem::take(&mut wheel[bucket]);
        for (target, strength) in due {
            if !(*runstate).neuron_refractory.contains_key(&target) {
                *input.entry(target).or_insert(0.0) += strength;
            }
        }

        for (neuron_id, neuron) in (*network).neurons.iter() {
            let i = input.get(neuron_id).copied().unwrap_or(0.0);

            match neuron.model {
                NeuronModelKind::IntegrateFire => {
                    get_integrate_fire_state(&mut *runstate, *neuron_id).voltage += i;
                }
                NeuronModelKind::LIF => {
                    //https://www.cns.nyu.edu/~eorhan/notes/lif-neuron.pdf
                    let state = get_lif_state(&mut *runstate, *neuron_id);
                    let dv = -(state.voltage - i) / state.leak_constant;
                    state.voltage += dv * 0.1;
                }
                NeuronModelKind::Izhikevich => {
                    // https://www.izhikevich.org/publications/spikes.pdf
                    let state = get_izhikevich_state(&mut *runstate, *neuron_id);
                    let v = state.voltage;
                    let dv = 0.04 * (v * v) + 5.0 * v + 140.0 - state.recovery_var + i;
                    let du = state.a_var * (state.b_var * v - state.recovery_var);
                    state.voltage += dv * 0.1;
                    state.recovery_var += du * 0.1;
                }
            }
        }

        let mut refractory_remove_indices: Vec<NeuronId> = vec![];

        for (neuron_id, refractory) in (*runstate).neuron_refractory.iter_mut() {
            let neuron = (*network).neurons.get(neuron_id).unwrap();
            if *refractory < neuron.refractory_period {
                *refractory += 1;
            } else {
                refractory_remove_indices.push(*neuron_id);
            }
        }

        for neuron_id in refractory_remove_indices {
            (*runstate).neuron_refractory.remove(&neuron_id);
        }

        (*runstate).timestamp += 1;
    }

    return runstate;
}
