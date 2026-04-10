//! Add border segments for compound graph subgraphs.
//!
//! For compound nodes with min_rank/max_rank set, adds border_left and
//! border_right dummy nodes at each rank level and chains them with edges.
//!
//! Ported from dagre.js add-border-segments.ts

use crate::graph::Graph;
use super::types::*;
use super::util::add_border_node;

/// Add border segments (left/right border node chains) for each compound node.
pub(crate) fn add_border_segments(g: &mut Graph<NodeLabel, EdgeLabel>) {
    // Collect compound nodes that have min_rank set (i.e., subgraph nodes)
    let subgraph_nodes: Vec<(String, i32, i32)> = g
        .nodes()
        .into_iter()
        .filter_map(|v| {
            let node = g.node(&v)?;
            let min_rank = node.min_rank?;
            let max_rank = node.max_rank?;
            // Only process nodes that are subgraphs (have children)
            if g.children(Some(&v)).is_empty() {
                return None;
            }
            Some((v, min_rank, max_rank))
        })
        .collect();

    for (v, min_rank, max_rank) in subgraph_nodes {
        for rank in min_rank..=max_rank {
            add_border_node_for(g, &v, BorderType::Left, "_bl", rank);
            add_border_node_for(g, &v, BorderType::Right, "_br", rank);
        }
    }
}

fn add_border_node_for(
    g: &mut Graph<NodeLabel, EdgeLabel>,
    v: &str,
    border_type: BorderType,
    prefix: &str,
    rank: i32,
) {
    let node_id = add_border_node(g, prefix, Some(rank), None);

    // Set the border node's border_type
    if let Some(node) = g.node_mut(&node_id) {
        node.border_type = Some(border_type);
    }

    // Set parent to v
    g.set_parent(&node_id, Some(v));

    // Get previous border node for this side at rank-1 to chain edges
    let prev_border = {
        let node = g.node(v);
        node.and_then(|n| {
            let list = match border_type {
                BorderType::Left => &n.border_left,
                BorderType::Right => &n.border_right,
            };
            list.last().cloned()
        })
    };

    // Add to the border list
    if let Some(node) = g.node_mut(v) {
        match border_type {
            BorderType::Left => node.border_left.push(node_id.clone()),
            BorderType::Right => node.border_right.push(node_id.clone()),
        }
    }

    // Chain edge from previous border node to this one
    if let Some(prev) = prev_border {
        let mut el = EdgeLabel::default();
        el.weight = 1;
        el.minlen = 1;
        g.set_edge(prev, node_id, Some(el), None);
    }
}
