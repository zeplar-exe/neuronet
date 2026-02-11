use crate::models::{NeuronModelKind};
use crate::settings::{AppSettings, SimulationSettings};
use crate::util::variant_eq;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ViewType {
    EDIT,
    EXECUTE,
}

type NeuronId = i64;
type GroupId = i64;
type EdgeId = i64;
type StateIndex = i64;
type Position = (f64, f64);
type Voltage = f64;

#[derive(Clone, PartialEq)]
pub struct Neuron {
    pub id: NeuronId,
    pub model: NeuronModelKind,
    pub state_index: StateIndex,
    pub position: Position,
    pub parent: GroupId,
}

#[derive(Clone, Default, PartialEq)]
pub struct Edge {
    pub id: EdgeId,
    pub source: NeuronId,
    pub target: NeuronId
}

#[derive(Clone, Default, PartialEq)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub parent: GroupId,
    pub children: Vec<i64>,
}

#[derive(Clone, Default)]
pub struct IntegrateFireState {
    pub voltage: Vec<Voltage>,
    pub reset_potential: Vec<Voltage>,
    pub strength: Vec<Voltage>,
    pub threshold: Vec<Voltage>,
    pub minimum_voltage: Vec<Voltage>,
    pub maximum_voltage: Vec<Voltage>,
}

#[derive(Clone, Default)]
pub struct Executor {
    pub tick: i64,
    pub integrate_fire: IntegrateFireState,
}

impl Executor {
    pub fn step(&mut self, dt: f64, neurons: &[Neuron], edges: &[Edge]) {

    }
}

#[derive(Clone, Default)]
pub struct Network {
    pub executor: Executor,
    pub neurons: Vec<Neuron>,
    pub edges: Vec<Edge>,
    pub groups: Vec<Group>,
}

pub struct IntegrateFireParams {
    pub reset_potential: Voltage,
    pub strength: Voltage,
    pub threshold: Voltage,
    pub minimum_voltage: Voltage,
    pub maximum_voltage: Voltage,
}

impl Default for IntegrateFireParams {
    fn default() -> Self { Self {
        strength: 30.0 as Voltage,
        reset_potential: -70.0 as Voltage,
        threshold: -55.0 as Voltage,
        minimum_voltage: -100.0 as Voltage,
        maximum_voltage: 50.0 as Voltage,
    } }
}

impl Network {
    pub fn get_voltage(&self, id: NeuronId) -> Voltage {
        match &self.neurons[id as usize].model {
            NeuronModelKind::IntegrateFire(_) => {
                self.executor.integrate_fire.voltage[id as usize]
            }
            NeuronModelKind::LIF(_) => {0.0}
            NeuronModelKind::Izhikevich(_) => {0.0}
        }
    }

    pub fn add_integrate_fire_neuron(&mut self, params: IntegrateFireParams, position: Position, parent: GroupId) -> NeuronId {
        let neuron = Neuron {
            id: self.neurons.len() as NeuronId,
            model: NeuronModelKind::integrate_fire(),
            state_index: self.executor.integrate_fire.voltage.len() as StateIndex,
            position,
            parent,
        };
        let id = neuron.id;

        self.neurons.push(neuron);
        self.executor.integrate_fire.voltage.push(params.reset_potential);
        self.executor.integrate_fire.reset_potential.push(params.reset_potential);
        self.executor.integrate_fire.strength.push(params.strength);
        self.executor.integrate_fire.threshold.push(params.threshold);
        self.executor.integrate_fire.minimum_voltage.push(params.minimum_voltage);
        self.executor.integrate_fire.maximum_voltage.push(params.maximum_voltage);

        id
    }

    pub fn remove_neuron(&mut self, id: NeuronId) {
        let model = self.neurons[id as usize].model.clone();
        let state_index = self.neurons[id as usize].state_index;

        if variant_eq(&model, &NeuronModelKind::integrate_fire()) {
            self.executor.integrate_fire.voltage.remove(id as usize);
            self.executor.integrate_fire.reset_potential.remove(id as usize);
            self.executor.integrate_fire.strength.remove(id as usize);
            self.executor.integrate_fire.threshold.remove(id as usize);
            self.executor.integrate_fire.minimum_voltage.remove(id as usize);
            self.executor.integrate_fire.maximum_voltage.remove(id as usize);
        }

        self.neurons.remove(id as usize);
        self.edges.retain(|e| e.source != id && e.target != id);

        for g in self.groups.iter_mut() {
            g.children.retain(|&cid| cid != id);
        }

        for i in (id as usize)..self.neurons.len() {
            self.neurons[i].id -= 1;

            if self.neurons[i].model == model && self.neurons[i].state_index > state_index {
                self.neurons[i].state_index -= 1;
            }
        }
        
        for e in self.edges.iter_mut() {
            if e.source > id { e.source -= 1; }
            if e.target > id { e.target -= 1; }
        }
    }
}

#[derive(Clone)]
pub struct ViewState {
    pub selected_tool: &'static str,
}

impl Default for ViewState {
    fn default() -> Self {
        Self { selected_tool: "add" }
    }
}

#[derive(Clone)]
pub struct AppStore {
    pub current_view: ViewType,
    pub network: Network,
    pub selected: Vec<NeuronId>,
    pub selected_groups: Vec<GroupId>,
    pub edit_view: ViewState,
    pub execute_view: ViewState,
    pub app_settings: AppSettings,
    pub sim_settings: SimulationSettings
}

impl Default for AppStore {
    fn default() -> Self {
        Self {
            current_view: ViewType::EDIT,
            network: Network::default(),
            selected: vec![],
            selected_groups: vec![],
            edit_view: ViewState::default(),
            execute_view: ViewState { selected_tool: "select" },
            app_settings: AppSettings::default(),
            sim_settings: SimulationSettings::default(),
        }
    }
}

impl AppStore {
    pub fn push_neuron(&mut self, node: Neuron) { self.network.neurons.push(node); }
    pub fn get_selected_neurons(&self) -> Vec<&Neuron> {
        self.network.neurons.iter().filter(|n| self.selected.contains(&n.id)).collect()
    }
    pub fn clear_selection(&mut self) { self.selected.clear(); self.selected_groups.clear(); }
    pub fn select_only(&mut self, id: NeuronId) { self.selected = vec![id]; self.selected_groups.clear(); }
    pub fn select_all(&mut self) {
        self.selected = self.network.neurons.iter().map(|n| n.id).collect();
    }
    pub fn toggle_selected(&mut self, id: NeuronId) {
        if let Some(idx) = self.selected.iter().position(|x| *x == id) {
            self.selected.remove(idx);
        } else {
            self.selected.push(id);
        }
    }
    pub fn ensure_selected(&mut self, id: NeuronId) {
        if !self.selected.iter().any(|x| *x == id) {
            self.selected.push(id);
        }
    }
    fn remove_child_from_parents(&mut self, child_id: NeuronId) {
        for g in self.network.groups.iter_mut() {
            if let Some(pos) = g.children.iter().position(|&cid| cid == child_id) {
                g.children.remove(pos);
            }
        }
    }
    pub fn clear_group_selection(&mut self) { self.selected_groups.clear(); }
    pub fn select_group_only(&mut self, id: NeuronId) { self.selected_groups = vec![id]; self.selected.clear(); }
    pub fn toggle_group_selected(&mut self, id: GroupId) {
        if let Some(idx) = self.selected_groups.iter().position(|x| *x == id) {
            self.selected_groups.remove(idx);
        } else {
            self.selected_groups.push(id);
        }
    }
    pub fn ensure_group_selected(&mut self, id: GroupId) {
        if !self.selected_groups.iter().any(|x| *x == id) {
            self.selected_groups.push(id);
        }
    }
    pub fn has_edge(&self, source: NeuronId, target: NeuronId) -> bool {
        self.network.edges.iter().any(|e| e.source == source && e.target == target)
    }
    pub fn push_edge(&mut self, source: NeuronId, target: NeuronId) -> EdgeId {
        if source == target {
            return 0;
        }
        if let Some(existing) = self.network.edges.iter().find(|e| e.source == source && e.target == target) {
            return existing.id;
        }
        
        let next_id = self.network.edges.iter().map(|e| e.id).max().unwrap_or(0) + 1;
        self.network.edges.push(Edge { id: next_id, source, target });
        next_id
    }
    pub fn set_tool(&mut self, view: ViewType, tool: &'static str) {
        match view {
            ViewType::EDIT => self.edit_view.selected_tool = tool,
            ViewType::EXECUTE => self.execute_view.selected_tool = tool,
        }
    }
    pub fn view_state(&self, view: ViewType) -> &ViewState {
        match view {
            ViewType::EDIT => &self.edit_view,
            ViewType::EXECUTE => &self.execute_view,
        }
    }

    pub fn group_selected(&mut self, name: Option<String>) -> Option<GroupId> {
        let any = !self.selected.is_empty() || !self.selected_groups.is_empty();
        if !any { return None; }
        
        let next_id = self.network.groups.iter().map(|g| g.id).max().unwrap_or(0) + 1;
        let group_name = name.unwrap_or_else(|| format!("Group {}", next_id));
        
        self.network.groups.push(Group { id: next_id, name: group_name, parent: 0, children: vec![] });

        let nodes_to_move = self.selected.clone();
        let groups_to_move = self.selected_groups.clone();

        for nid in nodes_to_move.iter() { self.remove_child_from_parents(*nid); }
        for gid in groups_to_move.iter() { if *gid != next_id { self.remove_child_from_parents(*gid); } }
        
        for n in self.network.neurons.iter_mut() {
            if nodes_to_move.contains(&n.id) { n.parent = next_id; }
        }
        
        for g in self.network.groups.iter_mut() {
            if g.id != next_id && groups_to_move.contains(&g.id) { g.parent = next_id; }
        }
        
        if let Some(parent) = self.network.groups.iter_mut().find(|pg| pg.id == next_id) {
            for nid in nodes_to_move.iter() { parent.children.push(*nid); }
            for gid in groups_to_move.iter() { if *gid != next_id { parent.children.push(*gid); } }
        }

        self.selected.clear();
        self.selected_groups = vec![next_id];

        Some(next_id)
    }

    pub fn delete_neuron(&mut self, id: NeuronId) {
        self.network.edges.retain(|e| e.source != id && e.target != id);
        
        for g in self.network.groups.iter_mut() {
            g.children.retain(|&cid| cid != id);
        }
        
        self.network.neurons.retain(|n| n.id != id);
        self.selected.retain(|&sid| sid != id);
    }

    pub fn delete_edge(&mut self, id: EdgeId) {
        self.network.edges.retain(|e| e.id != id);
    }

    pub fn delete_group_recursive(&mut self, id: GroupId) {
        let mut groups_to_delete: Vec<i64> = vec![id];
        let mut nodes_to_delete: Vec<i64> = vec![];
        let mut idx = 0;

        while idx < groups_to_delete.len() {
            let gid = groups_to_delete[idx];

            for g in self.network.groups.iter() {
                if g.parent == gid && g.id != id {
                    if !groups_to_delete.contains(&g.id) {
                        groups_to_delete.push(g.id);
                    }
                }
            }
            // Children nodes
            for n in self.network.neurons.iter() {
                if n.parent == gid {
                    nodes_to_delete.push(n.id);
                }
            }
            idx += 1;
        }

        for nid in nodes_to_delete.iter() {
            self.delete_neuron(*nid);
        }

        for pg in self.network.groups.iter_mut() {
            pg.children.retain(|cid| !groups_to_delete.contains(cid));
        }
        // Then remove the groups themselves
        self.network.groups.retain(|g| !groups_to_delete.contains(&g.id));
        // Clear from selection
        self.selected_groups.retain(|gid| !groups_to_delete.contains(gid));
    }

    pub fn get_next_state_index(&self, model: &NeuronModelKind) -> StateIndex {
        if variant_eq(model, &NeuronModelKind::integrate_fire()) {
            self.network.executor.integrate_fire.voltage.len() as StateIndex
        } else { -1 }
    }
}