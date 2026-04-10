//! Ordering phase: minimize edge crossings via the Sugiyama heuristic.
//!
//! Applies barycenter-based heuristics with alternating up/down sweeps
//! to minimize edge crossings in a layered graph layout.
//!
//! Pre-conditions:
//!   1. Graph must be a DAG
//!   2. Graph nodes must have a `rank` attribute
//!   3. Graph edges must have a `weight` attribute
//!
//! Post-conditions:
//!   1. Graph nodes will have an `order` attribute based on the results.

pub(crate) mod add_subgraph_constraints;
pub(crate) mod barycenter;
pub(crate) mod build_layer_graph;
pub(crate) mod cross_count;
pub(crate) mod init_order;
pub(crate) mod resolve_conflicts;
pub(crate) mod sort;
pub(crate) mod sort_subgraph;

use std::collections::HashMap;

use crate::graph::Graph;
use crate::layout::types::{EdgeLabel, NodeLabel};
use crate::layout::util::{build_layer_matrix, max_rank};

use self::add_subgraph_constraints::add_subgraph_constraints;
use self::build_layer_graph::{build_layer_graph, get_root, Relationship};
use self::cross_count::cross_count;
use self::init_order::init_order;
use self::sort_subgraph::sort_subgraph;

/// Assign `order` to each node, minimizing edge crossings.
///
/// This is the main entry point for the ordering phase.
pub(crate) fn order(g: &mut Graph<NodeLabel, EdgeLabel>) {
    let mr = max_rank(g);
    if mr < 0 {
        return;
    }

    // Build layer graphs for down-sweeps (ranks 1..=maxRank) and
    // up-sweeps (ranks maxRank-1..=0).
    let down_ranks: Vec<i32> = (1..=mr).collect();
    let up_ranks: Vec<i32> = (0..mr).rev().collect();

    let down_layer_graphs = build_layer_graphs(g, &down_ranks, Relationship::InEdges);
    let up_layer_graphs = build_layer_graphs(g, &up_ranks, Relationship::OutEdges);

    // Initial ordering via DFS
    let layering = init_order(g);
    assign_order(g, &layering);

    let mut best_cc = usize::MAX;
    let mut best: Option<Vec<Vec<String>>> = None;
    let mut last_best = 0_usize;

    for i in 0..24 {
        if last_best >= 4 {
            break;
        }

        let layer_graphs = if i % 2 != 0 {
            &down_layer_graphs
        } else {
            &up_layer_graphs
        };
        let bias_right = i % 4 >= 2;

        sweep_layer_graphs(g, layer_graphs, bias_right);

        let layering = build_layer_matrix(g);
        let cc = cross_count(g, &layering);

        if cc < best_cc {
            last_best = 0;
            best = Some(layering);
            best_cc = cc;
        } else {
            last_best += 1;
        }
    }

    if let Some(ref best_layering) = best {
        assign_order(g, best_layering);
    }
}

/// Build layer graphs for all the given ranks, pre-computing which nodes
/// belong to each rank to avoid quadratic scanning.
fn build_layer_graphs(
    g: &Graph<NodeLabel, EdgeLabel>,
    ranks: &[i32],
    relationship: Relationship,
) -> Vec<Graph<NodeLabel, EdgeLabel>> {
    // Build an index: rank -> Vec<node_id>
    let mut nodes_by_rank: HashMap<i32, Vec<String>> = HashMap::new();

    for v in g.nodes() {
        if let Some(node) = g.node(&v) {
            if let Some(rank) = node.rank {
                nodes_by_rank.entry(rank).or_default().push(v.clone());
            }
            // If the node spans multiple ranks (subgraph), add it to each
            if let (Some(min_r), Some(max_r)) = (node.min_rank, node.max_rank) {
                for r in min_r..=max_r {
                    if Some(r) != node.rank {
                        nodes_by_rank.entry(r).or_default().push(v.clone());
                    }
                }
            }
        }
    }

    ranks
        .iter()
        .map(|&rank| {
            let empty = Vec::new();
            let nodes = nodes_by_rank.get(&rank).unwrap_or(&empty);
            build_layer_graph(g, rank, relationship, nodes)
        })
        .collect()
}

/// Execute one sweep across the layer graphs, sorting each layer and
/// accumulating subgraph constraints.
fn sweep_layer_graphs(
    g: &mut Graph<NodeLabel, EdgeLabel>,
    layer_graphs: &[Graph<NodeLabel, EdgeLabel>],
    bias_right: bool,
) {
    let mut cg: Graph<(), ()> = Graph::new();

    for lg in layer_graphs {
        let root = get_root(lg);
        let sorted = sort_subgraph(lg, &root, &cg, bias_right);

        // Assign order values back to the main graph
        for (i, v) in sorted.vs.iter().enumerate() {
            if let Some(node) = g.node_mut(v) {
                node.order = Some(i);
            }
        }

        add_subgraph_constraints(lg, &mut cg, &sorted.vs);
    }
}

/// Write `order` values into the graph from a layering matrix.
fn assign_order(g: &mut Graph<NodeLabel, EdgeLabel>, layering: &[Vec<String>]) {
    for layer in layering {
        for (i, v) in layer.iter().enumerate() {
            if !v.is_empty() {
                if let Some(node) = g.node_mut(v) {
                    node.order = Some(i);
                }
            }
        }
    }
}
