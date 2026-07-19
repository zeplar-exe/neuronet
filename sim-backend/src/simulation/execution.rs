use std::collections::HashMap;

use crate::{
    models::NeuronModelKind,
    simulation::{
        events::{CurrentApplyEvent, EventContainer, SpikeEvent},
        id::NeuronId,
        network::{Current, Network, Time, Voltage},
        state::{
            get_integrate_fire_state, get_izhikevich_state, get_lif_state, set_integrate_fire_state,
            set_izhikevich_state, set_lif_state, Runstate,
        },
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
            NeuronModelKind::IntegrateFire => get_integrate_fire_state(runstate, id).voltage,
            NeuronModelKind::LIF => get_lif_state(runstate, id).voltage,
            NeuronModelKind::Izhikevich => get_izhikevich_state(runstate, id).voltage,
        }
    }
}

#[no_mangle]
pub extern "C" fn get_threshold(network: *const Network, runstate: *mut Runstate, id: NeuronId) -> Voltage {
    unsafe {
        let neuron = (*network).neurons.get(&id).unwrap();

        match &neuron.model {
            NeuronModelKind::IntegrateFire => get_integrate_fire_state(runstate, id).threshold,
            NeuronModelKind::LIF => get_lif_state(runstate, id).threshold,
            NeuronModelKind::Izhikevich => get_izhikevich_state(runstate, id).threshold,
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
            NeuronModelKind::LIF => get_lif_state(runstate, id).voltage = voltage,
            NeuronModelKind::Izhikevich => get_izhikevich_state(runstate, id).voltage = voltage,
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
                    let mut state = get_integrate_fire_state(runstate, *neuron_id);
                    if state.voltage >= state.threshold {
                        (*events).spikes.push(SpikeEvent {
                            timestamp: (*runstate).timestamp,
                            neuron_id: *neuron_id,
                            voltage: state.voltage,
                        });
                        state.voltage = state.reset_potential;

                        fired = true;
                    }
                }
                NeuronModelKind::LIF => {
                    let mut state = get_lif_state(runstate, *neuron_id);
                    if state.voltage >= state.threshold {
                        (*events).spikes.push(SpikeEvent {
                            timestamp: (*runstate).timestamp,
                            neuron_id: *neuron_id,
                            voltage: state.voltage,
                        });
                        state.voltage = state.reset_potential;

                        fired = true;
                    }
                }
                NeuronModelKind::Izhikevich => {
                    let mut state = get_izhikevich_state(runstate, *neuron_id);
                    if state.voltage >= state.threshold {
                        (*events).spikes.push(SpikeEvent {
                            timestamp: (*runstate).timestamp,
                            neuron_id: *neuron_id,
                            voltage: state.voltage,
                        });
                        state.voltage = state.reset_potential;
                        state.recovery_var = state.recovery_var + state.d_var;

                        fired = true;
                        // hmm.. well, we need to do the current additive stuff directly in here
                        // instead of globally.. so the question is how did izhikevich and co. do
                        // their simulation? as in, what determines the amount of current that
                        // comes from an incoming synapse (this can be found in stdp.pdf)
                    }
                }
            }

            if fired {
                for synapse_id in (*neuron).outgoing.iter() {
                    (*runstate)
                        .synapse_propagation
                        .entry(*synapse_id)
                        .or_insert(vec![])
                        .push(0);
                }
            }
        }

        let apply_stimulus = |id: NeuronId, stimulus: Current| {
            if (*runstate).neuron_refractory.contains_key(&id) {
                return;
            }

            let neuron = (*network).neurons.get(&id).unwrap();

            match neuron.model {
                NeuronModelKind::IntegrateFire => {
                    let mut state = get_integrate_fire_state(runstate, id);
                    state.voltage += stimulus;
                    set_integrate_fire_state(runstate, id, *state);
                }
                NeuronModelKind::LIF => {}
                NeuronModelKind::Izhikevich => {
                    let mut state = get_izhikevich_state(runstate, id);
                    let v = state.voltage as f64 * 1e-4;
                    let i = stimulus as f64;
                    let dv = 0.04 * (v * v) + 5.0 * v + 140.0 - state.recovery_var + i;
                    let du = state.a_var * (state.b_var * v - state.recovery_var);

                    state.voltage += dv; //dv.round() as Voltage;
                    state.recovery_var += du; //du.round() as f32;

                    set_izhikevich_state(runstate, id, *state);
                }
            }
        };

        for (neuron_id, stimulus) in (*stimuli).current_stimuli.iter() {
            apply_stimulus(*neuron_id, *stimulus);
        }

        for (synapse_id, prop) in (*runstate).synapse_propagation.iter_mut() {
            let synapse = (*network).synapses.get(synapse_id).unwrap();
            let mut update_indices: Vec<i8> = vec![];
            for p in prop.iter() {
                if (*p + 1) >= synapse.conduction_time {
                    update_indices.push(-1);
                } else {
                    update_indices.push(1);
                }
            }
            for (idx, v) in update_indices.iter().enumerate() {
                if *v == -1 {
                    apply_stimulus(synapse.target, synapse.strength);
                    continue;
                }
                prop[idx] += 1;
            }
            let mut i = 0;
            prop.retain(|_| {
                i += 1;
                update_indices[i - 1] != -1
            });
        }

        let refractory_remove_indices: Vec<&NeuronId> = vec![];

        for (neuron_id, refractory) in (*runstate).neuron_refractory.iter_mut() {
            let neuron = (*network).neurons.get(neuron_id).unwrap();
            if *refractory < neuron.refractory_period {
                *refractory += 1;
            } else {
                refractory_remove_indices.push(neuron_id);
            }
        }

        for neuron_id in refractory_remove_indices {
            (*runstate).neuron_refractory.remove(neuron_id);
        }

        (*runstate).timestamp += 1;
    }

    return runstate;
}
