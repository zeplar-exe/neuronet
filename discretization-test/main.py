from neuronet import *
import random
import numpy as np

NEURONS: int = 20
SOURCE_POINTS: int = 50
DEVIATION_RADIUS: int = 1000
DEVIATION_POINTS: int = 1000
INPUT_CURRENT: float = 10.0
MIN_SYNAPSE_CURRENT: int = 10
MAX_SYNAPSE_CURRENT: int = 30
MIN_REFRACTORY_TIME: int = 0
MAX_REFRACTORY_TIME: int = 0
MIN_PROP_TIME: int = 0
MAX_PROP_TIME: int = 0
RUN_DURATION: int = 10000
DEFAULT_IZH_STATE = IzhikevichState(
    voltage=-65.0, reset_potential=-65.0, threshold=30.0,
    a_var=0.02, b_var=0.2, recovery_var=-65.0 * 0.2, d_var=8.0,
)

D = NEURONS * (NEURONS - 1)

def run_trial(network, stimuli, outputs, weight_parameters, prop_parameters):
    events = create_event_container()
    runstate = create_runstate()
    
    for i in range(NEURONS):
        set_izhikevich_state(runstate, i, DEFAULT_IZH_STATE)
    
    for i in range(NEURONS):
        for j in range(NEURONS):
            if i == j:
                continue
            
            syn_idx = i * (NEURONS - 1) + (j if j < i else j - 1)
            
            weight = weight_parameters[syn_idx]
            prop = prop_parameters[syn_idx]
            
            set_synapse_strength(network, syn_idx, weight)
            set_synapse_conduction_time(network, syn_idx, prop)

    for _ in range(RUN_DURATION):
        step(network, runstate, stimuli, events)

    count = get_spike_count(events)
    output_spikes = 0
    
    for event in [get_spike(events, i) for i in range(count)]:
        if event.neuron_id in outputs:
            output_spikes += 1
    
    results = [count, output_spikes, output_spikes / len(outputs)]
    
    destroy_runstate(runstate)
    destroy_event_container(events)
    
    return results

def random_in_ball(centers, radius):
    lengths = [len(center) for center in centers]
    n = sum(lengths)
    r = radius * np.random.uniform(-radius, radius, n)
    
    full = []
    for center in centers: full.extend(center)
    rand = full + r
    rand = rand.tolist()
    for i in range(len(full)):
        if type(full[i]) == int:
            rand[i] = int(rand[i])
    out = []
    
    for length in lengths:
        out.append(rand[:length])
        rand = rand[length:]

    return out

INPUT_COUNT = 5
OUTPUT_COUNT = 5

sources = []

for _ in range(SOURCE_POINTS):
    network = create_network()
    stimuli = create_stimulus_container()
    
    all_neurons = set()
    
    refractory_parameters = [random.randint(MIN_REFRACTORY_TIME, MAX_REFRACTORY_TIME) for _ in range(NEURONS)]
    
    for i in range(NEURONS):
        n = add_izhikevich_neuron(network, refractory_parameters[i])
        all_neurons.add(n)
    
    inputs = random.sample(sorted(all_neurons), INPUT_COUNT)
    
    for input in inputs:
        set_current_stimulus(stimuli, input, INPUT_CURRENT)
    
    for i in range(NEURONS):
        for j in range(NEURONS):
            if i == j:
                continue
            s = add_synapse(network, i, j, 0, 0)

    outputs = random.sample(sorted(all_neurons), OUTPUT_COUNT)

    # include inhibitory neurons later
    weight_parameters = [random.randint(MIN_SYNAPSE_CURRENT, MAX_SYNAPSE_CURRENT) for _ in range(D)]
    prop_parameters = [random.randint(MIN_PROP_TIME, MAX_PROP_TIME) for _ in range(D)]
    
    results = run_trial(network, stimuli, outputs, weight_parameters, prop_parameters)
    
    destroy_stimulus_container(stimuli)
    destroy_network(network)
    
    sources.append((weight_parameters, prop_parameters, refractory_parameters, results))

for source_weight_parameters, source_prop_parameters, source_refractory_parameters, source_results in sources:
    for _ in range(DEVIATION_POINTS):
        network = create_network()
        stimuli = create_stimulus_container()
        
        param = [source_weight_parameters, source_prop_parameters, source_refractory_parameters]
        weight_parameters, prop_parameters, refractory_parameters = random_in_ball(param, DEVIATION_RADIUS)
        
        all_neurons = set()
        
        for i in range(NEURONS):
            n = add_izhikevich_neuron(network, refractory_parameters[i])
            all_neurons.add(n)
        
        inputs = random.sample(sorted(all_neurons), INPUT_COUNT)
        
        for input in inputs:
            set_current_stimulus(stimuli, input, INPUT_CURRENT)
        
        for i in range(NEURONS):
            for j in range(NEURONS):
                if i == j:
                    continue
                s = add_synapse(network, i, j, 0, 0)

        outputs = random.sample(sorted(all_neurons), OUTPUT_COUNT)
        
        deviation_results = run_trial(network, stimuli, outputs, weight_parameters, prop_parameters)
        
        destroy_stimulus_container(stimuli)
        destroy_network(network)
        
        source_parameters = source_weight_parameters + source_prop_parameters + source_refractory_parameters
        deviation_parameters = weight_parameters + prop_parameters + refractory_parameters
        source_parameters = np.array(source_parameters)
        deviation_parameters = np.array(deviation_parameters)
        
        deviation_magnitude = np.sqrt(np.sum(deviation_parameters ** 2))
        deviation_param_distance = np.sqrt(np.sum((source_parameters - deviation_parameters) ** 2))
        results_distance = np.sqrt(np.sum((np.array(source_results) - np.array(deviation_results)) ** 2))
        
        print(f"source: {source_results}, deviation: {deviation_results}, deviation magnitude: {deviation_magnitude:.2f}, deviation parameter distance: {deviation_param_distance:.2f}, results distance: {results_distance:.2f}")
