use crate::models::NeuronModel;

#[derive(Clone, PartialEq)]
pub struct IntegrateFireState {
    pub excitation: f64,
}

impl Default for IntegrateFireState {
    fn default() -> Self { Self { excitation: -70.0 } }
}

#[derive(Clone, PartialEq)]
pub struct IntegrateFireConfig {
    pub strength: f64,
    pub reset_potential: f64,
    pub threshold: f64,
    pub minimum_voltage: f64,
    pub maximum_voltage: f64,
}

impl Default for IntegrateFireConfig {
    fn default() -> Self { Self {
        strength: 30.0,
        reset_potential: -70.0,
        threshold: -55.0,
        minimum_voltage: -100.0,
        maximum_voltage: 50.0,
    } }
}

#[derive(PartialEq, Default, Clone)]
pub struct IntegrateFireModel {
    pub state: IntegrateFireState,
    pub config: IntegrateFireConfig
}

impl NeuronModel for IntegrateFireModel {
    fn name(&self) -> &'static str { "Integrate and Fire" }
    fn update(&mut self, dt: f64) -> f64 {
        let _ = dt; // dt reserved for future use
        if self.state.excitation >= self.config.threshold {
            self.reset();
            self.config.strength
        } else { 0.0 }
    }
    fn excite(&mut self, amount: f64) {
        let next = self.state.excitation + amount;
        let min_v = self.config.minimum_voltage;
        let max_v = self.config.maximum_voltage;
        self.state.excitation = next.clamp(min_v, max_v);
    }
    fn inhibit(&mut self, amount: f64) {
        let next = self.state.excitation - amount;
        let min_v = self.config.minimum_voltage;
        let max_v = self.config.maximum_voltage;
        self.state.excitation = next.clamp(min_v, max_v);
    }
    fn reset(&mut self) {
        self.state.excitation = self.config.reset_potential;
    }
}