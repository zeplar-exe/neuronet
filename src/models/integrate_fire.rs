use dioxus::prelude::use_signal;
use dioxus::signals::{ReadableExt, Signal, WritableExt};
use crate::models::NeuronModel;

#[derive(PartialEq, Clone)]
pub struct IntegrateFireModel {
    pub excitation: Signal<f64>,
    pub strength: Signal<f64>,
    pub reset_potential: Signal<f64>,
    pub threshold: Signal<f64>,
    pub minimum_voltage: Signal<f64>,
    pub maximum_voltage: Signal<f64>,
}

impl Default for IntegrateFireModel {
    fn default() -> Self { Self {
        excitation: use_signal(|| -70.0),
        strength: use_signal(|| 30.0),
        reset_potential: use_signal(|| -70.0),
        threshold: use_signal(|| -55.0),
        minimum_voltage: use_signal(|| -100.0),
        maximum_voltage: use_signal(|| 50.0),
    } }
}

impl NeuronModel for IntegrateFireModel {
    fn name(&self) -> &'static str { "Integrate and Fire" }
    fn update(&mut self, dt: f64) -> f64 {
        if *self.excitation.read() >= *self.threshold.read() {
            self.reset();
            *self.strength.read()
        } else { 0.0 }
    }
    fn excite(&mut self, amount: f64) {
        let next = *self.excitation.read() + amount;
        let min_v = *self.minimum_voltage.read();
        let max_v = *self.maximum_voltage.read();
        self.excitation.set(next.clamp(min_v, max_v));
    }
    fn inhibit(&mut self, amount: f64) {
        let next = *self.excitation.read() - amount;
        let min_v = *self.minimum_voltage.read();
        let max_v = *self.maximum_voltage.read();
        self.excitation.set(next.clamp(min_v, max_v));
    }
    fn reset(&mut self) {
        self.excitation.set(*self.reset_potential.read());
    }
}