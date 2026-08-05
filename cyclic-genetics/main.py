from neuronet import *
import random
import numpy as np
import itertools



# input and output locations need to be fixed for every organism
    # all that should vary is the edge weights for now
# but how do we implement the reproduction rule?
    # maybe we should add hebbian learning for during-trial growth?


NEURONS = 100
ADJACENT_CONNECTIVITY = 0.85
INPUTS = {
    "thalamic": 20,
    "sensor_north": 3,
    "sensor_south": 3,
    "sensor_east": 3,
    "sensor_west": 3,
}
OUTPUTS = {
    "move_north": 2,
    "move_south": 2,
    "move_east": 2,
    "move_west": 2,
}
DEFAULT_LIF_STATE = LifState(
    voltage=0,
    threshold=20,
    reset_potential=0,
    leak_rate=0.02,
    input_gain=1
)

class Organism:
    def __init__(self, network, neuron_grid, all_synapses, input_neuron_map, output_neuron_map, weights=None):
        self.network = network
        self.stimuli = create_stimulus_container()
        self.events = create_event_container()
        self.runstate = create_runstate()

        self.input_neuron_map = input_neuron_map
        self.output_neuron_map = output_neuron_map
        self.input_neurons = {}
        self.output_neurons = {}

        for mapped_input_index, input_type in self.input_neuron_map.items():
            neuron_id = neuron_grid[mapped_input_index]
            self.input_neurons[neuron_id] = input_type
        for mapped_output_index, output_type in self.output_neuron_map.items():
            neuron_id = neuron_grid[mapped_output_index]
            self.output_neurons[neuron_id] = output_type

        for i, j, k in itertools.product(range(neuron_grid.shape[0]), range(neuron_grid.shape[1]), range(neuron_grid.shape[2])):
            source_neuron = neuron_grid[i][j][k]
            set_lif_state(self.runstate, source_neuron, DEFAULT_LIF_STATE)
            set_neuron_refractory_period(self.runstate, source_neuron, 15)

        generated_weights = np.zeros(len(all_synapses))
        
        for idx, (synapse_id, i, j, k) in enumerate(all_synapses):
            weight = weights[idx] if weights is not None else random.uniform(10, 30)
            
            if random.random() > ADJACENT_CONNECTIVITY:
                weight = 0
            
            set_synapse_conduction_time(self.runstate, synapse_id, 0)
            set_synapse_strength(self.runstate, synapse_id, weight)
            
            if weights is None:
                generated_weights[idx] = weight

        self.weights = weights if weights is not None else generated_weights
    
    def reset_runstate(self):
        for neuron in iter_neurons(self.network):
            runstate_remove_neuron(self.runstate, neuron)
            set_lif_state(self.runstate, neuron, DEFAULT_LIF_STATE)
            set_neuron_refractory_period(self.runstate, neuron, 15)
        clear_synapse_wheel(self.runstate)

    def take_input(self, inputs):
        for neuron_id, input_type in self.input_neurons.items():
            if input_type in inputs:
                set_current_stimulus(self.stimuli, neuron_id, inputs[input_type])
            else:
                set_current_stimulus(self.stimuli, neuron_id, 0)
    
    def tick(self, dt):
        for _ in range(dt):
            step(self.network, self.runstate, self.stimuli, self.events)
        
        output_spikes = []
        
        for i in range(get_spike_count(self.events)):
            neuron_id = get_spike(self.events, i).neuron_id
            
            if neuron_id in self.output_neurons:
                output_spikes.append(self.output_neurons[neuron_id])
        
        clear_events(self.events)
        return output_spikes
    
    def reproduce_with(self, our_score, other_score, other: "Organism"):
        higher_weight = (max(our_score, other_score) - min(our_score, other_score)) / max(our_score, other_score)
        lower_weight = 1 - higher_weight
        our_weights = self.weights if our_score < other_score else other.weights
        their_weights = other.weights if our_score < other_score else self.weights
        our_weight = higher_weight if our_score < other_score else lower_weight
        their_weight = lower_weight if our_score < other_score else higher_weight
        
        return our_weights * our_weight + their_weights * their_weight
    
    def destroy(self):
        destroy_stimulus_container(self.stimuli)
        destroy_event_container(self.events)
        destroy_runstate(self.runstate)


network = create_network()
neurons = []

for _ in range(NEURONS):
    n = add_lif_neuron(network)
    neurons.append(n)

neurons = np.reshape(neurons, (10, 1, 10))

neighbor_indices = [d for d in itertools.product([0, 1, -1], [0, 1, -1], [0, 1, -1]) if d != (0, 0, 0)]
all_synapses = []

for i, j, k in itertools.product(range(neurons.shape[0]), range(neurons.shape[1]), range(neurons.shape[2])):
    for di, dj, dk in neighbor_indices:
        ti, tj, tk = i + di, j + dj, k + dk
        
        if 0 <= ti < neurons.shape[0] and 0 <= tj < neurons.shape[1] and 0 <= tk < neurons.shape[2]:
            synapse_id = add_synapse(network, neurons[i][j][k], neurons[ti][tj][tk])
            all_synapses.append((synapse_id, i, j, k))

available_neurons = list(np.ndindex((10, 1, 10)))
input_neuron_map = {}
output_neuron_map = {}

for input_type, count in INPUTS.items():
    for _ in range(count):
        index = random.choice(available_neurons)
        input_neuron_map[index] = input_type
        available_neurons.remove(index)
for output_type, count in OUTPUTS.items():
    for _ in range(count):
        index = random.choice(available_neurons)
        output_neuron_map[index] = output_type
        available_neurons.remove(index)


ORGANISM_COUNT = 200
ITERATIONS = 500
TICKS = 2000
TICK_DT = 10
MIN_SENSOR_DISTANCE = 20
MAX_SENSOR_CURRENT = 40

organisms = [Organism(network, neurons, all_synapses, input_neuron_map, output_neuron_map) for _ in range(ORGANISM_COUNT)]
goal_position = [10, 10]

for iteration in range(ITERATIONS):
    organism_positions = {}
    for _, organism in enumerate(organisms):
        organism_positions[organism] = [0, 0]
    
    for organism in organisms:
        organism.reset_runstate()
        
        for _ in range(TICKS):
            north_dist = max(0, goal_position[1] - organism_positions[organism][1])
            south_dist = max(0, organism_positions[organism][1] - goal_position[1])
            east_dist = max(0, goal_position[0] - organism_positions[organism][0])
            west_dist = max(0, organism_positions[organism][0] - goal_position[0])
            organism.take_input({
                "sensor_north": (1 - (north_dist / MIN_SENSOR_DISTANCE)) * MAX_SENSOR_CURRENT if abs(north_dist) < MIN_SENSOR_DISTANCE else 0.0, 
                "sensor_south": (1 - (south_dist / MIN_SENSOR_DISTANCE)) * MAX_SENSOR_CURRENT if abs(south_dist) < MIN_SENSOR_DISTANCE else 0.0,
                "sensor_west": (1 - (west_dist / MIN_SENSOR_DISTANCE)) * MAX_SENSOR_CURRENT if abs(west_dist) < MIN_SENSOR_DISTANCE else 0.0,
                "sensor_east": (1 - (east_dist / MIN_SENSOR_DISTANCE)) * MAX_SENSOR_CURRENT if abs(east_dist) < MIN_SENSOR_DISTANCE else 0.0,
                "thalamic": 60
            })
            output_spikes = organism.tick(TICK_DT)
            for neuron in iter_neurons(organism.network):
                voltage = get_voltage(organism.network, organism.runstate, neuron)
                #if voltage > 0:
                #    print(f"Neuron {neuron} voltage: {voltage}")
            
            for output in output_spikes:
                if output == "move_north":
                    organism_positions[organism][1] += 1
                elif output == "move_south":
                    organism_positions[organism][1] -= 1
                elif output == "move_east":
                    organism_positions[organism][0] += 1
                elif output == "move_west":
                    organism_positions[organism][0] -= 1

    scores = []
    for organism in organisms:
        position = organism_positions[organism]
        distance = np.linalg.norm(np.array(position) - np.array(goal_position))
        scores.append((distance.item(), organism))
    scores = sorted(scores, key=lambda x: x[0])
    
    chunked_scores = [scores[i:i + 5] for i in range(0, len(scores), 5)]
    for chunk in chunked_scores:
        random.shuffle(chunk)
    chunked_combined = [item for sublist in chunked_scores for item in sublist]

    for i in range(0, len(chunked_combined), 2):
        our_score, our_organism = chunked_combined[i]
        other_score, other_organism = chunked_combined[i+1]
        new_weights = our_organism.reproduce_with(our_score, other_score, other_organism)
        new_organism = Organism(network, neurons, all_synapses, input_neuron_map, output_neuron_map, weights=new_weights)
        organisms.append(new_organism)

    to_remove = scores[ORGANISM_COUNT//2:ORGANISM_COUNT]
    for score, organism in to_remove:
        organisms.remove(organism)
        organism.destroy()
    
    print("Best 5 Scores:", [s[0] for s in scores[:5]])

for organism in organisms:
    organism.destroy()

destroy_network(network)