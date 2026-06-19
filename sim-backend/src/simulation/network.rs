use generational_arena::Arena;
use slotmap::SlotMap;

use crate::common::{Color, Position, Rect, Voltage};
use crate::models::NeuronModelKind;
use crate::r#gen::generate_neuron_name;
use crate::simulation::execution::IntegrateFireParams;
use crate::simulation::id::StateIndex;
use crate::simulation::{
    execution::Executor,
    id::{GroupId, NeuronId, SynapseId},
};
use crate::util::variant_eq;

#[derive(Clone, PartialEq)]
pub struct Neuron {
    pub id: NeuronId,
    pub name: String,
    pub model: NeuronModelKind,
    pub state_index: StateIndex,
    pub position: Position,
    pub parent: Option<GroupId>,
    pub outgoing: Vec<SynapseId>,
}

#[derive(Clone, PartialEq)]
pub struct Synapse {
    pub weight: f64,
    pub target: NeuronId,
}

#[derive(Clone, PartialEq)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub parent: Option<GroupId>,
    pub neurons: Vec<NeuronId>,
    pub groups: Vec<GroupId>,
    pub rect: Rect,
    pub color: Color,
}

#[derive(Clone, Default)]
pub struct Network {
    pub executor: Executor,
    pub neurons: Arena<Neuron>,
    pub synapses: SlotMap<SynapseId, Synapse>,
    pub groups: Arena<Group>,
}

impl Network {
    pub fn get_voltage(&self, id: NeuronId) -> Voltage {
        let neuron = self.neurons.get(id).unwrap();
        match &neuron.model {
            NeuronModelKind::IntegrateFire(_) => self.executor.integrate_fire.voltage[neuron.state_index as usize],
            NeuronModelKind::LIF(_) => 0.0,
            NeuronModelKind::Izhikevich(_) => 0.0,
        }
    }
    pub fn get_threshold(&self, id: NeuronId) -> Voltage {
        let neuron = self.neurons.get(id).unwrap();
        match &neuron.model {
            NeuronModelKind::IntegrateFire(_) => self.executor.integrate_fire.threshold[neuron.state_index as usize],
            NeuronModelKind::LIF(_) => 0.0,
            NeuronModelKind::Izhikevich(_) => 0.0,
        }
    }

    pub fn add_integrate_fire_neuron(
        &mut self,
        params: IntegrateFireParams,
        position: Position,
        parent: GroupId,
    ) -> NeuronId {
        let idx = self.neurons.insert_with(|idx| {
            let state_index = self.executor.integrate_fire.voltage.len() as StateIndex;
            self.executor.integrate_fire.state_to_meta.push(idx);

            Neuron {
                id: idx,
                name: generate_neuron_name(),
                model: NeuronModelKind::integrate_fire(),
                state_index: state_index,
                position,
                parent: Some(parent),
            }
        });

        self.executor.integrate_fire.voltage.push(params.reset_potential);
        self.executor
            .integrate_fire
            .reset_potential
            .push(params.reset_potential);
        self.executor.integrate_fire.strength.push(params.strength);
        self.executor.integrate_fire.threshold.push(params.threshold);
        self.executor
            .integrate_fire
            .minimum_voltage
            .push(params.minimum_voltage);
        self.executor
            .integrate_fire
            .maximum_voltage
            .push(params.maximum_voltage);

        idx
    }

    pub fn remove_neuron(&mut self, id: NeuronId) {
        let model = self.neurons[id].model.clone();
        let state_index = self.neurons[id].state_index as usize;

        if variant_eq(&model, &NeuronModelKind::integrate_fire()) {
            let moved_index = self.executor.integrate_fire.state_to_meta.pop().unwrap();
            if self.executor.integrate_fire.state_to_meta.len() > 0 {
                let moved = self.neurons.get_mut(moved_index).unwrap();

                self.executor.integrate_fire.state_to_meta[state_index] = moved_index;
                moved.state_index = state_index as StateIndex;
            }

            self.executor.integrate_fire.voltage.swap_remove(state_index);
            self.executor.integrate_fire.reset_potential.swap_remove(state_index);
            self.executor.integrate_fire.strength.swap_remove(state_index);
            self.executor.integrate_fire.threshold.swap_remove(state_index);
            self.executor.integrate_fire.minimum_voltage.swap_remove(state_index);
            self.executor.integrate_fire.maximum_voltage.swap_remove(state_index);
        }

        self.neurons.remove(id);
        self.synapses.retain(|_, e| e.source != id && e.target != id);

        for (_, g) in self.groups.iter_mut() {
            g.neurons.retain(|&cid| cid != id);
        }
    }

    pub fn remove_group(&mut self, id: GroupId) {
        let mut groups_to_delete = vec![id];
        let mut neurons_to_delete = vec![];
        let mut idx = 0;

        while idx < groups_to_delete.len() {
            let current_group_id = groups_to_delete[idx];
            let group = self.groups.get(current_group_id).unwrap();

            neurons_to_delete.extend(group.neurons.clone());
            groups_to_delete.extend(group.groups.iter().map(|id| id.clone()));

            idx += 1;
        }

        for neuron_id in &neurons_to_delete {
            self.remove_neuron(*neuron_id);
        }

        self.groups.retain(|id, _| !groups_to_delete.contains(&id));

        for (_, group) in self.groups.iter_mut() {
            group
                .groups
                .retain(|&sub_group_id| !groups_to_delete.contains(&sub_group_id));
        }
    }
}
