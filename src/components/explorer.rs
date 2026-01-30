use dioxus::prelude::*;
use crate::state::AppStore;

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

#[derive(Clone, Debug, PartialEq)]
struct CtxMenu { x: f64, y: f64, kind: TreeKind, id: i64, name: String }

#[component]
pub fn Explorer() -> Element {
    let mut store = use_context::<Signal<AppStore>>();
    // Track expanded groups
    let expanded: Signal<std::collections::HashSet<i64>> = use_signal(Default::default);
    // Context menu state
    let mut ctx_menu: Signal<Option<CtxMenu>> = use_signal(|| None);
    // Build tree reactively from store — real-time updates
    let tree: Vec<TreeNode> = build_tree(&store.read());

    // Keydown handler to support grouping when Explorer has focus
    let onkeydown = {
        let mut store = store.clone();
        move |e: KeyboardEvent| {
            match e.key() {
                Key::Character(k) if (k.eq_ignore_ascii_case("g")) && (e.modifiers().ctrl() || e.modifiers().meta()) => {
                    e.prevent_default();
                    store.write().group_selected(None);
                }
                _ => {}
            }
        }
    };

    rsx! {
        div { class: "text-[#f6f6f6] relative", tabindex: 0, onkeydown: onkeydown,
            div { class: "flex items-center justify-between mb-2",
                h3 { "Explorer" }
            }
            for node in tree.iter() {
                RenderTreeNode { node: node.clone(), depth: 0, expanded: expanded.clone(), store: store.clone(), ctx_menu: ctx_menu.clone() }
            }

            // Context menu overlay
            if let Some(menu) = ctx_menu.read().clone() {
                // Backdrop to close
                div { class: "fixed inset-0 z-[65]", onclick: move |_| ctx_menu.set(None) }
                // Menu box positioned within sidebar (absolute is fine since sidebar is fixed)
                {
                    // Clone values for use across multiple move closures
                    let kind_for_select = menu.kind.clone();
                    let id_for_select = menu.id;
                    let kind_for_delete = menu.kind.clone();
                    let id_for_delete = menu.id;
                    rsx! {
                        div { style: format!("position: absolute; left: {}px; top: {}px;", menu.x, menu.y), class: "nn-menu",
                    div { class: "nn-menu-item", onclick: move |_| {
                        // Select: nodes select-only; groups select group only
                        let mut s = store.write();
                        match kind_for_select {
                            TreeKind::Node => s.select_only(id_for_select),
                            TreeKind::Group => s.select_group_only(id_for_select),
                        }
                        ctx_menu.set(None);
                    }, "Select" }
                    div { class: "nn-menu-item opacity-60", onclick: move |_| { ctx_menu.set(None); }, "Rename (coming soon)" }
                    div { class: "nn-menu-item", onclick: move |_| {
                        // Delete node or group (recursive)
                        {
                            let mut s = store.write();
                            match kind_for_delete {
                                TreeKind::Node => s.delete_node(id_for_delete),
                                TreeKind::Group => s.delete_group_recursive(id_for_delete),
                            }
                        }
                        ctx_menu.set(None);
                    }, "Delete" }
                }
                    }
                }
            }
        }
    }
}

fn build_tree(store: &AppStore) -> Vec<TreeNode> {
    // Bucket all entries by parent id to avoid ordering issues
    let mut buckets: std::collections::HashMap<i64, Vec<TreeNode>> = Default::default();

    // Groups
    for group in store.groups.iter() {
        buckets.entry(group.parent).or_default().push(TreeNode {
            kind: TreeKind::Group,
            id: group.id,
            name: group.name.clone(),
            parent: group.parent,
            children: vec![],
        });
    }
    // Nodes
    for node in store.nodes.iter() {
        buckets.entry(node.parent).or_default().push(TreeNode {
            kind: TreeKind::Node,
            id: node.id,
            name: node.id.to_string(),
            parent: node.parent,
            children: vec![],
        });
    }

    fn sort_nodes(list: &mut Vec<TreeNode>) {
        use std::cmp::Ordering::*;
        list.sort_by(|a, b| match (&a.kind, &b.kind) {
            (TreeKind::Group, TreeKind::Node) => Less,
            (TreeKind::Node, TreeKind::Group) => Greater,
            _ => a.id.cmp(&b.id),
        });
    }

    fn assemble(parent_id: i64, buckets: &mut std::collections::HashMap<i64, Vec<TreeNode>>) -> Vec<TreeNode> {
        // Take the children for this parent if present
        let mut items = buckets.remove(&parent_id).unwrap_or_default();
        // For each group, assemble its subtree
        for item in items.iter_mut() {
            if let TreeKind::Group = item.kind {
                let children = assemble(item.id, buckets);
                item.children = children;
            }
        }
        // Sort groups before nodes and by id
        sort_nodes(&mut items);
        items
    }

    // Build from root parent (0)
    let mut roots = assemble(0, &mut buckets);

    // Any remaining entries with missing parents become additional roots
    // (handles orphaned nodes/groups gracefully)
    if !buckets.is_empty() {
        let mut extra: Vec<TreeNode> = vec![];
        // Collect all remaining and clear buckets
        for (_, mut v) in buckets.drain() {
            extra.append(&mut v);
        }
        // For any remaining groups in extras, ensure their nested children are connected
        // by running assemble on their ids using an empty buckets map (no further nesting possible now)
        sort_nodes(&mut extra);
        roots.extend(extra);
        sort_nodes(&mut roots);
    }

    roots
}

#[component]
fn RenderTreeNode(
    node: TreeNode,
    depth: i32,
    expanded: Signal<std::collections::HashSet<i64>>,
    store: Signal<AppStore>,
    ctx_menu: Signal<Option<CtxMenu>>,
) -> Element {
    let is_group = matches!(node.kind, TreeKind::Group);
    let is_expanded = expanded.read().contains(&node.id);
    let has_children = !node.children.is_empty();
    let is_selected = if is_group {
        store.read().selected_groups.iter().any(|&gid| gid == node.id)
    } else {
        store.read().selected.iter().any(|&sid| sid == node.id)
    };

    let indent = depth * 16;
    let cursor_class = if is_group { "cursor-pointer" } else { "cursor-default" };
    let row_style = format!("padding-left: {indent}px;");

    rsx! {
        div { class: "tree-node",
            div { class: format!("tree-item {} {} {} nn-select-none", cursor_class, if !is_group {"node"} else {""}, if is_selected {"selected"} else {""}),
                style: row_style,
                onclick: move |e| {
                    if is_group {
                        // Ctrl/Cmd click toggles group selection; otherwise expand/collapse
                        if e.modifiers().ctrl() || e.modifiers().meta() {
                            store.write().toggle_group_selected(node.id);
                        } else {
                            let mut s = expanded.write();
                            if s.contains(&node.id) { s.remove(&node.id); } else { s.insert(node.id); }
                        }
                    } else {
                        // Node: Ctrl/Cmd toggles, otherwise select only
                        if e.modifiers().ctrl() || e.modifiers().meta() {
                            store.write().toggle_selected(node.id);
                        } else {
                            store.write().select_only(node.id);
                        }
                    }
                },
                oncontextmenu: move |e: MouseEvent| {
                    e.prevent_default();
                    let p = e.client_coordinates();
                    ctx_menu.set(Some(CtxMenu {
                        x: p.x as f64,
                        y: p.y as f64,
                        kind: node.kind.clone(),
                        id: node.id,
                        name: node.name.clone(),
                    }));
                },
                if is_group { span { class: "tree-icon", if is_expanded { "▼" } else { "▶" } } }
                span { class: "tree-label", "{node.name}" }
            }
            if is_group && is_expanded && has_children {
                div { class: "tree-children",
                    for child in node.children.iter() {
                        RenderTreeNode { node: child.clone(), depth: depth + 1, expanded: expanded.clone(), store: store.clone(), ctx_menu: ctx_menu.clone() }
                    }
                }
            }
        }
    }
}
