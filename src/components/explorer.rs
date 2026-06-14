use crate::gen::generate_group_name;
use crate::simulation::id::{GroupId, NeuronId};
use crate::state::{AppStore, ContextMenuTarget};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
struct TreeNode {
    kind: TreeKind,
    item_id: NeuronId,
    name: String,
    parent: Option<GroupId>,
    children: Vec<TreeNode>,
}

#[derive(Clone, Debug, PartialEq)]
enum TreeKind {
    Group,
    Neuron,
}

#[component]
pub fn Explorer() -> Element {
    let store = use_context::<Signal<AppStore>>();
    let expanded: Signal<std::collections::HashSet<NeuronId>> = use_signal(Default::default);
    let tree: Vec<TreeNode> = build_tree(&store.read());

    let onkeydown = {
        let mut store = store.clone();
        move |e: KeyboardEvent| match e.key() {
            Key::Character(k) if (k.eq_ignore_ascii_case("g")) && (e.modifiers().ctrl() || e.modifiers().meta()) => {
                e.prevent_default();
                store.write().group_selected(&generate_group_name());
            }
            _ => {}
        }
    };

    rsx! {
        div { class: "text-[#f6f6f6] relative", tabindex: 0, onkeydown: onkeydown,
            div { class: "flex items-center justify-between mb-2",
                h3 { "Explorer" }
            }
            for node in tree.iter() {
                RenderTreeNode { node: node.clone(), depth: 0, expanded: expanded.clone(), store: store.clone() }
            }
        }
    }
}

fn build_tree(store: &AppStore) -> Vec<TreeNode> {
    let mut buckets: HashMap<Option<GroupId>, Vec<TreeNode>> = Default::default();

    for (_, group) in store.network.groups.iter() {
        buckets.entry(group.parent).or_default().push(TreeNode {
            kind: TreeKind::Group,
            item_id: group.id,
            name: group.name.clone(),
            parent: group.parent,
            children: vec![],
        });
    }
    for (_, neuron) in store.network.neurons.iter() {
        buckets.entry(neuron.parent).or_default().push(TreeNode {
            kind: TreeKind::Neuron,
            item_id: neuron.id,
            name: neuron.name.clone(),
            parent: neuron.parent,
            children: vec![],
        });
    }

    fn sort_nodes(list: &mut Vec<TreeNode>) {
        use std::cmp::Ordering::*;
        list.sort_by(|a, b| match (&a.kind, &b.kind) {
            (TreeKind::Group, TreeKind::Neuron) => Less,
            (TreeKind::Neuron, TreeKind::Group) => Greater,
            _ => a.item_id.cmp(&b.item_id),
        });
    }

    fn assemble(parent_id: Option<GroupId>, buckets: &mut HashMap<Option<GroupId>, Vec<TreeNode>>) -> Vec<TreeNode> {
        let mut items = buckets.remove(&parent_id).unwrap_or_default();

        for item in items.iter_mut() {
            if let TreeKind::Group = item.kind {
                let children = assemble(Some(item.item_id), buckets);
                item.children = children;
            }
        }

        sort_nodes(&mut items);
        items
    }

    assemble(None, &mut buckets)
}

#[component]
fn RenderTreeNode(node: TreeNode, depth: i32, expanded: Signal<HashSet<GroupId>>, store: Signal<AppStore>) -> Element {
    let is_group = matches!(node.kind, TreeKind::Group);
    let is_expanded = expanded.read().contains(&node.item_id);
    let has_children = !node.children.is_empty();
    let is_selected = if is_group {
        store.read().selected_groups.iter().any(|&gid| gid == node.item_id)
    } else {
        store.read().selected.iter().any(|&sid| sid == node.item_id)
    };
    // NOTE: Only neurons have neuron data; do not attempt to fetch neuron data for groups.

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
                            store.write().toggle_group_selected(node.item_id);
                        } else {
                            let mut s = expanded.write();
                            if s.contains(&node.item_id) { s.remove(&node.item_id); } else { s.insert(node.item_id); }
                        }
                    } else {
                        if e.modifiers().ctrl() || e.modifiers().meta() {
                            store.write().toggle_selected(node.item_id);
                        } else {
                            store.write().select_only(node.item_id);
                        }
                    }
                },
                ondoubleclick: move |e| {
                    if !is_group {
                        e.prevent_default();
                        let s = store.write().clone();
                        let mut s2 = store.write().clone();
                        if let Some(n) = s.network.neurons.get(node.item_id) {
                            s2.pan_to(n.position, true);
                        }
                    }
                },
                oncontextmenu: move |e: MouseEvent| {
                    e.prevent_default();
                    let p = e.client_coordinates();
                    let target = match node.kind {
                        TreeKind::Neuron => ContextMenuTarget::Neuron(node.item_id),
                        TreeKind::Group => ContextMenuTarget::Group(node.item_id),
                    };
                    store.write().open_context_menu(p.x, p.y, target, Some(node.name.clone()));
                },
                if is_group { span { class: "tree-icon", if is_expanded { "▼" } else { "▶" } } }
                span { class: "tree-label", "{node.name}" }
            }
            if is_group && is_expanded && has_children {
                div { class: "tree-children",
                    for child in node.children.iter() {
                        RenderTreeNode { node: child.clone(), depth: depth + 1, expanded: expanded.clone(), store: store.clone() }
                    }
                }
            }
        }
    }
}
