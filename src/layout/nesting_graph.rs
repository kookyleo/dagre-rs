//! Nesting graph: handles compound graph hierarchy for layout.
//!
//! Creates dummy nodes for tops/bottoms of subgraphs, adds edges to keep
//! cluster nodes between boundaries, and ensures the graph is connected.
//!
//! Ported from dagre.js nesting-graph.ts

use std::collections::HashMap;
use crate::graph::Graph;
use super::types::*;
use super::util::{add_border_node, add_dummy_node};

const GRAPH_NODE: &str = "\x00";

/// Run nesting graph transformation for compound graphs.
pub(crate) fn run(g: &mut Graph<NodeLabel, EdgeLabel>) -> Option<String> {
    if !g.is_compound() {
        return None;
    }

    let root = add_dummy_node(g, "root", NodeLabel::default(), "_root");
    let depths = tree_depths(g);
    let height = depths.values().copied().max().unwrap_or(1) - 1;
    let node_sep = 2 * height + 1;

    // Multiply minlen by node_sep to align nodes on non-border ranks
    for e in g.edges() {
        if let Some(label) = g.edge_mut(&e.v, &e.w, e.name.as_deref()) {
            label.minlen *= node_sep as i32;
        }
    }

    // Calculate weight sufficient to keep subgraphs vertically compact
    let weight: i32 = g
        .edges()
        .iter()
        .filter_map(|e| g.edge(&e.v, &e.w, e.name.as_deref()).map(|l| l.weight))
        .sum::<i32>()
        + 1;

    // Process children of root
    let top_children: Vec<String> = g.children(None);
    for child in top_children {
        dfs_nesting(g, &root, node_sep, weight, height, &depths, &child);
    }

    Some(root)
}

fn dfs_nesting(
    g: &mut Graph<NodeLabel, EdgeLabel>,
    root: &str,
    node_sep: usize,
    weight: i32,
    height: usize,
    depths: &HashMap<String, usize>,
    v: &str,
) {
    let children = g.children(Some(v));
    if children.is_empty() {
        if v != root {
            let mut el = EdgeLabel::default();
            el.weight = 0;
            el.minlen = node_sep as i32;
            g.set_edge(root.to_string(), v.to_string(), Some(el), None);
        }
        return;
    }

    let top = add_border_node(g, "_bt", None, None);
    let bottom = add_border_node(g, "_bb", None, None);

    g.set_parent(v, None); // ensure v exists
    g.set_parent(&top, Some(v));
    if let Some(node) = g.node_mut(v) {
        node.border_top = Some(top.clone());
    }

    g.set_parent(&bottom, Some(v));
    if let Some(node) = g.node_mut(v) {
        node.border_bottom = Some(bottom.clone());
    }

    for child in children {
        dfs_nesting(g, root, node_sep, weight, height, depths, &child);

        let child_node = g.node(&child).cloned();
        let child_top = child_node
            .as_ref()
            .and_then(|n| n.border_top.clone())
            .unwrap_or_else(|| child.clone());
        let child_bottom = child_node
            .as_ref()
            .and_then(|n| n.border_bottom.clone())
            .unwrap_or_else(|| child.clone());

        let this_weight = if child_node
            .as_ref()
            .and_then(|n| n.border_top.as_ref())
            .is_some()
        {
            weight
        } else {
            2 * weight
        };

        let minlen = if child_top != child_bottom {
            1
        } else {
            (height - depths.get(v).copied().unwrap_or(0) + 1) as i32
        };

        let mut el_top = EdgeLabel::default();
        el_top.weight = this_weight;
        el_top.minlen = minlen;
        el_top.nesting_edge = true;
        g.set_edge(top.clone(), child_top, Some(el_top), None);

        let mut el_bottom = EdgeLabel::default();
        el_bottom.weight = this_weight;
        el_bottom.minlen = minlen;
        el_bottom.nesting_edge = true;
        g.set_edge(child_bottom, bottom.clone(), Some(el_bottom), None);
    }

    if g.parent(v).is_none() {
        let mut el = EdgeLabel::default();
        el.weight = 0;
        el.minlen = (height + depths.get(v).copied().unwrap_or(0)) as i32;
        g.set_edge(root.to_string(), top, Some(el), None);
    }
}

fn tree_depths(g: &Graph<NodeLabel, EdgeLabel>) -> HashMap<String, usize> {
    let mut depths = HashMap::new();

    fn dfs_depth(
        g: &Graph<NodeLabel, EdgeLabel>,
        v: &str,
        depth: usize,
        depths: &mut HashMap<String, usize>,
    ) {
        let children = g.children(Some(v));
        for child in &children {
            dfs_depth(g, child, depth + 1, depths);
        }
        depths.insert(v.to_string(), depth);
    }

    let top_level = g.children(None);
    for v in top_level {
        dfs_depth(g, &v, 1, &mut depths);
    }
    depths
}

/// Clean up nesting graph artifacts.
pub(crate) fn cleanup(g: &mut Graph<NodeLabel, EdgeLabel>, nesting_root: &str) {
    g.remove_node(nesting_root);

    // Remove nesting edges
    let nesting_edges: Vec<_> = g
        .edges()
        .into_iter()
        .filter(|e| {
            g.edge(&e.v, &e.w, e.name.as_deref())
                .map_or(false, |l| l.nesting_edge)
        })
        .collect();

    for e in nesting_edges {
        g.remove_edge(&e.v, &e.w, e.name.as_deref());
    }
}
