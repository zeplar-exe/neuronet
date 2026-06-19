use std::collections::HashSet;

use generational_arena::{Arena, Index};

use crate::common::{Color, Position, Rect};
use crate::models::NeuronModelKind;
use crate::settings::{AppSettings, SimulationSettings};
use crate::simulation::id::{GroupId, NeuronId, StateIndex, SynapseId};
use crate::simulation::network::{Group, Network, Neuron, Synapse};
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
