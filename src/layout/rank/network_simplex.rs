//! Network simplex algorithm for optimal rank assignment.
//!
//! This assigns ranks to each node in the input graph and iteratively
//! improves the ranking to reduce the length of edges.
//!
//! Algorithm outline:
//!   1. Assign initial ranks via longest path.
//!   2. Construct a feasible tight tree.
//!   3. Assign DFS low/lim numbering to tree nodes.
//!   4. Compute cut values for tree edges.
//!   5. Iteratively find leave edges (negative cut value), find replacement
//!      enter edges, and exchange them until no negative cut values remain.
//!
//! Derived from Gansner, et al., "A Technique for Drawing Directed Graphs."

use crate::graph::{Edge, Graph};
use crate::layout::rank::feasible_tree::{feasible_tree, TreeEdgeLabel, TreeNodeLabel};
use crate::layout::rank::util::{longest_path, slack};
use crate::layout::types::{EdgeLabel, NodeLabel};
use crate::layout::util::simplify;
use std::collections::HashSet;

/// Runs the network simplex algorithm to assign optimal ranks to each node.
///
/// Pre-conditions:
///   1. The input graph must be a DAG.
///   2. All nodes must have an object value.
///   3. All edges must have "minlen" and "weight" attributes.
///
/// Post-conditions:
///   1. All nodes will have an assigned "rank" that has been optimized
///      by the network simplex algorithm. Ranks start at 0.
pub(crate) fn network_simplex(g: &mut Graph<NodeLabel, EdgeLabel>) {
    // Simplify: merge multi-edges into single edges with combined weights
    let mut sg = simplify(g);

    longest_path(&mut sg);

    let mut tree = feasible_tree(&mut sg);
    init_low_lim_values(&mut tree, None);
    init_cut_values(&mut tree, &sg);

    while let Some(leave) = leave_edge(&tree) {
        let enter = enter_edge(&tree, &sg, &leave);
        exchange_edges(&mut tree, &mut sg, &leave, &enter);
    }

    // Copy ranks back to original graph
    for v in g.nodes() {
        if let Some(sg_label) = sg.node(&v) {
            if let Some(label) = g.node_mut(&v) {
                label.rank = sg_label.rank;
            }
        }
    }
}


// ---------------------------------------------------------------------------
// Cut values
// ---------------------------------------------------------------------------

/// Initializes cut values for all edges in the tree.
fn init_cut_values(
    tree: &mut Graph<TreeNodeLabel, TreeEdgeLabel>,
    g: &Graph<NodeLabel, EdgeLabel>,
) {
    let roots: Vec<String> = tree.nodes();
    let root_refs: Vec<&str> = roots.iter().map(|s| s.as_str()).collect();
    let mut visited = postorder_undirected(tree, &root_refs);
    // Remove the last node (the root) -- we process leaves inward
    visited.pop();

    for v in &visited {
        assign_cut_value(tree, g, v);
    }
}

/// Assigns a cut value to the tree edge between `child` and its parent.
fn assign_cut_value(
    tree: &mut Graph<TreeNodeLabel, TreeEdgeLabel>,
    g: &Graph<NodeLabel, EdgeLabel>,
    child: &str,
) {
    let parent = match tree.node(child).and_then(|n| n.parent.clone()) {
        Some(p) => p,
        None => return,
    };
    let cv = calc_cut_value(tree, g, child);

    // The tree is undirected, so the edge might be stored as (child, parent) or (parent, child)
    if let Some(e) = tree.edge_mut(child, &parent, None) {
        e.cutvalue = Some(cv);
    } else if let Some(e) = tree.edge_mut(&parent, child, None) {
        e.cutvalue = Some(cv);
    }
}

/// Given the tight tree, its graph, and a child in the graph, calculate and
/// return the cut value for the edge between the child and its parent.
fn calc_cut_value(
    tree: &Graph<TreeNodeLabel, TreeEdgeLabel>,
    g: &Graph<NodeLabel, EdgeLabel>,
    child: &str,
) -> i32 {
    let parent = tree
        .node(child)
        .and_then(|n| n.parent.as_deref())
        .unwrap();

    // True if the child is on the tail end of the edge in the directed graph
    let mut child_is_tail = true;
    let graph_edge = match g.edge(child, parent, None) {
        Some(e) => e,
        None => {
            child_is_tail = false;
            g.edge(parent, child, None).unwrap()
        }
    };

    let mut cut_value: i32 = graph_edge.weight;

    let node_edges = g.node_edges(child, None).unwrap_or_default();
    for edge in &node_edges {
        let is_out_edge = edge.v == child;
        let other = if is_out_edge { &edge.w } else { &edge.v };

        if other.as_str() == parent {
            continue;
        }

        let points_to_head = is_out_edge == child_is_tail;
        let other_weight = g
            .edge(&edge.v, &edge.w, edge.name.as_deref())
            .map(|l| l.weight)
            .unwrap_or(1);

        if points_to_head {
            cut_value += other_weight;
        } else {
            cut_value -= other_weight;
        }

        if is_tree_edge(tree, child, other) {
            let other_cut_value = get_tree_edge_cutvalue(tree, child, other);
            if points_to_head {
                cut_value -= other_cut_value;
            } else {
                cut_value += other_cut_value;
            }
        }
    }

    cut_value
}

/// Returns true if the edge (u, v) exists in the tree (undirected).
fn is_tree_edge(tree: &Graph<TreeNodeLabel, TreeEdgeLabel>, u: &str, v: &str) -> bool {
    tree.has_edge(u, v, None)
}

/// Gets the cut value from a tree edge between u and v.
fn get_tree_edge_cutvalue(
    tree: &Graph<TreeNodeLabel, TreeEdgeLabel>,
    u: &str,
    v: &str,
) -> i32 {
    tree.edge(u, v, None)
        .and_then(|e| e.cutvalue)
        .or_else(|| tree.edge(v, u, None).and_then(|e| e.cutvalue))
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Low/lim DFS numbering
// ---------------------------------------------------------------------------

/// Initializes low/lim values for all nodes in the tree via DFS.
fn init_low_lim_values(
    tree: &mut Graph<TreeNodeLabel, TreeEdgeLabel>,
    root: Option<&str>,
) {
    let root_node = match root {
        Some(r) => r.to_string(),
        None => match tree.nodes().first() {
            Some(n) => n.clone(),
            None => return,
        },
    };
    let mut visited = HashSet::new();
    dfs_assign_low_lim(tree, &mut visited, 1, &root_node, None);
}

/// DFS that assigns low, lim, and parent values to tree nodes.
/// Returns the next available lim counter.
fn dfs_assign_low_lim(
    tree: &mut Graph<TreeNodeLabel, TreeEdgeLabel>,
    visited: &mut HashSet<String>,
    next_lim: i32,
    v: &str,
    parent: Option<&str>,
) -> i32 {
    let low = next_lim;
    let mut current_lim = next_lim;

    visited.insert(v.to_string());

    // Get neighbors in the undirected tree
    let neighbors = tree.neighbors(v).unwrap_or_default();
    for w in &neighbors {
        if !visited.contains(w) {
            current_lim = dfs_assign_low_lim(tree, visited, current_lim, w, Some(v));
        }
    }

    if let Some(label) = tree.node_mut(v) {
        label.low = Some(low);
        label.lim = Some(current_lim);
        label.parent = parent.map(|p| p.to_string());
    }

    current_lim + 1
}

// ---------------------------------------------------------------------------
// Leave edge: find a tree edge with negative cut value
// ---------------------------------------------------------------------------

/// Finds a tree edge with a negative cut value, or None if all are >= 0.
fn leave_edge(tree: &Graph<TreeNodeLabel, TreeEdgeLabel>) -> Option<Edge> {
    tree.edges().into_iter().find(|e| {
        let edge = tree.edge(&e.v, &e.w, e.name.as_deref());
        edge.and_then(|el| el.cutvalue).map_or(false, |cv| cv < 0)
    })
}

// ---------------------------------------------------------------------------
// Enter edge: find a non-tree edge to replace the leaving edge
// ---------------------------------------------------------------------------

/// Finds the non-tree edge with minimum slack that should enter the tree.
fn enter_edge(
    tree: &Graph<TreeNodeLabel, TreeEdgeLabel>,
    g: &Graph<NodeLabel, EdgeLabel>,
    edge: &Edge,
) -> Edge {
    let mut v = edge.v.clone();
    let mut w = edge.w.clone();

    // Ensure v is the tail and w is the head in the original directed graph.
    if !g.has_edge(&v, &w, None) {
        std::mem::swap(&mut v, &mut w);
    }

    let v_label = tree.node(&v).unwrap();
    let w_label = tree.node(&w).unwrap();

    let v_lim = v_label.lim.unwrap();
    let w_lim = w_label.lim.unwrap();

    // If the root is in the tail component, flip the descendant check.
    let (tail_label, flip) = if v_lim > w_lim {
        (w_label.clone(), true)
    } else {
        (v_label.clone(), false)
    };

    // Filter candidate edges: those crossing from one component to the other.
    let candidates: Vec<Edge> = g
        .edges()
        .into_iter()
        .filter(|e| {
            let e_v_label = tree.node(&e.v);
            let e_w_label = tree.node(&e.w);
            match (e_v_label, e_w_label) {
                (Some(vl), Some(wl)) => {
                    let v_desc = is_descendant(vl, &tail_label);
                    let w_desc = is_descendant(wl, &tail_label);
                    (flip == v_desc) && (flip != w_desc)
                }
                _ => false,
            }
        })
        .collect();

    // Choose the candidate with minimum slack.
    candidates
        .into_iter()
        .min_by_key(|e| slack(g, e))
        .expect("enter_edge: no candidate edge found")
}

/// Returns true if `v_label` is a descendant of `root_label`
/// per the assigned low/lim attributes.
fn is_descendant(v_label: &TreeNodeLabel, root_label: &TreeNodeLabel) -> bool {
    let root_low = root_label.low.unwrap();
    let root_lim = root_label.lim.unwrap();
    let v_lim = v_label.lim.unwrap();
    root_low <= v_lim && v_lim <= root_lim
}

// ---------------------------------------------------------------------------
// Exchange edges
// ---------------------------------------------------------------------------

/// Exchanges the leaving tree edge with the entering edge, then
/// recalculates low/lim values, cut values, and ranks.
fn exchange_edges(
    tree: &mut Graph<TreeNodeLabel, TreeEdgeLabel>,
    g: &mut Graph<NodeLabel, EdgeLabel>,
    leave: &Edge,
    enter: &Edge,
) {
    tree.remove_edge(&leave.v, &leave.w, None);
    tree.set_edge(
        enter.v.clone(),
        enter.w.clone(),
        Some(TreeEdgeLabel::default()),
        None,
    );
    init_low_lim_values(tree, None);
    init_cut_values(tree, g);
    update_ranks(tree, g);
}

/// Updates ranks of all nodes in the graph based on the current tree structure.
fn update_ranks(
    tree: &Graph<TreeNodeLabel, TreeEdgeLabel>,
    g: &mut Graph<NodeLabel, EdgeLabel>,
) {
    // Find the root: node with no parent in the tree
    let root = tree
        .nodes()
        .into_iter()
        .find(|v| {
            tree.node(v)
                .map_or(true, |n| n.parent.is_none())
        });

    let root = match root {
        Some(r) => r,
        None => return,
    };

    // Preorder traversal of the undirected tree from the root
    let vs = preorder_undirected(tree, &root);

    // Skip the root itself, process children
    for v in vs.iter().skip(1) {
        let parent = match tree.node(v).and_then(|n| n.parent.clone()) {
            Some(p) => p,
            None => continue,
        };

        let mut edge = g.edge(v, &parent, None);
        let mut flipped = false;
        if edge.is_none() {
            edge = g.edge(&parent, v, None);
            flipped = true;
        }

        let minlen = edge.map(|e| e.minlen).unwrap_or(1);
        let parent_rank = g.node(&parent).unwrap().rank.unwrap();

        let new_rank = if flipped {
            parent_rank + minlen
        } else {
            parent_rank - minlen
        };

        g.node_mut(v).unwrap().rank = Some(new_rank);
    }
}

// ---------------------------------------------------------------------------
// Undirected tree traversals (the tree graph is undirected)
// ---------------------------------------------------------------------------

/// Preorder DFS traversal of an undirected graph starting from a single root.
fn preorder_undirected(
    g: &Graph<TreeNodeLabel, TreeEdgeLabel>,
    root: &str,
) -> Vec<String> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();

    fn dfs(
        g: &Graph<TreeNodeLabel, TreeEdgeLabel>,
        v: &str,
        visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) {
        if visited.contains(v) {
            return;
        }
        visited.insert(v.to_string());
        result.push(v.to_string());

        if let Some(neighbors) = g.neighbors(v) {
            for w in neighbors {
                dfs(g, &w, visited, result);
            }
        }
    }

    dfs(g, root, &mut visited, &mut result);
    result
}

/// Postorder DFS traversal of an undirected graph visiting all connected components.
fn postorder_undirected(
    g: &Graph<TreeNodeLabel, TreeEdgeLabel>,
    roots: &[&str],
) -> Vec<String> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();

    fn dfs(
        g: &Graph<TreeNodeLabel, TreeEdgeLabel>,
        v: &str,
        visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) {
        if visited.contains(v) {
            return;
        }
        visited.insert(v.to_string());

        if let Some(neighbors) = g.neighbors(v) {
            for w in neighbors {
                dfs(g, &w, visited, result);
            }
        }

        result.push(v.to_string());
    }

    for root in roots {
        dfs(g, root, &mut visited, &mut result);
    }

    result
}
