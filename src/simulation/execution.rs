use generational_arena::Index;

use crate::{
    common::Voltage,
    simulation::{
        id::NeuronId,
        network::{Network, Neuron},
    },
};

pub struct IntegrateFireParams {
    pub reset_potential: Voltage,
    pub strength: Voltage,
    pub threshold: Voltage,
    pub minimum_voltage: Voltage,
    pub maximum_voltage: Voltage,
}

#[derive(Clone, Default)]
pub struct IntegrateFireState {
    pub voltage: Vec<Voltage>,
    pub reset_potential: Vec<Voltage>,
    pub strength: Vec<Voltage>,
    pub threshold: Vec<Voltage>,
    pub minimum_voltage: Vec<Voltage>,
    pub maximum_voltage: Vec<Voltage>,
    pub leak_constant: Vec<Voltage>,

    pub state_to_meta: Vec<Index>,
}

impl Default for IntegrateFireParams {
    fn default() -> Self {
        Self {
            strength: 30.0 as Voltage,
            reset_potential: -70.0 as Voltage,
            threshold: -55.0 as Voltage,
            minimum_voltage: -100.0 as Voltage,
            maximum_voltage: 50.0 as Voltage,
        }
    }
}

#[derive(Clone, Default)]
pub struct Executor {
    pub tick: i64,
    pub integrate_fire: IntegrateFireState,
}

impl Executor {
    pub fn step(&mut self, dt: f64, network: &Network, neurons: &[Neuron]) {
        let mut fired: Vec<NeuronId> = vec![];

        for neuron in neurons.iter() {
            let id = neuron.id;
            let v = network.get_voltage(id);
            let thresh = network.get_threshold(id);
            if (v > thresh) {
                fired.push(id);
            }
        }
    }
}
