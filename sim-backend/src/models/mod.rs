#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NeuronModelKind {
    IntegrateFire,
    LIF,
    Izhikevich,
}
