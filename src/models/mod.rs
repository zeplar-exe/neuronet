use std::hash::{Hash, Hasher};
use std::ops::Deref;
use crate::models::integrate_fire::IntegrateFireModel;

pub mod integrate_fire;

pub trait NeuronModel {
    fn name(&self) -> &'static str;
    fn update(&mut self, dt: f64) -> f64;
    fn excite(&mut self, amount: f64);
    fn inhibit(&mut self, amount: f64);
    fn reset(&mut self);
}

#[derive(Clone, PartialEq)]
pub enum NeuronModelKind {
    IntegrateFire(IntegrateFireModel),
    // LIF(LifModel),
    // Izhikevich(IzhModel),
}

impl Default for NeuronModelKind {
    fn default() -> Self { Self::IntegrateFire(IntegrateFireModel::default()) }
}

impl NeuronModel for NeuronModelKind {
    fn name(&self) -> &'static str {
        match self {
            Self::IntegrateFire(m) => m.name(),
        }
    }
    fn update(&mut self, dt: f64) -> f64 {
        match self {
            Self::IntegrateFire(m) => m.update(dt),
        }
    }
    fn excite(&mut self, amount: f64) {
        match self {
            Self::IntegrateFire(m) => m.excite(amount),
        }
    }
    fn inhibit(&mut self, amount: f64) {
        match self {
            Self::IntegrateFire(m) => m.inhibit(amount),
        }
    }
    fn reset(&mut self) {
        match self {
            Self::IntegrateFire(m) => m.reset(),
        }
    }
}