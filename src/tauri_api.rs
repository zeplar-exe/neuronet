use dioxus::prelude::*;

// Minimal types to mirror the React tauri.tsx
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeId(pub i64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupId(pub i64);

#[derive(Clone, Debug, PartialEq)]
pub struct NodeInfo {
    pub id: i64,
    pub pos_x: f64,
    pub pos_y: f64,
    pub parent: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupInfo {
    pub id: i64,
    pub name: String,
    pub node_ids: Vec<i64>,
    pub parent: i64,
}

// Stubs: replace with real tauri invocations in your Dioxus+Tauri app.

pub async fn get_nodes() -> Option<Vec<i64>> {
    // e.g., tauri::invoke("get_nodes", ...)
    Some(vec![]) // start empty
}

pub async fn get_node(id: i64) -> Option<NodeInfo> {
    // Placeholder sample node to visualize structure if needed.
    Some(NodeInfo { id, pos_x: 100.0, pos_y: 120.0, parent: 0 })
}

pub async fn create_node() -> Option<i64> {
    // Return a fake id; in real app, call a Tauri command
    static mut NEXT: i64 = 1000;
    unsafe {
        NEXT += 1;
        Some(NEXT)
    }
}

pub async fn get_groups() -> Option<Vec<i64>> { Some(vec![]) }

pub async fn get_group(id: i64) -> Option<GroupInfo> {
    Some(GroupInfo { id, name: format!("Group {id}"), node_ids: vec![], parent: 0 })
}
