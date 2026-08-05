use crate::simulation::network::{NeuronId, Voltage};

pub struct EventContainer {
    pub spikes: Vec<SpikeEvent>,
}

#[repr(C)]
pub struct SpikeEvent {
    pub neuron_id: NeuronId,
    pub timestamp: i32,
    pub voltage: Voltage,
}

#[no_mangle]
pub extern "C" fn create_event_container() -> *mut EventContainer {
    ffi_catch_ptr!(Box::into_raw(Box::new(EventContainer { spikes: Vec::new() })))
}

#[no_mangle]
pub extern "C" fn destroy_event_container(container: *mut EventContainer) {
    ffi_catch_void!(unsafe { drop(Box::from_raw(container)) })
}

#[no_mangle]
pub extern "C" fn clear_events(container: &mut EventContainer) {
    ffi_catch_void!(container.spikes.clear())
}

#[no_mangle]
pub extern "C" fn get_spike_count(container: *const EventContainer) -> usize {
    ffi_catch_num!(unsafe { (*container).spikes.len() })
}

#[no_mangle]
pub extern "C" fn get_spike(container: *const EventContainer, index: usize) -> SpikeEvent {
    ffi_catch_struct!(
        SpikeEvent {
            neuron_id: 0,
            timestamp: 0,
            voltage: 0.0
        },
        unsafe {
            let container = &*container;
            let s = &container.spikes[index];
            SpikeEvent {
                neuron_id: s.neuron_id,
                timestamp: s.timestamp,
                voltage: s.voltage,
            }
        }
    )
}
