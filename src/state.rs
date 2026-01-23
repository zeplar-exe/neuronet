use dioxus::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ViewType {
    EDIT,
    CONFIGURE,
    EXECUTE,
}

#[derive(Clone, Default, PartialEq)]
pub struct Node {
    pub id: String,
    pub r#type: Option<String>,
    pub data: serde_json::Value,
    pub position: (f64, f64),
    pub parent: i64,
}

#[derive(Clone, Default, PartialEq)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub animated: bool,
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
    pub selected: Vec<String>, // selected node ids
    pub edit_view: ViewState,
    pub configure_view: ViewState,
    pub execute_view: ViewState,
}

impl Default for AppStore {
    fn default() -> Self {
        Self {
            current_view: ViewType::EDIT,
            nodes: vec![],
            edges: vec![],
            selected: vec![],
            edit_view: ViewState::default(),
            configure_view: ViewState::default(),
            execute_view: ViewState::default(),
        }
    }
}

impl AppStore {
    pub fn set_nodes(&mut self, nodes: Vec<Node>) { self.nodes = nodes; }
    pub fn push_node(&mut self, node: Node) { self.nodes.push(node); }
    pub fn clear_selection(&mut self) { self.selected.clear(); }
    pub fn select_only(&mut self, id: &str) { self.selected = vec![id.to_string()]; }
    pub fn toggle_selected(&mut self, id: &str) {
        if let Some(idx) = self.selected.iter().position(|x| x == id) {
            self.selected.remove(idx);
        } else {
            self.selected.push(id.to_string());
        }
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
}
