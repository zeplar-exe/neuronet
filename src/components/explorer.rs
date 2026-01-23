use dioxus::prelude::*;
use crate::tauri_api::{get_group, get_groups, get_node, get_nodes, GroupInfo, NodeInfo};

#[derive(Clone, Debug, PartialEq)]
struct TreeNode {
    kind: TreeKind,
    id: i64,
    name: String,
    parent: i64,
    children: Vec<TreeNode>,
}

#[derive(Clone, Debug, PartialEq)]
enum TreeKind { Group, Node }

#[component]
pub fn Explorer() -> Element {
    let tree: Signal<Vec<TreeNode>> = use_signal(Vec::new);
    let expanded: Signal<std::collections::HashSet<i64>> = use_signal(Default::default);

    // Reload tree whenever mounted (simple approach)
    {
        let tree = tree.clone();
        use_future(move || async move {
            load_tree(tree).await;
        });
    }

    let header_style = "display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px;".to_string();

    rsx! {
        div { style: "color: #f6f6f6;",
            div { style: header_style,
                h3 { "Explorer" }
                button { onclick: move |_| {
                        let tree = tree.clone();
                        spawn(async move { load_tree(tree).await; });
                    }, "↻" }
            }
            for node in tree.read().iter() {
                RenderTreeNode { node: node.clone(), depth: 0, expanded: expanded.clone() }
            }
        }
    }
}

async fn load_tree(mut tree: Signal<Vec<TreeNode>>) {
    let group_ids = get_groups().await.unwrap_or_default();
    let node_ids = get_nodes().await.unwrap_or_default();

    let mut groups: std::collections::HashMap<i64, GroupInfo> = Default::default();
    let mut nodes: std::collections::HashMap<i64, NodeInfo> = Default::default();

    for id in group_ids { if let Some(g) = get_group(id).await { groups.insert(id, g); } }
    for id in node_ids { if let Some(n) = get_node(id).await { nodes.insert(id, n); } }

    let mut node_map: std::collections::HashMap<i64, TreeNode> = Default::default();
    for (_id, g) in groups.iter() {
        node_map.insert(g.id, TreeNode { kind: TreeKind::Group, id: g.id, name: g.name.clone(), parent: g.parent, children: vec![] });
    }
    for (_id, n) in nodes.iter() {
        node_map.insert(n.id, TreeNode { kind: TreeKind::Node, id: n.id, name: format!("{}", n.id), parent: n.parent, children: vec![] });
    }

    let mut roots: Vec<TreeNode> = vec![];
    let keys: Vec<i64> = node_map.keys().cloned().collect();
    for id in keys {
        if let Some(node) = node_map.remove(&id) {
            if node.parent == 0 { roots.push(node); } else {
                if let Some(parent) = node_map.get_mut(&node.parent) {
                    parent.children.push(node);
                } else {
                    // parent not found; treat as root
                    roots.push(node);
                }
            }
        }
    }

    // Simple sort: groups first, then nodes, by id
    fn sort_nodes(list: &mut Vec<TreeNode>) {
        list.sort_by(|a, b| {
            use std::cmp::Ordering::*;
            match (&a.kind, &b.kind) {
                (TreeKind::Group, TreeKind::Node) => Less,
                (TreeKind::Node, TreeKind::Group) => Greater,
                _ => a.id.cmp(&b.id),
            }
        });
        for n in list.iter_mut() { sort_nodes(&mut n.children); }
    }
    sort_nodes(&mut roots);
    tree.set(roots);
}

#[component]
fn RenderTreeNode(node: TreeNode, depth: i32, expanded: Signal<std::collections::HashSet<i64>>) -> Element {
    let is_group = matches!(node.kind, TreeKind::Group);
    let is_expanded = expanded.read().contains(&node.id);
    let has_children = !node.children.is_empty();

    let indent = depth * 16;
    let cursor = if is_group { "pointer" } else { "default" };
    let row_style = format!("padding-left: {indent}px; cursor: {cursor};");

    rsx! {
        div { class: "tree-node",
            div { class: "tree-item",
                style: row_style,
                onclick: move |_| {
                    if is_group {
                        let mut s = expanded.write();
                        if s.contains(&node.id) { s.remove(&node.id); } else { s.insert(node.id); }
                    }
                },
                if is_group { span { class: "tree-icon", if is_expanded { "▼" } else { "▶" } } }
                span { class: "tree-label", "{node.name}" }
            }
            if is_group && is_expanded && has_children {
                div { class: "tree-children",
                    for child in node.children.iter() {
                        RenderTreeNode { node: child.clone(), depth: depth + 1, expanded: expanded.clone() }
                    }
                }
            }
        }
    }
}
