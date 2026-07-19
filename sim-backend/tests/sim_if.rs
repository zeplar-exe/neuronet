use nn_lib::simulation::{
    events::{create_event_container, destroy_event_container},
    execution::{create_stimulus_container, destroy_stimulus_container, get_voltage, set_current_stimulus, step},
    network::{
        add_integrate_fire_neuron, add_izhikevich_neuron, add_synapse, create_network, destroy_network, Current, Time,
        Voltage,
    },
    state::{
        create_runstate, destroy_runstate, fill_defaults, set_default_izhikevich_state, set_integrate_fire_state,
        IntegrateFireState, IzhikevichState,
    },
};

#[test]
fn test_add_integrate_fire_neuron() {
    let network = create_network();
    let if1 = add_integrate_fire_neuron(network, 0);

    unsafe {
        assert_eq!((*network).neurons.len(), 1);
    }
}

#[test]
fn test_add_current_stimulus() {
    let network = create_network();
    let if1 = add_integrate_fire_neuron(network, 0);
    let stimuli = create_stimulus_container();
    set_current_stimulus(stimuli, if1, 35.0);

    unsafe {
        assert_eq!((*stimuli).current_stimuli.len(), 1);
        assert_eq!((*stimuli).current_stimuli.get(&if1), Some(&35.0));
    }
}

#[test]
fn test_spike() {
    const CURRENT_STIMULUS: Voltage = 35.0;
    const SYNAPSE_STIMULUS: Voltage = 40.0;

    let network = create_network();
    let events = create_event_container();
    let if1 = add_integrate_fire_neuron(network, 0);
    let if2 = add_integrate_fire_neuron(network, 0);
    let _s1 = add_synapse(network, if1, if2, SYNAPSE_STIMULUS, 0);

    let empty_stimuli = create_stimulus_container();
    let stimuli = create_stimulus_container();
    set_current_stimulus(stimuli, if1, CURRENT_STIMULUS);

    let runstate = create_runstate();
    set_integrate_fire_state(
        runstate,
        if1,
        IntegrateFireState {
            voltage: 0.0,
            reset_potential: 0.0,
            threshold: 30.0,
        },
    );
    set_integrate_fire_state(
        runstate,
        if2,
        IntegrateFireState {
            voltage: 0.0,
            reset_potential: 0.0,
            threshold: 30.0,
        },
    );

    unsafe {
        let events_ref = events.as_ref().expect("events pointer is null");

        assert_eq!((*events).spikes.len(), 0);

        step(network, runstate, stimuli, events);
        assert_eq!((*runstate).timestamp, 1);
        assert_eq!((*events).spikes.len(), 0);
        assert_eq!(get_voltage(network, runstate, if1), CURRENT_STIMULUS);
        assert_eq!(get_voltage(network, runstate, if2), 0.0);

        step(network, runstate, empty_stimuli, events);
        assert_eq!((*runstate).timestamp, 2);
        assert_eq!((*events).spikes.len(), 1);
        assert_eq!(events_ref.spikes.get_unchecked(0).neuron_id, if1);
        assert_eq!(events_ref.spikes.get_unchecked(0).voltage, CURRENT_STIMULUS);
        assert_eq!(get_voltage(network, runstate, if1), 0.0);
        assert_eq!(get_voltage(network, runstate, if2), SYNAPSE_STIMULUS);

        step(network, runstate, empty_stimuli, events);
        assert_eq!((*runstate).timestamp, 3);
        assert_eq!((*events).spikes.len(), 2);
        assert_eq!(events_ref.spikes.get_unchecked(1).neuron_id, if2);
        assert_eq!(events_ref.spikes.get_unchecked(1).voltage, SYNAPSE_STIMULUS);
        assert_eq!(get_voltage(network, runstate, if1), 0.0);
        assert_eq!(get_voltage(network, runstate, if2), 0.0);
    }

    destroy_network(network);
    destroy_event_container(events);
    destroy_stimulus_container(stimuli);
    destroy_runstate(runstate);
}

#[test]
fn test_3_step_spike() {
    const CURRENT_STIMULUS: Voltage = 35.0;
    const SYNAPSE_STIMULUS: Voltage = 40.0;

    let network = create_network();
    let events = create_event_container();
    let if1 = add_integrate_fire_neuron(network, 0);
    let if2 = add_integrate_fire_neuron(network, 0);
    let _s1 = add_synapse(network, if1, if2, SYNAPSE_STIMULUS, 3);

    let empty_stimuli = create_stimulus_container();
    let stimuli = create_stimulus_container();
    set_current_stimulus(stimuli, if1, CURRENT_STIMULUS);

    let runstate = create_runstate();
    set_integrate_fire_state(
        runstate,
        if1,
        IntegrateFireState {
            voltage: 0.0,
            reset_potential: 0.0,
            threshold: 30.0,
        },
    );
    set_integrate_fire_state(
        runstate,
        if2,
        IntegrateFireState {
            voltage: 0.0,
            reset_potential: 0.0,
            threshold: 30.0,
        },
    );

    unsafe {
        step(network, runstate, stimuli, events);
        assert_eq!((*events).spikes.len(), 0);

        step(network, runstate, empty_stimuli, events);
        assert_eq!((*events).spikes.len(), 1);
        assert_eq!(get_voltage(network, runstate, if2), 0.0);

        step(network, runstate, empty_stimuli, events);
        assert_eq!(get_voltage(network, runstate, if2), 0.0);

        step(network, runstate, empty_stimuli, events);
        assert_eq!(get_voltage(network, runstate, if2), SYNAPSE_STIMULUS);
    }

    destroy_network(network);
    destroy_event_container(events);
    destroy_stimulus_container(stimuli);
    destroy_runstate(runstate);
}

#[test]
fn test_poisson_100() {
    const CURRENT_STIMULUS: Voltage = 35.0;
    const EXCITATORY_STIMULUS: Voltage = 36.0;
    const INHIBITORY_STIMULUS: Voltage = -15.0;
    const REFRACTORY: Time = 30; // 3ms
    const CONNECTIVITY: f32 = 0.3;
    const EXCITATORY_RATE: f32 = 0.8;

    let network = create_network();
    let events = create_event_container();

    let empty_stimuli = create_stimulus_container();
    let stimuli = create_stimulus_container();

    let mut neurons = Vec::new();

    for _ in 0..100 {
        let n = add_izhikevich_neuron(network, REFRACTORY);

        neurons.push(n);
    }

    for i in 0..100 {
        for j in 0..100 {
            if i == j {
                continue;
            }
            if rand::random::<f32>() < CONNECTIVITY {
                let duration = rand::random_range(3..10);

                if rand::random::<f32>() < EXCITATORY_RATE {
                    add_synapse(network, i, j, EXCITATORY_STIMULUS, duration);
                } else {
                    add_synapse(network, i, j, INHIBITORY_STIMULUS, duration);
                }
            }
        }
    }

    for _ in 0..20 {
        set_current_stimulus(stimuli, neurons[rand::random_range(0..100)], CURRENT_STIMULUS);
    }

    let runstate = create_runstate();
    set_default_izhikevich_state(
        runstate,
        IzhikevichState {
            voltage: -65 as Voltage,
            reset_potential: -70 as Voltage,
            threshold: 30 as Voltage,
            a_var: 0.02,
            b_var: 0.2,
            recovery_var: 0.0,
            d_var: 8.0,
        },
    );
    fill_defaults(network, runstate);

    unsafe {
        // 1000ms
        step(network, runstate, stimuli, events);
        for i in 1..10 * 1000 {
            step(network, runstate, empty_stimuli, events);
        }

        dbg!((*events).spikes.len());
    }

    destroy_network(network);
    destroy_event_container(events);
    destroy_stimulus_container(stimuli);
    destroy_runstate(runstate);
}
