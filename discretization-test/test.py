from neuronet import *

network = create_network()
runstate = create_runstate()
events = create_event_container()
stimuli = create_stimulus_container()

n = add_izhikevich_neuron(network, 0)
set_default_izhikevich_state(runstate, IzhikevichState(
    voltage=-65.0, reset_potential=-65.0, threshold=30.0,
    a_var=0.02, b_var=0.2, recovery_var=-65.0 * 0.2, d_var=8.0,
))
fill_defaults(network, runstate)
set_current_stimulus(stimuli, n, 10.0)

for _ in range(10000):
    step(network, runstate, stimuli, events)

count = get_spike_count(events)
print(f"spikes: {count}")

destroy_event_container(events)
destroy_stimulus_container(stimuli)
destroy_runstate(runstate)
destroy_network(network)
