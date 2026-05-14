use std::collections::HashSet;

use generational_arena::{Arena, Index};

use crate::gen::generate_neuron_name;
use crate::models::NeuronModelKind;
use crate::settings::{AppSettings, SimulationSettings};
use crate::util::variant_eq;

#[derive(Clone, Debug, PartialEq)]
pub enum ContextMenuTarget {
    Neuron(NeuronId),
    Group(GroupId),
    Canvas,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenuState {
    pub x: f64,
    pub y: f64,
    pub target: ContextMenuTarget,
    pub name: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ViewType {
    EDIT,
    EXECUTE,
}

pub type NeuronId = Index;
pub type GroupId = Index;
pub type SynapseId = Index;
pub type StateIndex = usize;
pub type Position = (f64, f64);
pub type Voltage = f64;

#[derive(Clone, PartialEq)]
pub struct Neuron {
    pub id: NeuronId,
    pub name: String,
    pub model: NeuronModelKind,
    pub state_index: StateIndex,
    pub position: Position,
    pub parent: Option<GroupId>,
}

#[derive(Clone, PartialEq)]
pub struct Synapse {
    pub id: SynapseId,
    pub source: NeuronId,
    pub target: NeuronId,
}

#[derive(Clone, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Default for Color {
    fn default() -> Self {
        Color {
            red: 255u8,
            green: 255u8,
            blue: 255u8,
            alpha: 255,
        }
    }
}

impl Color {
    pub fn hex(&self) -> String {
        format!("#{:02X?}", [self.red, self.green, self.blue, self.alpha])
    }
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Rect {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
}

impl Rect {
    pub fn from_corners(top_left: Position, bottom_right: Position) -> Rect {
        Rect {
            top: top_left.1,
            left: top_left.0,
            bottom: bottom_right.1,
            right: bottom_right.0,
        }
    }

    pub fn from_dimensions(position: Position, size: Position) -> Rect {
        Rect {
            top: position.1,
            left: position.0,
            bottom: position.1 + size.1,
            right: position.0 + size.0,
        }
    }

    pub fn from_dimensions_f(left: f64, top: f64, width: f64, height: f64) -> Rect {
        Rect {
            left,
            top,
            bottom: top + height,
            right: left + width,
        }
    }

    pub fn area(&self) -> f64 {
        (self.right - self.left) * (self.bottom - self.top)
    }

    pub fn width(&self) -> f64 {
        self.right - self.left
    }

    pub fn height(&self) -> f64 {
        self.bottom - self.top
    }

    pub fn as_tuple(&self) -> (f64, f64, f64, f64) {
        (self.top, self.left, self.bottom, self.right)
    }

    pub fn as_tuple_wh(&self) -> (f64, f64, f64, f64) {
        (self.left, self.top, self.width(), self.height())
    }
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

#[derive(Clone, Default)]
pub struct Executor {
    pub tick: i64,
    pub integrate_fire: IntegrateFireState,
}

impl Executor {
    pub fn step(&mut self, dt: f64, neurons: &[Neuron], edges: &[Synapse]) {}
}

#[derive(Clone, Default)]
pub struct Network {
    pub executor: Executor,
    pub neurons: Arena<Neuron>,
    pub synapses: Arena<Synapse>,
    pub groups: Arena<Group>,
}

pub struct IntegrateFireParams {
    pub reset_potential: Voltage,
    pub strength: Voltage,
    pub threshold: Voltage,
    pub minimum_voltage: Voltage,
    pub maximum_voltage: Voltage,
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

impl Network {
    pub fn get_voltage(&self, id: NeuronId) -> Voltage {
        let neuron = self.neurons.get(id).unwrap();
        match &neuron.model {
            NeuronModelKind::IntegrateFire(_) => self.executor.integrate_fire.voltage[neuron.state_index as usize],
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
pub struct PanAnim {
    pub start_offset: Position,
    pub target_offset: Position,
    pub start_ms: f64,
    pub duration_ms: f64,
}

impl PanAnim {
    pub fn new(start_offset: Position, target_offset: Position, duration_ms: f64, start_ms: f64) -> Self {
        Self {
            start_offset,
            target_offset,
            start_ms,
            duration_ms,
        }
    }
}

#[derive(Clone)]
pub struct AppStore {
    pub current_view: ViewType,
    pub network: Network,
    pub selected: HashSet<NeuronId>,
    pub selected_groups: HashSet<GroupId>,
    pub edit_view: ViewState,
    pub execute_view: ViewState,
    pub canvas_offset: Position,
    pub canvas_zoom: f64,
    pub pan_anim: Option<PanAnim>,
    pub app_time_ms: f64,
    pub app_settings: AppSettings,
    pub sim_settings: SimulationSettings,
    pub context_menu: Option<ContextMenuState>,
}

impl Default for AppStore {
    fn default() -> Self {
        Self {
            current_view: ViewType::EDIT,
            network: Network::default(),
            selected: HashSet::default(),
            selected_groups: HashSet::default(),
            edit_view: ViewState::default(),
            execute_view: ViewState {
                selected_tool: "select",
            },
            canvas_offset: (0.0, 0.0),
            canvas_zoom: 1.0,
            pan_anim: None,
            app_time_ms: 0.0,
            app_settings: AppSettings::default(),
            sim_settings: SimulationSettings::default(),
            context_menu: None,
        }
    }
}

impl AppStore {
    pub fn open_context_menu(&mut self, x: f64, y: f64, target: ContextMenuTarget, name: Option<String>) {
        self.context_menu = Some(ContextMenuState { x, y, target, name });
    }
    pub fn close_context_menu(&mut self) {
        self.context_menu = None;
    }
    pub fn get_selected_neurons(&self) -> Vec<&Neuron> {
        self.network
            .neurons
            .iter()
            .filter(|(id, _)| self.selected.contains(id))
            .map(|(_, n)| n)
            .collect()
    }
    pub fn clear_selection(&mut self) {
        self.selected.clear();
        self.selected_groups.clear();
    }
    pub fn select_only(&mut self, id: NeuronId) {
        self.selected = HashSet::from([id]);
        self.selected_groups.clear();
    }
    pub fn select_all(&mut self) {
        self.selected = self.network.neurons.iter().map(|(id, _)| id).collect();
    }
    pub fn toggle_selected(&mut self, id: NeuronId) {
        if !self.selected.remove(&id) {
            self.selected.insert(id);
        }
    }
    pub fn ensure_selected(&mut self, id: NeuronId) {
        if !self.selected.iter().any(|x| *x == id) {
            self.selected.insert(id);
        }
    }
    pub fn select_group_only(&mut self, id: NeuronId) {
        self.selected_groups = HashSet::default();
        self.selected.clear();
    }
    pub fn toggle_group_selected(&mut self, id: GroupId) {
        if self.selected_groups.remove(&id) {
            for nid in self.network.groups[id].neurons.iter() {
                self.selected.retain(|&sid| sid != *nid);
            }
            for gid in self.network.groups[id].groups.iter() {
                self.selected_groups.retain(|&sgid| sgid != *gid);
            }
        } else {
            self.selected_groups.insert(id);

            for nid in self.network.groups[id].neurons.iter() {
                self.selected.insert(*nid);
            }
            for gid in self.network.groups[id].groups.iter() {
                self.selected_groups.insert(*gid);
            }
        }
    }
    pub fn ensure_group_selected(&mut self, id: GroupId) {
        if !self.selected_groups.iter().any(|x| *x == id) {
            self.selected_groups.insert(id);
        }
    }
    pub fn has_edge(&self, source: NeuronId, target: NeuronId) -> bool {
        self.network
            .synapses
            .iter()
            .any(|(_, e)| e.source == source && e.target == target)
    }
    pub fn push_edge(&mut self, source: NeuronId, target: NeuronId) -> SynapseId {
        if source == target {
            panic!("Self-edge is not allowed");
        }
        if let Some(existing) = self
            .network
            .synapses
            .iter()
            .find(|(_, e)| e.source == source && e.target == target)
        {
            return existing.0;
        }

        self.network.synapses.insert_with(|idx| Synapse {
            id: idx as SynapseId,
            source,
            target,
        })
    }
    pub fn pan_to(&mut self, position: Position, animate: bool) {
        let target_offset = (-position.0, -position.1);
        if animate {
            let start_offset = self.canvas_offset;
            self.pan_anim = Some(PanAnim::new(start_offset, target_offset, 350.0, self.app_time_ms));
        } else {
            self.canvas_offset = target_offset;
            self.pan_anim = None;
        }
    }
    pub fn set_tool(&mut self, view: ViewType, tool: &'static str) {
        match view {
            ViewType::EDIT => self.edit_view.selected_tool = tool,
            ViewType::EXECUTE => self.execute_view.selected_tool = tool,
        }
    }
    pub fn set_view_state(&self, view: ViewType) -> &ViewState {
        match view {
            ViewType::EDIT => &self.edit_view,
            ViewType::EXECUTE => &self.execute_view,
        }
    }

    pub fn group_selected(&mut self, name: &String) -> Option<GroupId> {
        if self.selected.is_empty() && self.selected_groups.is_empty() {
            return None;
        }

        let mut left = f64::MAX;
        let mut top = f64::MAX;
        let mut right = f64::MIN;
        let mut bottom = f64::MIN;

        for nid in self.selected.iter() {
            let p = self.network.neurons[*nid].position;
            left = left.min(p.0);
            top = top.min(p.1);
            right = right.max(p.0);
            bottom = bottom.max(p.1);
        }

        for gid in self.selected_groups.iter() {
            let r = self.network.groups[*gid].rect.clone();
            left = left.min(r.left);
            top = top.min(r.top);
            right = right.max(r.right);
            bottom = bottom.max(r.bottom);
        }

        let width = right - left;
        let height = bottom - top;
        let padding_x = width * 0.1;
        let padding_y = height * 0.1;

        let common_ancestor = self
            .selected
            .iter()
            .chain(self.selected_groups.iter())
            .map(|&id| {
                if self.network.groups.contains(id) {
                    self.network.groups[id].parent
                } else {
                    self.network.neurons[id].parent
                }
            })
            .fold(None, |acc, parent| match acc {
                existing if existing == parent => acc,
                Some(_) => None,
                None => parent,
            });

        let new_parent = common_ancestor;

        let new_idx = self.network.groups.insert_with(|idx| Group {
            id: idx as GroupId,
            name: name.clone(),
            parent: new_parent,
            neurons: vec![],
            groups: vec![],
            color: Color::default(),
            rect: Rect::from_corners(
                (left - padding_x, top - padding_y),
                (right + padding_x, bottom + padding_y),
            ),
        });

        let nodes1: Vec<_> = self.selected.iter().copied().collect();
        let groups1: Vec<_> = self.selected_groups.iter().copied().collect();
        let nodes2: Vec<_> = nodes1.clone();
        let groups2: Vec<_> = groups1.clone();
        let nodes3: Vec<_> = nodes1.clone();
        let groups3: Vec<_> = groups1.clone();

        self._remove_nodes_from_parent(nodes1, groups1, new_idx);
        self._reassign_nodes_and_groups(nodes2, groups2, new_parent, new_idx);

        let mut child_neurons: Vec<NeuronId> = vec![];

        for nid in nodes3 {
            if self.network.neurons[nid].parent == Some(new_idx) {
                child_neurons.push(nid);
            }
        }

        let mut child_groups: Vec<GroupId> = vec![];
        for gid in groups3 {
            if gid != new_idx {
                if self.network.groups[gid].parent == Some(new_idx) {
                    child_groups.push(gid);
                }
            }
        }

        let new_group = self.network.groups.get_mut(new_idx).unwrap();
        new_group.neurons.extend(child_neurons);
        new_group.groups.extend(child_groups);

        self.selected.clear();
        self.toggle_group_selected(new_idx);

        Some(new_idx)
    }

    fn _remove_nodes_from_parent<N, G>(&mut self, nodes_to_move: N, groups_to_move: G, next_id: GroupId)
    where
        N: IntoIterator<Item = NeuronId>,
        G: IntoIterator<Item = GroupId>,
    {
        for nid in nodes_to_move {
            for (_, parent_group) in self.network.groups.iter_mut() {
                parent_group.neurons.retain(|&cid| cid != nid);
            }
        }

        for gid in groups_to_move {
            if gid != next_id {
                for (_, parent_group) in self.network.groups.iter_mut() {
                    parent_group.groups.retain(|&cid| cid != gid);
                }
            }
        }
    }

    fn _reassign_nodes_and_groups<N, G>(
        &mut self,
        nodes_to_move: N,
        groups_to_move: G,
        _old_parent: Option<GroupId>,
        next_id: GroupId,
    ) where
        N: IntoIterator<Item = NeuronId>,
        G: IntoIterator<Item = GroupId>,
    {
        for nid in nodes_to_move {
            if let Some(n) = self.network.neurons.get_mut(nid) {
                n.parent = Some(next_id);
            }
        }

        for gid in groups_to_move {
            if let Some(g) = self.network.groups.get_mut(gid) {
                if gid != next_id {
                    g.parent = Some(next_id);
                }
            }
        }
    }

    pub fn delete_neuron(&mut self, id: NeuronId) {
        self.network.remove_neuron(id);
        self.selected.remove(&id);
    }

    pub fn delete_edge(&mut self, id: SynapseId) {
        self.network.synapses.retain(|_, e| e.id != id);
    }

    pub fn delete_group_recursive(&mut self, id: GroupId) {
        self.network.remove_group(id);
        self.selected.clear();
        self.selected_groups.retain(|gid| *gid != id);
    }

    pub fn get_next_state_index(&self, model: &NeuronModelKind) -> StateIndex {
        if variant_eq(model, &NeuronModelKind::integrate_fire()) {
            self.network.executor.integrate_fire.voltage.len() as StateIndex
        } else {
            0
        }
    }
}
