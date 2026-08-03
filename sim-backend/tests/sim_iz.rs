use neuronet::simulation::{
    events::{create_event_container, destroy_event_container},
    execution::{create_stimulus_container, destroy_stimulus_container, set_current_stimulus, step},
    network::{add_izhikevich_neuron, add_synapse, create_network, destroy_network, Voltage},
    state::{create_runstate, destroy_runstate, set_izhikevich_state, set_neuron_refractory_period, set_synapse_strength, set_synapse_conduction_time, IzhikevichState},
};

const DEFAULT_IZH: IzhikevichState = IzhikevichState {
    voltage: -65.0,
    reset_potential: -65.0,
    threshold: 30.0,
    a_var: 0.02,
    b_var: 0.2,
    recovery_var: -13.0,
    d_var: 8.0,
};

#[test]
fn test_izhikevich_single() {
    const STIMULUS: Voltage = 35.0;

    let network = create_network();
    let events = create_event_container();
    let stimuli = create_stimulus_container();

    let n = add_izhikevich_neuron(network);
    set_current_stimulus(stimuli, n, STIMULUS);

    let runstate = create_runstate();
    set_izhikevich_state(runstate, n, DEFAULT_IZH);

    unsafe {
        for _ in 0..(10 * 1000) {
            step(network, runstate, stimuli, events);
        }
        let n_spikes = (*events).spikes.len();

        assert!(
            (50..=400).contains(&n_spikes),
            "expected regular spiking (~150 Hz), got {n_spikes}"
        );
    }

    destroy_network(network);
    destroy_event_container(events);
    destroy_stimulus_container(stimuli);
    destroy_runstate(runstate);
}

#[test]
fn test_poisson_100() {
    const CURRENT_STIMULUS: Voltage = 35.0;
    const EXCITATORY_STIMULUS: Voltage = 35.0;
    const INHIBITORY_STIMULUS: Voltage = -15.0;
    const REFRACTORY: u32 = 30;
    const CONNECTIVITY: f32 = 0.3;
    const EXCITATORY_RATE: f32 = 0.8;

    let network = create_network();
    let events = create_event_container();

    let empty_stimuli = create_stimulus_container();
    let stimuli = create_stimulus_container();

    let mut neurons = Vec::new();

    for _ in 0..100 {
        let n = add_izhikevich_neuron(network);
        neurons.push(n);
    }

    let mut synapses = Vec::new();
    for i in 0..100 {
        for j in 0..100 {
            if i == j {
                continue;
            }
            if rand::random::<f32>() < CONNECTIVITY {
                let duration = rand::random_range(3..10);
                let strength = if rand::random::<f32>() < EXCITATORY_RATE {
                    EXCITATORY_STIMULUS
                } else {
                    INHIBITORY_STIMULUS
                };
                let s = add_synapse(network, i, j);
                synapses.push((s, strength, duration));
            }
        }
    }

    for _ in 0..20 {
        set_current_stimulus(stimuli, neurons[rand::random_range(0..100)], CURRENT_STIMULUS);
    }

    let runstate = create_runstate();
    for &n in &neurons {
        set_izhikevich_state(runstate, n, DEFAULT_IZH);
        set_neuron_refractory_period(runstate, n, REFRACTORY);
    }
    for (s, strength, duration) in synapses {
        set_synapse_strength(runstate, s, strength);
        set_synapse_conduction_time(runstate, s, duration);
    }

    unsafe {
        for _ in 0..(10 * 1000) {
            step(network, runstate, stimuli, events);
        }

        dbg!((*events).spikes.len());
    }

    destroy_network(network);
    destroy_event_container(events);
    destroy_stimulus_container(stimuli);
    destroy_stimulus_container(empty_stimuli);
    destroy_runstate(runstate);
}
