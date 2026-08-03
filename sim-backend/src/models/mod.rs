use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub enum NeuronModelKind {
    IntegrateFire,
    LIF,
    Izhikevich,
}
