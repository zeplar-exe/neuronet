use neuronet::simulation::{
    events::{create_event_container, destroy_event_container},
    execution::{create_stimulus_container, destroy_stimulus_container, get_voltage, set_current_stimulus, step},
    network::{add_integrate_fire_neuron, add_synapse, create_network, destroy_network, Current, Time, Voltage},
    state::{create_runstate, destroy_runstate, fill_defaults, set_integrate_fire_state, IntegrateFireState},
};

#[test]
fn test_add_integrate_fire_neuron() {
    let network = create_network();
    let _if1 = add_integrate_fire_neuron(network, 0);

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

    destroy_network(network);
    destroy_stimulus_container(stimuli);
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
    destroy_stimulus_container(empty_stimuli);
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
    destroy_stimulus_container(empty_stimuli);
    destroy_runstate(runstate);
}

#[test]
fn test_four_neuron_cycle() {
    const CURRENT_STIMULUS: Voltage = 35.0;
    const SYNAPSE_STIMULUS: Voltage = 40.0;
    const THRESHOLD: Voltage = 30.0;

    let network = create_network();
    let events = create_event_container();
    let empty_stimuli = create_stimulus_container();
    let stimuli = create_stimulus_container();

    let neurons: Vec<_> = (0..4).map(|_| add_integrate_fire_neuron(network, 0)).collect();
    add_synapse(network, neurons[0], neurons[1], SYNAPSE_STIMULUS, 1);
    add_synapse(network, neurons[1], neurons[2], SYNAPSE_STIMULUS, 1);
    add_synapse(network, neurons[2], neurons[3], SYNAPSE_STIMULUS, 1);
    add_synapse(network, neurons[3], neurons[0], SYNAPSE_STIMULUS, 1);

    set_current_stimulus(stimuli, neurons[0], CURRENT_STIMULUS);

    let runstate = create_runstate();
    for &n in &neurons {
        set_integrate_fire_state(
            runstate,
            n,
            IntegrateFireState {
                voltage: 0.0,
                reset_potential: 0.0,
                threshold: THRESHOLD,
            },
        );
    }

    unsafe {
        step(network, runstate, stimuli, events);
        for _ in 1..20 {
            step(network, runstate, empty_stimuli, events);
        }

        assert_eq!((*runstate).timestamp, 20);
        assert!((*events).spikes.len() > 4, "pulse should circulate the ring");
    }

    destroy_network(network);
    destroy_event_container(events);
    destroy_stimulus_container(stimuli);
    destroy_stimulus_container(empty_stimuli);
    destroy_runstate(runstate);
}
