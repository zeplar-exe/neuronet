use neuronet::simulation::{
    events::{create_event_container, destroy_event_container},
    execution::{create_stimulus_container, destroy_stimulus_container, get_voltage, set_current_stimulus, step},
    network::{add_lif_neuron, create_network, destroy_network, Voltage},
    state::{create_runstate, destroy_runstate, set_lif_state, LifState},
};

#[test]
fn test_lif_single_neuron() {
    let tau: f64 = 10.0;
    let threshold: Voltage = 20.0;
    let current: f64 = 25.0;

    let network = create_network();
    let events = create_event_container();
    let stimuli = create_stimulus_container();
    let n = add_lif_neuron(network, 0);

    set_current_stimulus(stimuli, n, current);

    let runstate = create_runstate();
    set_lif_state(
        runstate,
        n,
        LifState {
            voltage: 0.0,
            resting_potential: 0.0,
            reset_potential: 0.0,
            threshold,
            leak_constant: tau,
        },
    );

    // Analytical time solution for spike
    let expected_steps = (-tau * (1.0 - threshold / current).ln() / 0.1) as i32;

    unsafe {
        let mut spike_step = None;
        for s in 0..500 {
            step(network, runstate, stimuli, events);
            if !(*events).spikes.is_empty() {
                spike_step = Some(s + 1);
                break;
            }
        }

        let spike_step = spike_step.expect("neuron should spike");
        assert!(
            (spike_step - expected_steps).abs() <= 2,
            "spike at step {spike_step}, expected ~{expected_steps}"
        );

        assert!(
            get_voltage(network, runstate, n).abs() < 1.0,
            "voltage should reset near 0"
        );
    }

    destroy_network(network);
    destroy_event_container(events);
    destroy_stimulus_container(stimuli);
    destroy_runstate(runstate);
}
