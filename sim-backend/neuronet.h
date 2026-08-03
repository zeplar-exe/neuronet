#ifndef NEURONET_H
#define NEURONET_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define CONDUCTION_WHEEL_SIZE 5001

typedef enum NeuronModelKind {
  IntegrateFire,
  LIF,
  Izhikevich,
} NeuronModelKind;

typedef struct EventContainer EventContainer;

typedef struct Network Network;

typedef struct Runstate Runstate;

typedef struct StimulusContainer StimulusContainer;

typedef uint32_t NeuronId;

typedef double Voltage;

typedef struct SpikeEvent {
  NeuronId neuron_id;
  int32_t timestamp;
  Voltage voltage;
} SpikeEvent;

typedef double Current;

typedef struct ByteBuffer {
  uint8_t *data;
  uintptr_t len;
} ByteBuffer;

typedef struct NeuronBuffer {
  const NeuronId *data;
  uintptr_t len;
} NeuronBuffer;

typedef uint32_t SynapseId;

typedef struct SynapseBuffer {
  const SynapseId *data;
  uintptr_t len;
} SynapseBuffer;

typedef uint32_t Time;

typedef struct IntegrateFireState {
  Voltage voltage;
  Voltage reset_potential;
  Voltage threshold;
} IntegrateFireState;

typedef struct LifState {
  Voltage voltage;
  Voltage reset_potential;
  Voltage threshold;
  double leak_rate;
  double input_gain;
} LifState;

typedef struct IzhikevichState {
  Voltage voltage;
  Voltage reset_potential;
  Voltage threshold;
  double a_var;
  double b_var;
  double recovery_var;
  double d_var;
} IzhikevichState;

struct EventContainer *create_event_container(void);

void destroy_event_container(struct EventContainer *container);

void clear_events(struct EventContainer *container);

uintptr_t get_spike_count(const struct EventContainer *container);

struct SpikeEvent get_spike(const struct EventContainer *container, uintptr_t index);

struct StimulusContainer *create_stimulus_container(void);

void destroy_stimulus_container(struct StimulusContainer *container);

Voltage get_voltage(const struct Network *network, struct Runstate *runstate, NeuronId id);

Voltage get_threshold(const struct Network *network, struct Runstate *runstate, NeuronId id);

void set_voltage(const struct Network *network,
                 struct Runstate *runstate,
                 NeuronId id,
                 Voltage voltage);

void set_current_stimulus(struct StimulusContainer *stimuli, NeuronId neuron_id, Current current);

struct Runstate *step(struct Network *network,
                      struct Runstate *runstate,
                      const struct StimulusContainer *stimuli,
                      struct EventContainer *events);

struct Network *create_network(void);

void destroy_network(struct Network *network);

struct ByteBuffer serialize_network(const struct Network *network, bool json);

struct Network *deserialize_network(const unsigned char *data, uintptr_t length, bool json);

struct NeuronBuffer get_neurons(const struct Network *network);

struct SynapseBuffer get_synapses(const struct Network *network);

SynapseId add_synapse(struct Network *network, NeuronId source, NeuronId target);

NeuronId add_integrate_fire_neuron(struct Network *network);

NeuronId add_lif_neuron(struct Network *network);

NeuronId add_izhikevich_neuron(struct Network *network);

void network_remove_neuron(struct Network *network, NeuronId id);

void network_remove_synapse(struct Network *network, SynapseId id);

enum NeuronModelKind get_neuron_model(const struct Network *network, NeuronId id);

bool network_has_neuron(struct Network *network, NeuronId id);

bool network_has_synapse(struct Network *network, SynapseId id);

struct ByteBuffer serialize_runstate(const struct Runstate *runstate, bool json);

struct Runstate *deserialize_runstate(const unsigned char *data, uintptr_t length, bool json);

struct Runstate *create_runstate(void);

struct Runstate *clone_runstate(struct Runstate *runstate);

void destroy_runstate(struct Runstate *runstate);

void set_neuron_refractory_period(struct Runstate *runstate, NeuronId id, Time refractory_period);

void set_synapse_strength(struct Runstate *runstate, SynapseId id, Voltage strength);

void set_synapse_conduction_time(struct Runstate *runstate, SynapseId id, Time conduction_time);

void runstate_remove_neuron(struct Runstate *runstate, NeuronId id);

struct IntegrateFireState *get_integrate_fire_state(struct Runstate *runstate, NeuronId id);

struct LifState *get_lif_state(struct Runstate *runstate, NeuronId id);

struct IzhikevichState *get_izhikevich_state(struct Runstate *runstate, NeuronId id);

void set_integrate_fire_state(struct Runstate *runstate,
                              NeuronId neuron_id,
                              struct IntegrateFireState state);

void set_lif_state(struct Runstate *runstate, NeuronId neuron_id, struct LifState state);

void set_izhikevich_state(struct Runstate *runstate,
                          NeuronId neuron_id,
                          struct IzhikevichState state);

void destroy_byte_buffer(struct ByteBuffer buffer);

void destroy_neuron_buffer(struct NeuronBuffer buffer);

void destroy_synapse_buffer(struct SynapseBuffer buffer);

#endif  /* NEURONET_H */
