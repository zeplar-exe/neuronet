pub type Current = i16;

use std::collections::{HashMap, HashSet};

use crate::simulation::{
    id::{NeuronId, SynapseId},
    network::{get_threshold, get_voltage, set_voltage, Network, Runstate, Voltage},
};

const TIME_CONV: f32 = 10e-4 as f32;

pub struct VoltageStimulus {
    pub neuron_id: NeuronId,
    pub voltage: Voltage,
}

pub struct CurrentStimulus {
    pub neuron_id: NeuronId,
    pub current: Current,
}

pub struct StimulusContainer {
    pub voltage_stimuli: HashMap<NeuronId, VoltageStimulus>,
    pub current_stimuli: HashMap<NeuronId, CurrentStimulus>,
}

pub fn set_voltage_stimulus(stimuli: *mut StimulusContainer, neuron_id: NeuronId, voltage: Voltage) {
    unsafe {
        (*stimuli)
            .voltage_stimuli
            .insert(neuron_id, VoltageStimulus { neuron_id, voltage });
    }
}

pub fn set_current_stimulus(stimuli: *mut StimulusContainer, neuron_id: NeuronId, current: Current) {
    unsafe {
        (*stimuli)
            .current_stimuli
            .insert(neuron_id, CurrentStimulus { neuron_id, current });
    }
}

pub fn step_single(
    network: *mut Network,
    runstate: *mut Runstate,
    stimuli: *const StimulusContainer,
    dt: i16, // dt in 10e-4
) -> *mut Runstate {
    let fdt = dt as f32 * TIME_CONV;
    let mut fired: HashSet<NeuronId> = HashSet::default();

    unsafe {
        for (neuron_id, stimulus) in (*stimuli).voltage_stimuli.iter() {
            let voltage = get_voltage(network, *neuron_id);
            set_voltage(network, *neuron_id, voltage + stimulus.voltage);
        }

        for (neuron_id, stimulus) in (*stimuli).current_stimuli.iter() {
            let voltage = get_voltage(network, *neuron_id);
            set_voltage(network, *neuron_id, max(int(voltage + stimulus.current * fdt), 1));
        }

        for (synapse_id, prop) in (*runstate).synapse_propagation.iter() {
            let synapse = (*network).synapses.get(synapse_id).unwrap();
            let update_indices: Vec<f32> = vec![];
            for (i, p) in prop.iter().enumerate() {
                if *p >= 1.0 {
                    update_indices.push(-1.0);
                    let voltage = get_voltage(network, synapse.target);
                    set_voltage(network, synapse.target, voltage + synapse.weight);
                } else {
                    update_indices.push(*p);
                }
            }
            for (idx, v) in update_indices.iter().enumerate() {
                if *v == -1.0 {
                    continue;
                }
                prop[idx] += v;
            }
            prop.retain(|x| *x != -1.0);
        }

        for (neuron_id, prop) in (*runstate).neuron_propagation.iter_mut() {
            let neuron = (*network).neurons.get(neuron_id).unwrap();
            let update_indices: Vec<f32> = vec![];
            for (i, p) in prop.iter().enumerate() {
                if *p >= 1.0 {
                    update_indices.push(-1.0);
                    for synapse_id in (*neuron).outgoing.iter() {
                        (*runstate).synapse_propagation.entry(*synapse_id).or_insert(vec![]);
                        (*runstate).synapse_propagation[synapse_id].push(0.0);
                    }
                } else {
                    update_indices.push(*p);
                }
            }
            for (idx, v) in update_indices.iter().enumerate() {
                if *v == -1.0 {
                    continue;
                }
                prop[idx] += v;
            }
            prop.retain(|x| *x != -1.0);
        }

        for (neuron_id, neuron) in (*network).neurons.iter() {
            let voltage = get_voltage(network, *neuron_id);
            let threshold = get_threshold(network, *neuron_id);

            if voltage >= threshold {
                let fired = true;

                // handle firing logic per neuron model

                if fired {
                    (*runstate).neuron_propagation.entry(*neuron_id).or_insert(vec![]);
                    (*runstate).neuron_propagation[neuron_id].push(0.0);
                }
            }
        }
    }

    return runstate;
}

pub fn step_roll(
    network: *mut Network,
    runstate: *mut Runstate,
    stimuli: *const StimulusContainer,
    dt: i16, // dt in 10e-4
) -> *mut Runstate {
    unsafe {
        let fdt = dt as f32 * TIME_CONV;
        let mut seen: HashSet<NeuronId> = HashSet::default();
        let mut fired: HashSet<NeuronId> = HashSet::default();

        let conduct_neuron;
        let conduct_synapse;
        let fire_neuron;

        for (neuron_id, stimulus) in (*stimuli).voltage_stimuli.iter() {
            let voltage = get_voltage(network, *neuron_id);
            set_voltage(network, *neuron_id, voltage + stimulus.voltage);
            seen.insert(*neuron_id);

            if voltage > get_threshold(network, *neuron_id) {
                (*runstate).neuron_propagation.insert(*neuron_id, 0.0);
                conduct_neuron(*neuron_id, fdt);
            }
        }

        for (neuron_id, stimulus) in (*stimuli).current_stimuli.iter() {
            let voltage = get_voltage(network, *neuron_id);
            set_voltage(network, *neuron_id, voltage + stimulus.current * fdt);

            if voltage > get_threshold(network, *neuron_id) {
                (*runstate).neuron_propagation.insert(*neuron_id, 0.0);
                conduct_neuron(*neuron_id, fdt);
            }
        }

        let conduct_neuron = |id: NeuronId, remaining_dt: f32| {
            let neuron = (*network).neurons.get(&id).unwrap();
            let prop = (*runstate).neuron_propagation.get(&id).unwrap_or(-1.0 as f32);

            if *prop == -1.0 {
                return;
            }

            let cond_vel = neuron.conduction_velocity;
            let prop_remaining = 1.0 - *prop;

            if cond_vel * remaining_dt >= prop_remaining {
                remaining_dt -= cond_vel / prop_remaining;
                for outgoing in neuron.outgoing.iter() {
                    let synapse = (*network).synapses.get(outgoing).unwrap();
                    let synapse_prop = (*runstate).synapse_propagation.get(outgoing).unwrap_or(&0.0);

                    conduct_synapse(synapse, remaining_dt);
                }

                (*runstate).neuron_propagation.insert(id, 0.0);
            } else {
                remaining_dt -= cond_vel * prop_remaining;
                (*runstate)
                    .neuron_propagation
                    .insert(id, *prop + cond_vel * remaining_dt);
            }
        };

        let conduct_synapse = |id: SynapseId, remaining_dt: f32| {
            let synapse = (*network).synapses.get(&id).unwrap();
            let prop = (*runstate).synapse_propagation.get(&id).unwrap_or(&-1.0);

            if *prop == -1.0 {
                return;
            }

            let cond_vel = synapse.conduction_velocity;
            let prop_remaining = 1.0 - *prop;

            if cond_vel * remaining_dt >= prop_remaining {
                remaining_dt -= cond_vel / prop_remaining;

                fire_neuron(synapse.target, remaining_dt);

                (*runstate).synapse_propagation.remove(&id);
            } else {
                remaining_dt -= cond_vel * prop_remaining;
                (*runstate)
                    .synapse_propagation
                    .insert(id, *prop + cond_vel * remaining_dt);
            }
        };

        let fire_neuron = |id: NeuronId, remaining_dt: f32| {
            let neuron = (*network).neurons.get(&id).unwrap();

            // need to implement the neuron models here
            // then conduct to any synapses if need be
        };

        runstate
    }
}
