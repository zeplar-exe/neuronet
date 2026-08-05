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
    ffi_catch_void!(unsafe {
        if !buffer.data.is_null() {
            drop(Vec::from_raw_parts(buffer.data, buffer.len, buffer.len));
        }
    })
}

#[no_mangle]
pub extern "C" fn destroy_neuron_buffer(buffer: NeuronBuffer) {
    ffi_catch_void!(unsafe {
        if !buffer.data.is_null() {
            drop(Vec::from_raw_parts(buffer.data as *mut NeuronId, buffer.len, buffer.len));
        }
    })
}

#[no_mangle]
pub extern "C" fn destroy_synapse_buffer(buffer: SynapseBuffer) {
    ffi_catch_void!(unsafe {
        if !buffer.data.is_null() {
            drop(Vec::from_raw_parts(buffer.data as *mut SynapseId, buffer.len, buffer.len));
        }
    })
}
