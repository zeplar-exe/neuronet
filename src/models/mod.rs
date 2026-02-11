#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelMeta {
    pub name: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NeuronModelKind {
    IntegrateFire(ModelMeta),
    LIF(ModelMeta),
    Izhikevich(ModelMeta),
}

impl Default for NeuronModelKind {
    fn default() -> Self { Self::integrate_fire() }
}

impl NeuronModelKind {
    pub const fn integrate_fire() -> Self { Self::IntegrateFire(ModelMeta { name: "Integrate & Fire" }) }
    pub const fn lif() -> Self { Self::LIF(ModelMeta { name: "LIF" }) }
    pub const fn izhikevich() -> Self { Self::Izhikevich(ModelMeta { name: "Izhikevich" }) }

    pub fn name(&self) -> &'static str {
        match self {
            NeuronModelKind::IntegrateFire(m)
            | NeuronModelKind::LIF(m)
            | NeuronModelKind::Izhikevich(m) => m.name,
        }
    }

    pub fn is_integrate_fire(&self) -> bool { matches!(self, NeuronModelKind::IntegrateFire(_)) }
}
