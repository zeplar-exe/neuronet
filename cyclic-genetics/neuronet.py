import ctypes
import os
from ctypes import c_void_p, c_uint, c_int, c_double, c_size_t, c_bool, c_ubyte, POINTER, Structure
from enum import IntEnum

_lib_path = os.path.join(os.path.dirname(__file__), "..", "sim-backend", "target", "release", "libneuronet.dylib")
_lib = ctypes.CDLL(_lib_path)

class NeuronModelKind(IntEnum):
    IntegrateFire = 0
    LIF = 1
    Izhikevich = 2

class SpikeEvent(Structure):
    neuron_id: c_uint
    timestamp: c_int
    voltage: c_double
    _fields_ = [
        ("neuron_id", c_uint),
        ("timestamp", c_int),
        ("voltage", c_double),
    ]

class ByteBuffer(Structure):
    data: POINTER(c_ubyte)
    len: c_size_t
    _fields_ = [
        ("data", POINTER(c_ubyte)),
        ("len", c_size_t),
    ]

class NeuronBuffer(Structure):
    data: POINTER(c_uint)
    len: c_size_t
    _fields_ = [
        ("data", POINTER(c_uint)),
        ("len", c_size_t),
    ]

class SynapseBuffer(Structure):
    data: POINTER(c_uint)
    len: c_size_t
    _fields_ = [
        ("data", POINTER(c_uint)),
        ("len", c_size_t),
    ]

class IntegrateFireState(Structure):
    voltage: c_double
    reset_potential: c_double
    threshold: c_double
    _fields_ = [
        ("voltage", c_double),
        ("reset_potential", c_double),
        ("threshold", c_double),
    ]

class LifState(Structure):
    voltage: c_double
    reset_potential: c_double
    threshold: c_double
    leak_rate: c_double
    input_gain: c_double
    _fields_ = [
        ("voltage", c_double),
        ("reset_potential", c_double),
        ("threshold", c_double),
        ("leak_rate", c_double),
        ("input_gain", c_double),
    ]

class IzhikevichState(Structure):
    voltage: c_double
    reset_potential: c_double
    threshold: c_double
    a_var: c_double
    b_var: c_double
    recovery_var: c_double
    d_var: c_double
    _fields_ = [
        ("voltage", c_double),
        ("reset_potential", c_double),
        ("threshold", c_double),
        ("a_var", c_double),
        ("b_var", c_double),
        ("recovery_var", c_double),
        ("d_var", c_double),
    ]

# --- ctypes setup ---
_lib.create_event_container.restype = c_void_p
_lib.destroy_event_container.argtypes = [c_void_p]
_lib.clear_events.argtypes = [c_void_p]
_lib.get_spike_count.argtypes = [c_void_p]; _lib.get_spike_count.restype = c_size_t
_lib.get_spike.argtypes = [c_void_p, c_size_t]; _lib.get_spike.restype = SpikeEvent
_lib.create_stimulus_container.restype = c_void_p
_lib.destroy_stimulus_container.argtypes = [c_void_p]
_lib.get_voltage.argtypes = [c_void_p, c_void_p, c_uint]; _lib.get_voltage.restype = c_double
_lib.get_threshold.argtypes = [c_void_p, c_void_p, c_uint]; _lib.get_threshold.restype = c_double
_lib.set_voltage.argtypes = [c_void_p, c_void_p, c_uint, c_double]
_lib.set_current_stimulus.argtypes = [c_void_p, c_uint, c_double]
_lib.step.argtypes = [c_void_p, c_void_p, c_void_p, c_void_p]; _lib.step.restype = c_void_p
_lib.create_network.restype = c_void_p
_lib.destroy_network.argtypes = [c_void_p]
_lib.serialize_network.argtypes = [c_void_p, c_bool]; _lib.serialize_network.restype = ByteBuffer
_lib.deserialize_network.argtypes = [POINTER(c_ubyte), c_size_t, c_bool]; _lib.deserialize_network.restype = c_void_p
_lib.get_neurons.argtypes = [c_void_p]; _lib.get_neurons.restype = NeuronBuffer
_lib.get_synapses.argtypes = [c_void_p]; _lib.get_synapses.restype = SynapseBuffer
_lib.add_synapse.argtypes = [c_void_p, c_uint, c_uint]; _lib.add_synapse.restype = c_uint
_lib.add_integrate_fire_neuron.argtypes = [c_void_p]; _lib.add_integrate_fire_neuron.restype = c_uint
_lib.add_lif_neuron.argtypes = [c_void_p]; _lib.add_lif_neuron.restype = c_uint
_lib.add_izhikevich_neuron.argtypes = [c_void_p]; _lib.add_izhikevich_neuron.restype = c_uint
_lib.network_remove_neuron.argtypes = [c_void_p, c_uint]
_lib.network_remove_synapse.argtypes = [c_void_p, c_uint]
_lib.get_neuron_model.argtypes = [c_void_p, c_uint]; _lib.get_neuron_model.restype = c_uint
_lib.network_has_neuron.argtypes = [c_void_p, c_uint]; _lib.network_has_neuron.restype = c_bool
_lib.network_has_synapse.argtypes = [c_void_p, c_uint]; _lib.network_has_synapse.restype = c_bool
_lib.serialize_runstate.argtypes = [c_void_p, c_bool]; _lib.serialize_runstate.restype = ByteBuffer
_lib.deserialize_runstate.argtypes = [POINTER(c_ubyte), c_size_t, c_bool]; _lib.deserialize_runstate.restype = c_void_p
_lib.create_runstate.restype = c_void_p
_lib.clone_runstate.argtypes = [c_void_p]; _lib.clone_runstate.restype = c_void_p
_lib.destroy_runstate.argtypes = [c_void_p]
_lib.set_neuron_refractory_period.argtypes = [c_void_p, c_uint, c_uint]
_lib.set_synapse_strength.argtypes = [c_void_p, c_uint, c_double]
_lib.set_synapse_conduction_time.argtypes = [c_void_p, c_uint, c_uint]
_lib.runstate_remove_neuron.argtypes = [c_void_p, c_uint]
_lib.clear_synapse_wheel.argtypes = [c_void_p]
_lib.get_integrate_fire_state.argtypes = [c_void_p, c_uint]; _lib.get_integrate_fire_state.restype = POINTER(IntegrateFireState)
_lib.get_lif_state.argtypes = [c_void_p, c_uint]; _lib.get_lif_state.restype = POINTER(LifState)
_lib.get_izhikevich_state.argtypes = [c_void_p, c_uint]; _lib.get_izhikevich_state.restype = POINTER(IzhikevichState)
_lib.set_integrate_fire_state.argtypes = [c_void_p, c_uint, IntegrateFireState]
_lib.set_lif_state.argtypes = [c_void_p, c_uint, LifState]
_lib.set_izhikevich_state.argtypes = [c_void_p, c_uint, IzhikevichState]
_lib.destroy_byte_buffer.argtypes = [ByteBuffer]
_lib.destroy_neuron_buffer.argtypes = [NeuronBuffer]
_lib.destroy_synapse_buffer.argtypes = [SynapseBuffer]

# --- Public API ---
def create_event_container() -> int: return _lib.create_event_container()
def destroy_event_container(container: int) -> None: _lib.destroy_event_container(container)
def clear_events(container: int) -> None: _lib.clear_events(container)
def get_spike_count(container: int) -> int: return _lib.get_spike_count(container)
def get_spike(container: int, index: int) -> SpikeEvent: return _lib.get_spike(container, index)
def create_stimulus_container() -> int: return _lib.create_stimulus_container()
def destroy_stimulus_container(container: int) -> None: _lib.destroy_stimulus_container(container)
def get_voltage(network: int, runstate: int, id: int) -> float: return _lib.get_voltage(network, runstate, id)
def get_threshold(network: int, runstate: int, id: int) -> float: return _lib.get_threshold(network, runstate, id)
def set_voltage(network: int, runstate: int, id: int, voltage: float) -> None: _lib.set_voltage(network, runstate, id, voltage)
def set_current_stimulus(stimuli: int, neuron_id: int, current: float) -> None: _lib.set_current_stimulus(stimuli, neuron_id, current)
def step(network: int, runstate: int, stimuli: int, events: int) -> int: return _lib.step(network, runstate, stimuli, events)
def create_network() -> int: return _lib.create_network()
def destroy_network(network: int) -> None: _lib.destroy_network(network)
def serialize_network(network: int, json: bool) -> ByteBuffer: return _lib.serialize_network(network, json)
def deserialize_network(data: POINTER(c_ubyte), length: int, json: bool) -> int: return _lib.deserialize_network(data, length, json)
def get_neurons(network: int) -> NeuronBuffer: return _lib.get_neurons(network)
def get_synapses(network: int) -> SynapseBuffer: return _lib.get_synapses(network)
def add_synapse(network: int, source: int, target: int) -> int: return _lib.add_synapse(network, source, target)
def add_integrate_fire_neuron(network: int) -> int: return _lib.add_integrate_fire_neuron(network)
def add_lif_neuron(network: int) -> int: return _lib.add_lif_neuron(network)
def add_izhikevich_neuron(network: int) -> int: return _lib.add_izhikevich_neuron(network)
def network_remove_neuron(network: int, id: int) -> None: _lib.network_remove_neuron(network, id)
def network_remove_synapse(network: int, id: int) -> None: _lib.network_remove_synapse(network, id)
def get_neuron_model(network: int, id: int) -> NeuronModelKind: return NeuronModelKind(_lib.get_neuron_model(network, id))
def network_has_neuron(network: int, id: int) -> bool: return _lib.network_has_neuron(network, id)
def network_has_synapse(network: int, id: int) -> bool: return _lib.network_has_synapse(network, id)
def serialize_runstate(runstate: int, json: bool) -> ByteBuffer: return _lib.serialize_runstate(runstate, json)
def deserialize_runstate(data: POINTER(c_ubyte), length: int, json: bool) -> int: return _lib.deserialize_runstate(data, length, json)
def create_runstate() -> int: return _lib.create_runstate()
def clone_runstate(runstate: int) -> int: return _lib.clone_runstate(runstate)
def destroy_runstate(runstate: int) -> None: _lib.destroy_runstate(runstate)
def set_neuron_refractory_period(runstate: int, id: int, refractory_period: int) -> None: _lib.set_neuron_refractory_period(runstate, id, refractory_period)
def set_synapse_strength(runstate: int, id: int, strength: float) -> None: _lib.set_synapse_strength(runstate, id, strength)
def set_synapse_conduction_time(runstate: int, id: int, conduction_time: int) -> None: _lib.set_synapse_conduction_time(runstate, id, conduction_time)
def runstate_remove_neuron(runstate: int, id: int) -> None: _lib.runstate_remove_neuron(runstate, id)
def clear_synapse_wheel(runstate: int) -> None: _lib.clear_synapse_wheel(runstate)
def get_integrate_fire_state(runstate: int, id: int) -> POINTER(IntegrateFireState): return _lib.get_integrate_fire_state(runstate, id)
def get_lif_state(runstate: int, id: int) -> POINTER(LifState): return _lib.get_lif_state(runstate, id)
def get_izhikevich_state(runstate: int, id: int) -> POINTER(IzhikevichState): return _lib.get_izhikevich_state(runstate, id)
def set_integrate_fire_state(runstate: int, neuron_id: int, state: IntegrateFireState) -> None: _lib.set_integrate_fire_state(runstate, neuron_id, state)
def set_lif_state(runstate: int, neuron_id: int, state: LifState) -> None: _lib.set_lif_state(runstate, neuron_id, state)
def set_izhikevich_state(runstate: int, neuron_id: int, state: IzhikevichState) -> None: _lib.set_izhikevich_state(runstate, neuron_id, state)
def destroy_byte_buffer(buffer: ByteBuffer) -> None: _lib.destroy_byte_buffer(buffer)
def destroy_neuron_buffer(buffer: NeuronBuffer) -> None: _lib.destroy_neuron_buffer(buffer)
def destroy_synapse_buffer(buffer: SynapseBuffer) -> None: _lib.destroy_synapse_buffer(buffer)

# --- Iterators ---
def iter_neurons(network: int) -> list[int]:
    buf = _lib.get_neurons(network)
    result = [buf.data[i] for i in range(buf.len)]
    _lib.destroy_neuron_buffer(buf)
    return result

def iter_synapses(network: int) -> list[int]:
    buf = _lib.get_synapses(network)
    result = [buf.data[i] for i in range(buf.len)]
    _lib.destroy_synapse_buffer(buf)
    return result

