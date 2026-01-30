use dioxus::prelude::*;
use crate::models::{NeuronModel, NeuronModelKind};
use crate::settings::{AppSettings, SimulationSettings};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ViewType {
    EDIT,
    CONFIGURE,
    EXECUTE,
}

#[derive(Clone, Default, PartialEq)]
pub struct Node {
    pub id: Signal<i64>,
    pub model: Signal<NeuronModelKind>,
    pub position: Signal<(f64, f64)>,
    pub parent: Signal<i64>,
}

#[derive(Clone, Default, PartialEq)]
pub struct Edge {
    pub id: Signal<i64>,
    pub source: Signal<i64>,
    pub target: Signal<i64>,
    pub animated: Signal<bool>,
}

#[derive(Clone, Default, PartialEq)]
pub struct Group {
    pub id: Signal<i64>,
    pub name: Signal<String>,
    pub parent: Signal<i64>,
    pub children: Signal<Vec<i64>>,
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
    pub nodes: Vec<Node>,
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
    pub fn push_node(&mut self, node: Node) { self.nodes.push(node); }
    pub fn get_selected_neurons(&self) -> Vec<&Node> {
        self.nodes.iter().filter(|n| self.selected.contains(&(n.id)())).collect()
    }
    pub fn clear_selection(&mut self) { self.selected.clear(); self.selected_groups.clear(); }
    pub fn select_only(&mut self, id: i64) { self.selected = vec![id]; self.selected_groups.clear(); }
    pub fn select_all(&mut self) {
        self.selected = self.nodes.iter().map(|n| (n.id)()).collect();
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
            let mut children = g.children.write();
            if let Some(pos) = children.iter().position(|&cid| cid == child_id) {
                children.remove(pos);
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
        self.edges
            .iter()
            .any(|e| (e.source)() == source && (e.target)() == target)
    }
    pub fn push_edge(&mut self, source: i64, target: i64) -> i64 {
        if source == target {
            return 0;
        }
        if let Some(existing) = self
            .edges
            .iter()
            .find(|e| (e.source)() == source && (e.target)() == target)
        {
            return (existing.id)();
        }
        
        let next_id = self
            .edges
            .iter()
            .map(|e| (e.id)())
            .max()
            .unwrap_or(0)
            + 1;
        self.edges.push(Edge {
            id: Signal::new(next_id),
            source: Signal::new(source),
            target: Signal::new(target),
            animated: Signal::new(false),
        });
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
        
        let next_id = self
            .groups
            .iter()
            .map(|g| (g.id)())
            .max()
            .unwrap_or(0)
            + 1;
        let group_name = name.unwrap_or_else(|| format!("Group {}", next_id));
        
        self.groups.push(Group {
            id: Signal::new(next_id),
            name: Signal::new(group_name),
            parent: Signal::new(0),
            children: Signal::new(vec![]),
        });

        let nodes_to_move = self.selected.clone();
        let groups_to_move = self.selected_groups.clone();

        for nid in nodes_to_move.iter() { self.remove_child_from_parents(*nid); }
        for gid in groups_to_move.iter() { if *gid != next_id { self.remove_child_from_parents(*gid); } }
        
        for n in self.nodes.iter_mut() {
            if nodes_to_move.contains(&(n.id)()) {
                n.parent.set(next_id);
            }
        }
        
        for g in self.groups.iter_mut() {
            if (g.id)() != next_id && groups_to_move.contains(&(g.id)()) {
                g.parent.set(next_id);
            }
        }
        
        if let Some(parent) = self.groups.iter_mut().find(|pg| (pg.id)() == next_id) {
            let mut ch = parent.children.write();
            for nid in nodes_to_move.iter() { ch.push(*nid); }
            for gid in groups_to_move.iter() { if *gid != next_id { ch.push(*gid); } }
        }

        self.selected.clear();
        self.selected_groups = vec![next_id];

        Some(next_id)
    }

    pub fn delete_node(&mut self, id: i64) {
        self.edges.retain(|e| (e.source)() != id && (e.target)() != id);
        
        for g in self.groups.iter_mut() {
            g.children.write().retain(|&cid| cid != id);
        }
        
        self.nodes.retain(|n| (n.id)() != id);
        self.selected.retain(|&sid| sid != id);
    }

    pub fn delete_group_recursive(&mut self, id: i64) {
        let mut groups_to_delete: Vec<i64> = vec![id];
        let mut nodes_to_delete: Vec<i64> = vec![];
        let mut idx = 0;

        while idx < groups_to_delete.len() {
            let gid = groups_to_delete[idx];

            for g in self.groups.iter() {
                if (g.parent)() == gid && (g.id)() != id {
                    let gid_val = (g.id)();
                    if !groups_to_delete.contains(&gid_val) {
                        groups_to_delete.push(gid_val);
                    }
                }
            }
            // Children nodes
            for n in self.nodes.iter() {
                if (n.parent)() == gid {
                    nodes_to_delete.push((n.id)());
                }
            }
            idx += 1;
        }

        for nid in nodes_to_delete.iter() {
            self.delete_node(*nid);
        }

        for pg in self.groups.iter_mut() {
            pg.children.write().retain(|cid| !groups_to_delete.contains(cid));
        }
        // Then remove the groups themselves
        self.groups.retain(|g| {
            let id_val = (g.id)();
            !groups_to_delete.contains(&id_val)
        });
        // Clear from selection
        self.selected_groups.retain(|gid| !groups_to_delete.contains(gid));
    }
}
