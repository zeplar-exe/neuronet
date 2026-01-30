use crate::models::{NeuronModel, NeuronModelKind};
use crate::settings::{AppSettings, SimulationSettings};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ViewType {
    EDIT,
    CONFIGURE,
    EXECUTE,
}

#[derive(Clone, Default, PartialEq)]
pub struct Neuron {
    pub id: i64,
    pub model: NeuronModelKind,
    pub position: (f64, f64),
    pub parent: i64,
}

#[derive(Clone, Default, PartialEq)]
pub struct Edge {
    pub id: i64,
    pub source: i64,
    pub target: i64,
    pub animated: bool,
}

#[derive(Clone, Default, PartialEq)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub parent: i64,
    pub children: Vec<i64>,
}

#[derive(Clone)]
pub struct PrimaryTool {
    pub name: &'static str,
}

#[derive(Clone)]
pub struct ViewState {
    pub selected_tool: PrimaryTool,
}

impl Default for ViewState {
    fn default() -> Self {
        Self { selected_tool: PrimaryTool { name: "add" } }
    }
}

#[derive(Clone)]
pub struct AppStore {
    pub current_view: ViewType,
    pub nodes: Vec<Neuron>,
    pub edges: Vec<Edge>,
    pub groups: Vec<Group>,
    pub selected: Vec<i64>,
    pub selected_groups: Vec<i64>,
    pub edit_view: ViewState,
    pub configure_view: ViewState,
    pub execute_view: ViewState,
    pub app_settings: AppSettings,
    pub sim_settings: SimulationSettings
}

impl Default for AppStore {
    fn default() -> Self {
        Self {
            current_view: ViewType::EDIT,
            nodes: vec![],
            edges: vec![],
            groups: vec![],
            selected: vec![],
            selected_groups: vec![],
            edit_view: ViewState::default(),
            configure_view: ViewState::default(),
            execute_view: ViewState::default(),
            app_settings: AppSettings::default(),
            sim_settings: SimulationSettings::default(),
        }
    }
}

impl AppStore {
    pub fn push_neuron(&mut self, node: Neuron) { self.nodes.push(node); }
    pub fn get_selected_neurons(&self) -> Vec<&Neuron> {
        self.nodes.iter().filter(|n| self.selected.contains(&n.id)).collect()
    }
    pub fn clear_selection(&mut self) { self.selected.clear(); self.selected_groups.clear(); }
    pub fn select_only(&mut self, id: i64) { self.selected = vec![id]; self.selected_groups.clear(); }
    pub fn select_all(&mut self) {
        self.selected = self.nodes.iter().map(|n| n.id).collect();
    }
    pub fn toggle_selected(&mut self, id: i64) {
        if let Some(idx) = self.selected.iter().position(|x| *x == id) {
            self.selected.remove(idx);
        } else {
            self.selected.push(id);
        }
    }
    pub fn ensure_selected(&mut self, id: i64) {
        if !self.selected.iter().any(|x| *x == id) {
            self.selected.push(id);
        }
    }
    fn remove_child_from_parents(&mut self, child_id: i64) {
        for g in self.groups.iter_mut() {
            if let Some(pos) = g.children.iter().position(|&cid| cid == child_id) {
                g.children.remove(pos);
            }
        }
    }
    pub fn clear_group_selection(&mut self) { self.selected_groups.clear(); }
    pub fn select_group_only(&mut self, id: i64) { self.selected_groups = vec![id]; self.selected.clear(); }
    pub fn toggle_group_selected(&mut self, id: i64) {
        if let Some(idx) = self.selected_groups.iter().position(|x| *x == id) {
            self.selected_groups.remove(idx);
        } else {
            self.selected_groups.push(id);
        }
    }
    pub fn ensure_group_selected(&mut self, id: i64) {
        if !self.selected_groups.iter().any(|x| *x == id) {
            self.selected_groups.push(id);
        }
    }
    pub fn has_edge(&self, source: i64, target: i64) -> bool {
        self.edges.iter().any(|e| e.source == source && e.target == target)
    }
    pub fn push_edge(&mut self, source: i64, target: i64) -> i64 {
        if source == target {
            return 0;
        }
        if let Some(existing) = self.edges.iter().find(|e| e.source == source && e.target == target) {
            return existing.id;
        }
        
        let next_id = self.edges.iter().map(|e| e.id).max().unwrap_or(0) + 1;
        self.edges.push(Edge { id: next_id, source, target, animated: false });
        next_id
    }
    pub fn set_tool(&mut self, view: ViewType, tool: PrimaryTool) {
        match view {
            ViewType::EDIT => self.edit_view.selected_tool = tool,
            ViewType::CONFIGURE => self.configure_view.selected_tool = tool,
            ViewType::EXECUTE => self.execute_view.selected_tool = tool,
        }
    }
    pub fn view_state(&self, view: ViewType) -> &ViewState {
        match view {
            ViewType::EDIT => &self.edit_view,
            ViewType::CONFIGURE => &self.configure_view,
            ViewType::EXECUTE => &self.execute_view,
        }
    }

    pub fn group_selected(&mut self, name: Option<String>) -> Option<i64> {
        let any = !self.selected.is_empty() || !self.selected_groups.is_empty();
        if !any { return None; }
        
        let next_id = self.groups.iter().map(|g| g.id).max().unwrap_or(0) + 1;
        let group_name = name.unwrap_or_else(|| format!("Group {}", next_id));
        
        self.groups.push(Group { id: next_id, name: group_name, parent: 0, children: vec![] });

        let nodes_to_move = self.selected.clone();
        let groups_to_move = self.selected_groups.clone();

        for nid in nodes_to_move.iter() { self.remove_child_from_parents(*nid); }
        for gid in groups_to_move.iter() { if *gid != next_id { self.remove_child_from_parents(*gid); } }
        
        for n in self.nodes.iter_mut() {
            if nodes_to_move.contains(&n.id) { n.parent = next_id; }
        }
        
        for g in self.groups.iter_mut() {
            if g.id != next_id && groups_to_move.contains(&g.id) { g.parent = next_id; }
        }
        
        if let Some(parent) = self.groups.iter_mut().find(|pg| pg.id == next_id) {
            for nid in nodes_to_move.iter() { parent.children.push(*nid); }
            for gid in groups_to_move.iter() { if *gid != next_id { parent.children.push(*gid); } }
        }

        self.selected.clear();
        self.selected_groups = vec![next_id];

        Some(next_id)
    }

    pub fn delete_neuron(&mut self, id: i64) {
        self.edges.retain(|e| e.source != id && e.target != id);
        
        for g in self.groups.iter_mut() {
            g.children.retain(|&cid| cid != id);
        }
        
        self.nodes.retain(|n| n.id != id);
        self.selected.retain(|&sid| sid != id);
    }

    pub fn delete_group_recursive(&mut self, id: i64) {
        let mut groups_to_delete: Vec<i64> = vec![id];
        let mut nodes_to_delete: Vec<i64> = vec![];
        let mut idx = 0;

        while idx < groups_to_delete.len() {
            let gid = groups_to_delete[idx];

            for g in self.groups.iter() {
                if g.parent == gid && g.id != id {
                    if !groups_to_delete.contains(&g.id) {
                        groups_to_delete.push(g.id);
                    }
                }
            }
            // Children nodes
            for n in self.nodes.iter() {
                if n.parent == gid {
                    nodes_to_delete.push(n.id);
                }
            }
            idx += 1;
        }

        for nid in nodes_to_delete.iter() {
            self.delete_neuron(*nid);
        }

        for pg in self.groups.iter_mut() {
            pg.children.retain(|cid| !groups_to_delete.contains(cid));
        }
        // Then remove the groups themselves
        self.groups.retain(|g| !groups_to_delete.contains(&g.id));
        // Clear from selection
        self.selected_groups.retain(|gid| !groups_to_delete.contains(gid));
    }
}
