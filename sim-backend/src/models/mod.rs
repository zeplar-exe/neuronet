#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum NeuronModelKind {
    IntegrateFire,
    LIF,
    Izhikevich,
}
