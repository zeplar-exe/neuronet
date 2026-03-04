use dioxus::prelude::*;
use crate::state::AppStore;

#[derive(Clone, Debug, PartialEq)]
struct TreeNode {
    kind: TreeKind,
    neuron_id: i64,
    name: String,
    parent: i64,
    children: Vec<TreeNode>,
}

#[derive(Clone, Debug, PartialEq)]
enum TreeKind { Group, Neuron }

#[derive(Clone, Debug, PartialEq)]
struct CtxMenu { x: f64, y: f64, kind: TreeKind, id: i64, name: String }

#[component]
pub fn Explorer() -> Element {
    let mut store = use_context::<Signal<AppStore>>();
    let expanded: Signal<std::collections::HashSet<i64>> = use_signal(Default::default);
    let mut ctx_menu: Signal<Option<CtxMenu>> = use_signal(|| None);
    let tree: Vec<TreeNode> = build_tree(&store.read());

    let onkeydown = {
        let mut store = store.clone();
        move |e: KeyboardEvent| {
            match e.key() {
                Key::Character(k) if (k.eq_ignore_ascii_case("g")) && (e.modifiers().ctrl() || e.modifiers().meta()) => {
                    e.prevent_default();
                    store.write().group_selected(&"Group".to_string());
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

            if let Some(menu) = ctx_menu.read().clone() {
                div { class: "fixed inset-0 z-[65]", onclick: move |_| ctx_menu.set(None) }
                {
                    let kind_for_select = menu.kind.clone();
                    let id_for_select = menu.id;
                    let kind_for_delete = menu.kind.clone();
                    let id_for_delete = menu.id;
                    rsx! {
                        div { style: format!("position: absolute; left: {}px; top: {}px;", menu.x, menu.y), class: "nn-menu",
                        div { class: "nn-menu-item", onclick: move |_| {
                            let mut s = store.write();
                            match kind_for_select {
                                TreeKind::Neuron => s.select_only(id_for_select),
                                TreeKind::Group => s.select_group_only(id_for_select),
                            }
                            ctx_menu.set(None);
                        }, "Select" }
                        div { class: "nn-menu-item opacity-60", onclick: move |_| { ctx_menu.set(None); }, "Rename (coming soon)" }
                        div { class: "nn-menu-item opacity-60", onclick: move |_| {
                                let mut s = store.write();
                                s.group_selected(&"Group".to_string());
    
                                ctx_menu.set(None);
                        }, "Group" }
                        div { class: "nn-menu-item", onclick: move |_| {
                            {
                                let mut s = store.write();
                                match kind_for_delete {
                                    TreeKind::Neuron => s.delete_neuron(id_for_delete),
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
    let mut buckets: std::collections::HashMap<i64, Vec<TreeNode>> = Default::default();

    for group in store.network.groups.iter() {
        buckets.entry(group.parent).or_default().push(TreeNode {
            kind: TreeKind::Group,
            neuron_id: group.id,
            name: group.name.clone(),
            parent: group.parent,
            children: vec![],
        });
    }
    for neuron in store.network.neurons.iter() {
        buckets.entry(neuron.parent).or_default().push(TreeNode {
            kind: TreeKind::Neuron,
            neuron_id: neuron.id,
            name: neuron.id.to_string(),
            parent: neuron.parent,
            children: vec![],
        });
    }

    fn sort_nodes(list: &mut Vec<TreeNode>) {
        use std::cmp::Ordering::*;
        list.sort_by(|a, b| match (&a.kind, &b.kind) {
            (TreeKind::Group, TreeKind::Neuron) => Less,
            (TreeKind::Neuron, TreeKind::Group) => Greater,
            _ => a.neuron_id.cmp(&b.neuron_id),
        });
    }

    fn assemble(parent_id: i64, buckets: &mut std::collections::HashMap<i64, Vec<TreeNode>>) -> Vec<TreeNode> {
        let mut items = buckets.remove(&parent_id).unwrap_or_default();
        
        for item in items.iter_mut() {
            if let TreeKind::Group = item.kind {
                let children = assemble(item.neuron_id, buckets);
                item.children = children;
            }
        }
        
        sort_nodes(&mut items);
        items
    }

    let mut roots = assemble(0, &mut buckets);

    if !buckets.is_empty() {
        let mut extra: Vec<TreeNode> = vec![];
        
        for (_, mut v) in buckets.drain() {
            extra.append(&mut v);
        }
        
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
    let is_expanded = expanded.read().contains(&node.neuron_id);
    let has_children = !node.children.is_empty();
    let is_selected = if is_group {
        store.read().selected_groups.iter().any(|&gid| gid == node.neuron_id)
    } else {
        store.read().selected.iter().any(|&sid| sid == node.neuron_id)
    };
    let neuron_data = store.read().get_neuron(node.neuron_id).clone();

    let indent = depth * 16;
    let cursor_class = if is_group { "cursor-pointer" } else { "cursor-default" };
    let row_style = format!("padding-left: {indent}px;");

    rsx! {
        div { class: "tree-node",
            div { class: format!("tree-item {} {} {} nn-select-none", cursor_class, if !is_group {"node"} else {""}, if is_selected {"selected"} else {""}),
                style: row_style,
                onclick: move |e| {
                    if is_group {
                        if e.modifiers().ctrl() || e.modifiers().meta() {
                            store.write().toggle_group_selected(node.neuron_id);
                        } else {
                            let mut s = expanded.write();
                            if s.contains(&node.neuron_id) { s.remove(&node.neuron_id); } else { s.insert(node.neuron_id); }
                        }
                    } else {
                        if e.modifiers().ctrl() || e.modifiers().meta() {
                            store.write().toggle_selected(node.neuron_id);
                        } else {
                            store.write().select_only(node.neuron_id);
                        }
                    }
                },
                ondoubleclick: move |e| {
                    e.prevent_default();
                    
                    store.write().pan_to(neuron_data.position, true);
                },
                oncontextmenu: move |e: MouseEvent| {
                    e.prevent_default();
                    let p = e.client_coordinates();
                    ctx_menu.set(Some(CtxMenu {
                        x: p.x,
                        y: p.y,
                        kind: node.kind.clone(),
                        id: node.neuron_id,
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
