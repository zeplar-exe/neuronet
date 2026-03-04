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
    pub parent: GroupId
}

#[derive(Clone, Default, PartialEq)]
pub struct Edge {
    pub id: EdgeId,
    pub source: NeuronId,
    pub target: NeuronId
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
        Color { red: 255u8, green: 255u8, blue: 255u8, alpha: 255 }
    }
}

impl Color {
    pub fn hex(&self) -> String { format!("#{:02X?}", [self.red, self.green, self.blue, self.alpha]) }
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
            right: bottom_right.0
        }
    }

    pub fn from_dimensions(position: Position, size: Position) -> Rect {
        Rect {
            top: position.1,
            left: position.0,
            bottom: position.1 + size.1,
            right: position.0 + size.0
        }
    }

    pub fn from_dimensions_f(left: f64, top: f64, width: f64, height: f64) -> Rect {
        Rect { left, top, bottom: top + height, right: left + width }
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

#[derive(Clone, Default, PartialEq)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub parent: GroupId,
    pub neurons: Vec<i64>,
    pub groups: Vec<i64>,
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
    pub leaky: Vec<bool>,
    pub leak_constant: Vec<Voltage>,
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
            g.neurons.retain(|&cid| cid != id);
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

    pub fn remove_group(&mut self, id: GroupId) {

    }

    pub fn get_neuron(&self, id: NeuronId) -> &Neuron {
        &self.neurons[id as usize]
    }

    pub fn get_edge(&self, id: EdgeId) -> &Edge {
        &self.edges[id as usize]
    }

    pub fn get_group(&self, id: GroupId) -> &Group {
        &self.groups.iter().find(|g| g.id == id).unwrap()
    }

    pub fn get_neuron_mut(&mut self, id: NeuronId) -> &mut Neuron {
        &mut self.neurons[id as usize]
    }

    pub fn get_edge_mut(&mut self, id: EdgeId) -> &mut Edge {
        &mut self.edges[id as usize]
    }

    pub fn get_group_mut(&mut self, id: GroupId) -> &mut Group {
        let index = self.groups.iter().position(|g| g.id == id).unwrap();
        &mut self.groups[index]
    }

    pub fn get_next_neuron_id(&self) -> NeuronId {
        self.neurons.len() as NeuronId
    }

    pub fn get_next_group_id(&self) -> GroupId {
        self.groups.len() as GroupId
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
    pub canvas_offset: Position,
    pub canvas_zoom: f64,
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
            canvas_offset: (0.0, 0.0),
            canvas_zoom: 1.0,
            app_settings: AppSettings::default(),
            sim_settings: SimulationSettings::default(),
        }
    }
}

impl AppStore {
    pub fn get_neuron(&self, id: NeuronId) -> &Neuron {
        &self.network.neurons[id as usize]
    }
    pub fn get_group(&self, id: GroupId) -> &Group {
        &self.network.groups.iter().find(|g| g.id == id).unwrap()
    }
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
    pub fn select_group_only(&mut self, id: NeuronId) { self.selected_groups = vec![id]; self.selected.clear(); }
    pub fn toggle_group_selected(&mut self, id: GroupId) {
        if let Some(idx) = self.selected_groups.iter().position(|x| *x == id) {
            self.selected_groups.remove(idx);

            for nid in self.network.groups.iter().find(|g| g.id == id).unwrap().neurons.iter() {
                self.selected.retain(|&sid| sid != *nid);
            }
            for gid in self.network.groups.iter().find(|g| g.id == id).unwrap().groups.iter() {
                self.selected_groups.retain(|&sgid| sgid != *gid);
            }
        } else {
            self.selected_groups.push(id);

            for nid in self.network.groups.iter().find(|g| g.id == id).unwrap().neurons.iter() {
                self.selected.push(*nid as usize as NeuronId);
            }
            for gid in self.network.groups.iter().find(|g| g.id == id).unwrap().groups.iter() {
                self.selected_groups.push(*gid as usize as GroupId);
            }
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
    pub fn pan_to(&mut self, position: Position, animate: bool) {
        let _ = animate;
        self.canvas_offset = (-position.0, -position.1);
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
        let any = !self.selected.is_empty() || !self.selected_groups.is_empty();
        if !any { return None; }
        
        let next_id = self.network.groups.iter().map(|g| g.id).max().unwrap_or(0) + 1;

        let mut left = f64::MAX;
        let mut top = f64::MAX;
        let mut right = f64::MIN;
        let mut bottom = f64::MIN;

        for &nid in self.selected.iter() {
            let n = self.network.get_neuron(nid);
            left = left.min(n.position.0);
            top = top.min(n.position.1);
            right = right.max(n.position.0);
            bottom = bottom.max(n.position.1);
        }

        for &gid in self.selected_groups.iter() {
            let g = self.network.get_group(gid);
            left = left.min(g.rect.left);
            top = top.min(g.rect.top);
            right = right.max(g.rect.right);
            bottom = bottom.max(g.rect.bottom);
        }

        let width = right - left;
        let height = bottom - top;
        let padding_x = width * 0.1;
        let padding_y = height * 0.1;

        self.network.groups.push(Group {
            id: next_id,
            name: name.clone(),
            parent: 0,
            neurons: vec![],
            groups: vec![],
            color: Color::default(),
            rect: Rect::from_corners((left - padding_x, top - padding_y), (right + padding_x, bottom + padding_y)),
        });

        let nodes_to_move = self.selected.clone();
        let groups_to_move = self.selected_groups.clone();

        let common_ancestor = self.selected.iter()
            .chain(self.selected_groups.iter())
            .map(|&id| match self.network.groups.iter().find(|g| g.id == id) {
                Some(group) => group.parent,
                None => self.network.get_neuron(id).parent,
            })
            .fold(None, |acc, parent| {
                match acc {
                    Some(existing) if existing == parent => acc,
                    Some(_) => None,
                    None => Some(parent),
                }
            });

        let new_parent = common_ancestor.unwrap_or(0);

        self._remove_nodes_from_parent(&nodes_to_move, &groups_to_move, next_id);
        self._reassign_nodes_and_groups(&nodes_to_move, &groups_to_move, new_parent, next_id);

        {
            let mut nodes = vec!();
            let mut groups = vec!();

            for nid in nodes_to_move.iter() {
                if self.network.get_neuron(*nid).parent == new_parent {
                    nodes.push(*nid);
                }
            }
            for gid in groups_to_move.iter() {
                if self.network.get_group(*gid).parent == new_parent && *gid != next_id {
                    groups.push(*gid);
                }
            }

            let parent = self.network.groups.iter_mut().find(|pg| pg.id == new_parent).unwrap();
            for nid in nodes.iter() {
                parent.neurons.push(*nid);
            }
            for gid in groups.iter() {
                parent.groups.push(*gid);
            }
        }

        self.selected.clear();
        self.toggle_group_selected(next_id);

        Some(next_id)
    }

    pub fn delete_neuron(&mut self, id: NeuronId) {
        self.network.edges.retain(|e| e.source != id && e.target != id);
        
        for g in self.network.groups.iter_mut() {
            g.neurons.retain(|&cid| cid != id);
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
            pg.groups.retain(|cid| !groups_to_delete.contains(cid));
        }
        self.network.groups.retain(|g| !groups_to_delete.contains(&g.id));
        self.selected_groups.retain(|gid| !groups_to_delete.contains(gid));
    }

    pub fn get_next_state_index(&self, model: &NeuronModelKind) -> StateIndex {
        if variant_eq(model, &NeuronModelKind::integrate_fire()) {
            self.network.executor.integrate_fire.voltage.len() as StateIndex
        } else { -1 }
    }

    fn _remove_nodes_from_parent(&mut self, nodes_to_move: &[NeuronId], groups_to_move: &[GroupId], next_id: GroupId) {
        for nid in nodes_to_move.iter() {
            for parent_group in self.network.groups.iter_mut() {
                parent_group.neurons.retain(|&cid| cid != *nid);
            }
        }

        for gid in groups_to_move.iter() {
            if *gid != next_id {
                for parent_group in self.network.groups.iter_mut() {
                    parent_group.groups.retain(|&cid| cid != *gid);
                }
            }
        }
    }

    fn _reassign_nodes_and_groups(&mut self, nodes_to_move: &[NeuronId], groups_to_move: &[GroupId], new_parent: GroupId, next_id: GroupId) {
        fn inside(network: &Network, selected_groups: &Vec<GroupId>, n: &Neuron) -> bool {
            selected_groups.iter().all(|&gid| !network.get_group(gid).neurons.contains(&n.id))
        }

        let mut nodes_inside = vec!();

        for n in self.network.neurons.iter() {
            if nodes_to_move.contains(&n.id) && inside(&self.network, &self.selected_groups, n) {
                nodes_inside.push(n.id);
            }
        }

        for n in self.network.neurons.iter_mut() {
            if nodes_inside.contains(&n.id) && n.parent == new_parent {
                n.parent = next_id;
            }
        }

        for g in self.network.groups.iter_mut() {
            if g.id != next_id && groups_to_move.contains(&g.id) && g.parent == new_parent {
                g.parent = next_id;
            }
        }
    }
}