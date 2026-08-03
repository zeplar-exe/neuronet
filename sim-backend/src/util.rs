use crate::simulation::network::{NeuronId, SynapseId};

#[repr(C)]
pub struct ByteBuffer {
    pub data: *mut u8,
    pub len: usize,
}

#[repr(C)]
pub struct NeuronBuffer {
    pub data: *const NeuronId,
    pub len: usize,
}

#[repr(C)]
pub struct SynapseBuffer {
    pub data: *const SynapseId,
    pub len: usize,
}

pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

#[no_mangle]
pub extern "C" fn destroy_byte_buffer(buffer: ByteBuffer) {
    unsafe {
        if !buffer.data.is_null() {
            drop(Vec::from_raw_parts(buffer.data, buffer.len, buffer.len));
        }
    }
}

#[no_mangle]
pub extern "C" fn destroy_neuron_buffer(buffer: NeuronBuffer) {
    unsafe {
        if !buffer.data.is_null() {
            drop(Box::from_raw(buffer.data as *mut u8));
        }
    }
}

#[no_mangle]
pub extern "C" fn destroy_synapse_buffer(buffer: SynapseBuffer) {
    unsafe {
        if !buffer.data.is_null() {
            drop(Box::from_raw(buffer.data as *mut u8));
        }
    }
}
