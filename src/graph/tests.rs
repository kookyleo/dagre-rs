//! Tests ported from graphlib.js graph-test.ts (170+ tests) and alg/ tests (57+ tests)

use super::*;
use super::alg;

// ============================================================
// Graph - Initial State
// ============================================================

#[test]
fn has_no_nodes_by_default() {
    let g: Graph<i32, i32> = Graph::new();
    assert_eq!(g.node_count(), 0);
}

#[test]
fn has_no_edges_by_default() {
    let g: Graph<i32, i32> = Graph::new();
    assert_eq!(g.edge_count(), 0);
}

#[test]
fn defaults_to_directed() {
    let g: Graph<i32, i32> = Graph::new();
    assert!(g.is_directed());
}

#[test]
fn defaults_to_non_multigraph() {
    let g: Graph<i32, i32> = Graph::new();
    assert!(!g.is_multigraph());
}

#[test]
fn defaults_to_non_compound() {
    let g: Graph<i32, i32> = Graph::new();
    assert!(!g.is_compound());
}

#[test]
fn can_create_undirected_graph() {
    let g: Graph<i32, i32> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    assert!(!g.is_directed());
}

#[test]
fn can_create_multigraph() {
    let g: Graph<i32, i32> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    assert!(g.is_multigraph());
}

#[test]
fn can_create_compound_graph() {
    let g: Graph<i32, i32> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    assert!(g.is_compound());
}

// ============================================================
// Graph - Nodes
// ============================================================

#[test]
fn set_node_creates_node() {
    let mut g: Graph<&str, ()> = Graph::new();
    g.set_node("a", None);
    assert!(g.has_node("a"));
    assert_eq!(g.node_count(), 1);
}

#[test]
fn set_node_with_label() {
    let mut g: Graph<i32, ()> = Graph::new();
    g.set_node("a", Some(42));
    assert_eq!(g.node("a"), Some(&42));
}

#[test]
fn set_node_overwrites_label() {
    let mut g: Graph<i32, ()> = Graph::new();
    g.set_node("a", Some(1));
    g.set_node("a", Some(2));
    assert_eq!(g.node("a"), Some(&2));
    assert_eq!(g.node_count(), 1);
}

#[test]
fn set_node_does_not_overwrite_with_none() {
    let mut g: Graph<i32, ()> = Graph::new();
    g.set_node("a", Some(42));
    g.set_node("a", None);
    assert_eq!(g.node("a"), Some(&42));
}

#[test]
fn set_node_with_default_label() {
    let mut g: Graph<String, ()> = Graph::new();
    g.set_default_node_label(|v| format!("label-{}", v));
    g.set_node("a", None);
    assert_eq!(g.node("a"), Some(&"label-a".to_string()));
}

#[test]
fn node_returns_none_for_nonexistent() {
    let g: Graph<i32, ()> = Graph::new();
    assert_eq!(g.node("a"), None);
}

#[test]
fn remove_node_removes_node() {
    let mut g: Graph<i32, ()> = Graph::new();
    g.set_node("a", Some(1));
    g.remove_node("a");
    assert!(!g.has_node("a"));
    assert_eq!(g.node_count(), 0);
}

#[test]
fn remove_node_returns_label() {
    let mut g: Graph<i32, ()> = Graph::new();
    g.set_node("a", Some(42));
    assert_eq!(g.remove_node("a"), Some(42));
}

#[test]
fn remove_node_removes_incident_edges() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    g.remove_node("b");
    assert_eq!(g.edge_count(), 0);
}

#[test]
fn remove_nonexistent_node_returns_none() {
    let mut g: Graph<i32, ()> = Graph::new();
    assert_eq!(g.remove_node("a"), None);
}

#[test]
fn nodes_returns_all_nodes() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    g.set_node("b", None);
    let mut nodes = g.nodes();
    nodes.sort();
    assert_eq!(nodes, vec!["a", "b"]);
}

// ============================================================
// Graph - Sources and Sinks
// ============================================================

#[test]
fn sources_returns_nodes_with_no_in_edges() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    let mut sources = g.sources();
    sources.sort();
    assert_eq!(sources, vec!["a"]);
}

#[test]
fn sinks_returns_nodes_with_no_out_edges() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    let mut sinks = g.sinks();
    sinks.sort();
    assert_eq!(sinks, vec!["c"]);
}

// ============================================================
// Graph - Edges
// ============================================================

#[test]
fn set_edge_creates_edge() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    assert!(g.has_edge("a", "b", None));
    assert_eq!(g.edge_count(), 1);
}

#[test]
fn set_edge_creates_nodes_if_needed() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    assert!(g.has_node("a"));
    assert!(g.has_node("b"));
}

#[test]
fn set_edge_with_label() {
    let mut g: Graph<(), i32> = Graph::new();
    g.set_edge("a", "b", Some(42), None);
    assert_eq!(g.edge("a", "b", None), Some(&42));
}

#[test]
fn set_edge_overwrites_label() {
    let mut g: Graph<(), i32> = Graph::new();
    g.set_edge("a", "b", Some(1), None);
    g.set_edge("a", "b", Some(2), None);
    assert_eq!(g.edge("a", "b", None), Some(&2));
    assert_eq!(g.edge_count(), 1);
}

#[test]
fn set_edge_does_not_overwrite_with_none() {
    let mut g: Graph<(), i32> = Graph::new();
    g.set_edge("a", "b", Some(42), None);
    g.set_edge("a", "b", None, None);
    assert_eq!(g.edge("a", "b", None), Some(&42));
}

#[test]
fn set_edge_with_default_label() {
    let mut g: Graph<(), String> = Graph::new();
    g.set_default_edge_label(|_| "default".to_string());
    g.set_edge("a", "b", None, None);
    assert_eq!(g.edge("a", "b", None), Some(&"default".to_string()));
}

#[test]
fn edge_returns_none_for_nonexistent() {
    let g: Graph<(), i32> = Graph::new();
    assert_eq!(g.edge("a", "b", None), None);
}

#[test]
fn remove_edge_removes_edge() {
    let mut g: Graph<(), i32> = Graph::new();
    g.set_edge("a", "b", Some(42), None);
    g.remove_edge("a", "b", None);
    assert!(!g.has_edge("a", "b", None));
    assert_eq!(g.edge_count(), 0);
}

#[test]
fn remove_edge_returns_label() {
    let mut g: Graph<(), i32> = Graph::new();
    g.set_edge("a", "b", Some(42), None);
    assert_eq!(g.remove_edge("a", "b", None), Some(42));
}

#[test]
fn remove_edge_keeps_nodes() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.remove_edge("a", "b", None);
    assert!(g.has_node("a"));
    assert!(g.has_node("b"));
}

#[test]
fn edges_returns_all_edges() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    assert_eq!(g.edge_count(), 2);
    assert_eq!(g.edges().len(), 2);
}

// ============================================================
// Graph - Directed edges
// ============================================================

#[test]
fn directed_edge_has_direction() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    assert!(g.has_edge("a", "b", None));
    assert!(!g.has_edge("b", "a", None));
}

#[test]
fn in_edges_returns_incoming() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("c", "b", None, None);
    let in_e = g.in_edges("b", None).unwrap();
    assert_eq!(in_e.len(), 2);
}

#[test]
fn in_edges_filtered_by_source() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("c", "b", None, None);
    let in_e = g.in_edges("b", Some("a")).unwrap();
    assert_eq!(in_e.len(), 1);
    assert_eq!(in_e[0].v, "a");
}

#[test]
fn out_edges_returns_outgoing() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("a", "c", None, None);
    let out_e = g.out_edges("a", None).unwrap();
    assert_eq!(out_e.len(), 2);
}

#[test]
fn out_edges_filtered_by_target() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("a", "c", None, None);
    let out_e = g.out_edges("a", Some("b")).unwrap();
    assert_eq!(out_e.len(), 1);
    assert_eq!(out_e[0].w, "b");
}

#[test]
fn node_edges_returns_all_incident() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("c", "b", None, None);
    let edges = g.node_edges("b", None).unwrap();
    assert_eq!(edges.len(), 2);
}

#[test]
fn predecessors_returns_incoming_nodes() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("c", "b", None, None);
    let mut preds = g.predecessors("b").unwrap();
    preds.sort();
    assert_eq!(preds, vec!["a", "c"]);
}

#[test]
fn successors_returns_outgoing_nodes() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("a", "c", None, None);
    let mut succs = g.successors("a").unwrap();
    succs.sort();
    assert_eq!(succs, vec!["b", "c"]);
}

#[test]
fn neighbors_returns_all_adjacent() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("c", "a", None, None);
    let mut neighbors = g.neighbors("a").unwrap();
    neighbors.sort();
    assert_eq!(neighbors, vec!["b", "c"]);
}

// ============================================================
// Graph - Undirected
// ============================================================

#[test]
fn undirected_edge_both_directions() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_edge("a", "b", None, None);
    assert!(g.has_edge("a", "b", None));
    assert!(g.has_edge("b", "a", None));
}

#[test]
fn undirected_predecessors_and_successors_are_same() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_edge("a", "b", None, None);
    let preds = g.predecessors("b").unwrap();
    let succs = g.successors("b").unwrap();
    assert_eq!(preds, succs);
}

// ============================================================
// Graph - Multigraph
// ============================================================

#[test]
fn multigraph_allows_named_edges() {
    let mut g: Graph<(), i32> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    g.set_edge("a", "b", Some(1), Some("x"));
    g.set_edge("a", "b", Some(2), Some("y"));
    assert_eq!(g.edge("a", "b", Some("x")), Some(&1));
    assert_eq!(g.edge("a", "b", Some("y")), Some(&2));
    assert_eq!(g.edge_count(), 2);
}

#[test]
fn multigraph_remove_named_edge() {
    let mut g: Graph<(), i32> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    g.set_edge("a", "b", Some(1), Some("x"));
    g.set_edge("a", "b", Some(2), Some("y"));
    g.remove_edge("a", "b", Some("x"));
    assert!(!g.has_edge("a", "b", Some("x")));
    assert!(g.has_edge("a", "b", Some("y")));
    assert_eq!(g.edge_count(), 1);
}

// ============================================================
// Graph - Compound
// ============================================================

#[test]
fn compound_set_parent() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_node("a", None);
    g.set_node("b", None);
    g.set_parent("b", Some("a"));
    assert_eq!(g.parent("b"), Some("a"));
}

#[test]
fn compound_children_of_parent() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_node("a", None);
    g.set_node("b", None);
    g.set_node("c", None);
    g.set_parent("b", Some("a"));
    g.set_parent("c", Some("a"));
    let mut children = g.children(Some("a"));
    children.sort();
    assert_eq!(children, vec!["b", "c"]);
}

#[test]
fn compound_top_level_nodes() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_node("a", None);
    g.set_node("b", None);
    g.set_parent("b", Some("a"));
    let mut top = g.children(None);
    top.sort();
    assert_eq!(top, vec!["a"]);
}

#[test]
fn compound_parent_defaults_to_none() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_node("a", None);
    assert_eq!(g.parent("a"), None);
}

#[test]
fn compound_reset_parent_to_root() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_node("a", None);
    g.set_node("b", None);
    g.set_parent("b", Some("a"));
    g.set_parent("b", None);
    assert_eq!(g.parent("b"), None);
    assert!(g.children(Some("a")).is_empty());
}

#[test]
fn compound_remove_node_reparents_children() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_node("a", None);
    g.set_node("b", None);
    g.set_node("c", None);
    g.set_parent("b", Some("a"));
    g.set_parent("c", Some("b"));
    g.remove_node("b");
    // c should be reparented to a
    assert_eq!(g.parent("c"), Some("a"));
}

#[test]
fn compound_set_parent_creates_nodes() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_parent("b", Some("a"));
    assert!(g.has_node("a"));
    assert!(g.has_node("b"));
    assert_eq!(g.parent("b"), Some("a"));
}

// ============================================================
// Graph - set_path
// ============================================================

#[test]
fn set_path_creates_chain() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c"], None);
    assert!(g.has_edge("a", "b", None));
    assert!(g.has_edge("b", "c", None));
    assert!(!g.has_edge("a", "c", None));
}

// ============================================================
// Graph - is_leaf
// ============================================================

#[test]
fn is_leaf_for_sink_node() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    assert!(g.is_leaf("b"));
    assert!(!g.is_leaf("a"));
}

#[test]
fn is_leaf_for_isolated_node() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    assert!(g.is_leaf("a"));
}

// ============================================================
// Graph - filter_nodes
// ============================================================

#[test]
fn filter_nodes_copies_matching_nodes() {
    let mut g: Graph<i32, ()> = Graph::new();
    g.set_node("a", Some(1));
    g.set_node("b", Some(2));
    let g2 = g.filter_nodes(|v| v == "a");
    assert!(g2.has_node("a"));
    assert!(!g2.has_node("b"));
    assert_eq!(g2.node("a"), Some(&1));
}

#[test]
fn filter_nodes_copies_edges_for_matching_nodes() {
    let mut g: Graph<(), i32> = Graph::new();
    g.set_edge("a", "b", Some(1), None);
    g.set_edge("b", "c", Some(2), None);
    let g2 = g.filter_nodes(|v| v != "c");
    assert!(g2.has_edge("a", "b", None));
    assert!(!g2.has_edge("b", "c", None));
}

#[test]
fn filter_nodes_preserves_compound_parents() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_node("a", None);
    g.set_node("b", None);
    g.set_node("c", None);
    g.set_parent("b", Some("a"));
    g.set_parent("c", Some("b"));
    // Filter out "b", "c" should reparent to "a"
    let g2 = g.filter_nodes(|v| v != "b");
    assert_eq!(g2.parent("c"), Some("a"));
}

// ============================================================
// Graph - node_mut
// ============================================================

#[test]
fn node_mut_allows_modification() {
    let mut g: Graph<i32, ()> = Graph::new();
    g.set_node("a", Some(1));
    *g.node_mut("a").unwrap() = 42;
    assert_eq!(g.node("a"), Some(&42));
}

// ============================================================
// Algorithm - Topological Sort
// ============================================================

#[test]
fn topsort_empty_graph() {
    let g: Graph<(), ()> = Graph::new();
    assert_eq!(alg::topsort(&g).unwrap(), Vec::<String>::new());
}

#[test]
fn topsort_simple_chain() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c"], None);
    assert_eq!(alg::topsort(&g).unwrap(), vec!["a", "b", "c"]);
}

#[test]
fn topsort_diamond() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "d"], None);
    g.set_path(&["a", "c", "d"], None);
    let result = alg::topsort(&g).unwrap();
    // a must come before b, c; d must come last
    let pos = |v: &str| result.iter().position(|x| x == v).unwrap();
    assert!(pos("a") < pos("b"));
    assert!(pos("a") < pos("c"));
    assert!(pos("b") < pos("d"));
    assert!(pos("c") < pos("d"));
}

#[test]
fn topsort_detects_cycle() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c", "a"], None);
    assert!(alg::topsort(&g).is_err());
}

#[test]
fn topsort_detects_self_loop() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "a", None, None);
    assert!(alg::topsort(&g).is_err());
}

// ============================================================
// Algorithm - is_acyclic
// ============================================================

#[test]
fn is_acyclic_true_for_dag() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c"], None);
    assert!(alg::is_acyclic(&g));
}

#[test]
fn is_acyclic_false_for_cyclic() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c", "a"], None);
    assert!(!alg::is_acyclic(&g));
}

#[test]
fn is_acyclic_false_for_self_loop() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "a", None, None);
    assert!(!alg::is_acyclic(&g));
}

// ============================================================
// Algorithm - Tarjan (SCC)
// ============================================================

#[test]
fn tarjan_empty_graph() {
    let g: Graph<(), ()> = Graph::new();
    assert!(alg::tarjan(&g).is_empty());
}

#[test]
fn tarjan_single_node() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    let sccs = alg::tarjan(&g);
    assert_eq!(sccs.len(), 1);
    assert_eq!(sccs[0], vec!["a"]);
}

#[test]
fn tarjan_cycle() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c", "a"], None);
    let sccs = alg::tarjan(&g);
    let big_scc: Vec<&Vec<String>> = sccs.iter().filter(|s| s.len() > 1).collect();
    assert_eq!(big_scc.len(), 1);
    let mut scc = big_scc[0].clone();
    scc.sort();
    assert_eq!(scc, vec!["a", "b", "c"]);
}

#[test]
fn tarjan_two_cycles() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "a"], None);
    g.set_path(&["c", "d", "c"], None);
    let sccs = alg::tarjan(&g);
    let mut big_sccs: Vec<Vec<String>> = sccs
        .into_iter()
        .filter(|s| s.len() > 1)
        .map(|mut s| {
            s.sort();
            s
        })
        .collect();
    big_sccs.sort();
    assert_eq!(big_sccs.len(), 2);
    assert_eq!(big_sccs[0], vec!["a", "b"]);
    assert_eq!(big_sccs[1], vec!["c", "d"]);
}

// ============================================================
// Algorithm - find_cycles
// ============================================================

#[test]
fn find_cycles_empty() {
    let g: Graph<(), ()> = Graph::new();
    assert!(alg::find_cycles(&g).is_empty());
}

#[test]
fn find_cycles_acyclic() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c"], None);
    assert!(alg::find_cycles(&g).is_empty());
}

#[test]
fn find_cycles_self_loop() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    g.set_edge("a", "a", None, None);
    let cycles = alg::find_cycles(&g);
    assert_eq!(cycles.len(), 1);
}

// ============================================================
// Algorithm - Components
// ============================================================

#[test]
fn components_empty() {
    let g: Graph<(), ()> = Graph::new();
    assert!(alg::components(&g).is_empty());
}

#[test]
fn components_unconnected_nodes() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    g.set_node("b", None);
    let comps = alg::components(&g);
    assert_eq!(comps.len(), 2);
}

#[test]
fn components_connected() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    let comps = alg::components(&g);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].len(), 3);
}

#[test]
fn components_two_components() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("c", "d", None, None);
    let comps = alg::components(&g);
    assert_eq!(comps.len(), 2);
}

// ============================================================
// Algorithm - DFS / Preorder / Postorder
// ============================================================

#[test]
fn preorder_singleton() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    assert_eq!(alg::preorder(&g, &["a"]), vec!["a"]);
}

#[test]
fn preorder_tree() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("a", "c", None, None);
    let result = alg::preorder(&g, &["a"]);
    assert_eq!(result[0], "a");
    assert_eq!(result.len(), 3);
}

#[test]
fn postorder_singleton() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    assert_eq!(alg::postorder(&g, &["a"]), vec!["a"]);
}

#[test]
fn postorder_tree() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("a", "c", None, None);
    let result = alg::postorder(&g, &["a"]);
    // a should be last in postorder
    assert_eq!(result.last().unwrap(), "a");
    assert_eq!(result.len(), 3);
}

#[test]
fn dfs_visits_each_node_once() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "d"], None);
    g.set_path(&["a", "c", "d"], None);
    let result = alg::preorder(&g, &["a"]);
    assert_eq!(result.len(), 4);
    let unique: std::collections::HashSet<_> = result.iter().collect();
    assert_eq!(unique.len(), 4);
}

// ============================================================
// Algorithm - Dijkstra
// ============================================================

#[test]
fn dijkstra_simple_path() {
    let mut g: Graph<(), f64> = Graph::new();
    g.set_edge("a", "b", Some(1.0), None);
    g.set_edge("b", "c", Some(2.0), None);
    g.set_edge("a", "c", Some(10.0), None);
    let result = alg::dijkstra(&g, "a", |w| *w);
    assert_eq!(result["a"].0, 0.0);
    assert_eq!(result["b"].0, 1.0);
    assert_eq!(result["c"].0, 3.0); // a->b->c = 3, cheaper than a->c = 10
}

// ============================================================
// Algorithm - Prim
// ============================================================

#[test]
fn prim_simple_graph() {
    let mut g: Graph<(), f64> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_edge("a", "b", Some(1.0), None);
    g.set_edge("b", "c", Some(2.0), None);
    g.set_edge("a", "c", Some(5.0), None);
    let mst = alg::prim(&g, |w| *w);
    assert_eq!(mst.node_count(), 3);
    assert_eq!(mst.edge_count(), 2);
    // MST should pick a-b (1) and b-c (2), not a-c (5)
    assert!(mst.has_edge("a", "b", None));
    assert!(mst.has_edge("b", "c", None));
}

// ============================================================
// Graph - Additional setNode / node tests
// ============================================================

#[test]
fn set_node_is_idempotent() {
    let mut g: Graph<&str, ()> = Graph::new();
    g.set_node("a", Some("foo"));
    g.set_node("a", Some("foo"));
    assert_eq!(g.node("a"), Some(&"foo"));
    assert_eq!(g.node_count(), 1);
}

#[test]
fn nodes_empty_graph() {
    let g: Graph<(), ()> = Graph::new();
    assert_eq!(g.nodes(), Vec::<String>::new());
}

#[test]
fn remove_node_is_idempotent() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    g.remove_node("a");
    let result = g.remove_node("a");
    assert_eq!(result, None);
    assert!(!g.has_node("a"));
    assert_eq!(g.node_count(), 0);
}

// ============================================================
// Graph - Additional sources / sinks tests
// ============================================================

#[test]
fn sources_includes_isolated_nodes() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c"], None);
    g.set_node("d", None);
    let mut sources = g.sources();
    sources.sort();
    assert_eq!(sources, vec!["a", "d"]);
}

#[test]
fn sinks_includes_isolated_nodes() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c"], None);
    g.set_node("d", None);
    let mut sinks = g.sinks();
    sinks.sort();
    assert_eq!(sinks, vec!["c", "d"]);
}

// ============================================================
// Graph - Additional filterNodes tests
// ============================================================

#[test]
fn filter_nodes_returns_identical_graph_for_all_pass() {
    let mut g: Graph<i32, i32> = Graph::new();
    g.set_node("a", Some(123));
    g.set_path(&["a", "b", "c"], None);
    g.set_edge("a", "c", Some(456), None);
    let g2 = g.filter_nodes(|_| true);
    let mut nodes = g2.nodes();
    nodes.sort();
    assert_eq!(nodes, vec!["a", "b", "c"]);
    assert!(g2.has_edge("a", "b", None));
    assert!(g2.has_edge("b", "c", None));
    assert!(g2.has_edge("a", "c", None));
    assert_eq!(g2.node("a"), Some(&123));
    assert_eq!(g2.edge("a", "c", None), Some(&456));
}

#[test]
fn filter_nodes_returns_empty_graph_for_none_pass() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c"], None);
    let g2 = g.filter_nodes(|_| false);
    assert_eq!(g2.nodes().len(), 0);
    assert_eq!(g2.edges().len(), 0);
}

#[test]
fn filter_nodes_removes_edges_to_removed_nodes() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    let g2 = g.filter_nodes(|v| v == "a");
    let mut nodes = g2.nodes();
    nodes.sort();
    assert_eq!(nodes, vec!["a"]);
    assert_eq!(g2.edges().len(), 0);
}

#[test]
fn filter_nodes_preserves_directed_option() {
    let g: Graph<(), ()> = Graph::new();
    assert!(g.filter_nodes(|_| true).is_directed());

    let g: Graph<(), ()> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    assert!(!g.filter_nodes(|_| true).is_directed());
}

#[test]
fn filter_nodes_preserves_multigraph_option() {
    let g: Graph<(), ()> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    assert!(g.filter_nodes(|_| true).is_multigraph());

    let g: Graph<(), ()> = Graph::new();
    assert!(!g.filter_nodes(|_| true).is_multigraph());
}

#[test]
fn filter_nodes_preserves_compound_option() {
    let g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    assert!(g.filter_nodes(|_| true).is_compound());

    let g: Graph<(), ()> = Graph::new();
    assert!(!g.filter_nodes(|_| true).is_compound());
}

#[test]
fn filter_nodes_includes_subgraphs() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_parent("a", Some("parent"));
    let g2 = g.filter_nodes(|_| true);
    assert_eq!(g2.parent("a"), Some("parent"));
}

#[test]
fn filter_nodes_includes_multi_level_subgraphs() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_parent("a", Some("parent"));
    g.set_parent("parent", Some("root"));
    let g2 = g.filter_nodes(|_| true);
    assert_eq!(g2.parent("a"), Some("parent"));
    assert_eq!(g2.parent("parent"), Some("root"));
}

#[test]
fn filter_nodes_promotes_child_when_parent_filtered_out() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_parent("a", Some("parent"));
    g.set_parent("parent", Some("root"));
    let g2 = g.filter_nodes(|v| v != "parent");
    assert_eq!(g2.parent("a"), Some("root"));
}

// ============================================================
// Graph - Additional setParent tests
// ============================================================

#[test]
#[should_panic]
fn set_parent_throws_on_non_compound() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_parent("a", Some("b"));
}

#[test]
fn set_parent_moves_node_to_new_parent() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_parent("a", Some("parent"));
    g.set_parent("a", Some("parent2"));
    assert_eq!(g.parent("a"), Some("parent2"));
    assert!(g.children(Some("parent")).is_empty());
    assert_eq!(g.children(Some("parent2")), vec!["a"]);
}

#[test]
fn set_parent_idempotent_remove() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_parent("a", Some("parent"));
    g.set_parent("a", None);
    g.set_parent("a", None);
    assert_eq!(g.parent("a"), None);
    let mut top = g.children(None);
    top.sort();
    assert_eq!(top, vec!["a", "parent"]);
}

// ============================================================
// Graph - Additional parent tests
// ============================================================

#[test]
fn parent_returns_none_for_non_compound() {
    let g: Graph<(), ()> = Graph::new();
    assert_eq!(g.parent("a"), None);
}

#[test]
fn parent_returns_none_for_unknown_node() {
    let g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    assert_eq!(g.parent("a"), None);
}

#[test]
fn parent_returns_current_assignment() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_node("a", None);
    g.set_node("parent", None);
    g.set_parent("a", Some("parent"));
    assert_eq!(g.parent("a"), Some("parent"));
}

// ============================================================
// Graph - Additional children tests
// ============================================================

#[test]
fn children_defaults_to_empty_for_new_compound_node() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_node("a", None);
    assert!(g.children(Some("a")).is_empty());
}

#[test]
fn children_returns_empty_for_non_compound_leaf() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    assert!(g.children(Some("a")).is_empty());
}

#[test]
fn children_returns_all_nodes_for_non_compound_root() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    g.set_node("b", None);
    let mut children = g.children(None);
    children.sort();
    assert_eq!(children, vec!["a", "b"]);
}

#[test]
fn children_of_parent_compound() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_parent("a", Some("parent"));
    g.set_parent("b", Some("parent"));
    let mut children = g.children(Some("parent"));
    children.sort();
    assert_eq!(children, vec!["a", "b"]);
}

#[test]
fn children_without_parent_returns_top_level() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_node("a", None);
    g.set_node("b", None);
    g.set_node("c", None);
    g.set_node("parent", None);
    g.set_parent("a", Some("parent"));
    let mut top = g.children(None);
    top.sort();
    assert_eq!(top, vec!["b", "c", "parent"]);
}

// ============================================================
// Graph - removeNode removes compound relationships
// ============================================================

#[test]
fn remove_node_removes_compound_relationships() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        compound: true,
        ..Default::default()
    });
    g.set_parent("c", Some("b"));
    g.set_parent("b", Some("a"));
    g.remove_node("b");
    assert_eq!(g.parent("b"), None);
    assert!(g.children(Some("b")).is_empty());
    // "a" should no longer list "b" as child
    let a_children = g.children(Some("a"));
    assert!(!a_children.contains(&"b".to_string()));
    // "c" gets reparented to "a"
    assert_eq!(g.parent("c"), Some("a"));
}

// ============================================================
// Graph - Additional predecessors / successors / neighbors tests
// ============================================================

#[test]
fn predecessors_returns_none_for_unknown_node() {
    let g: Graph<(), ()> = Graph::new();
    assert_eq!(g.predecessors("a"), None);
}

#[test]
fn predecessors_includes_self_loops() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    g.set_edge("a", "a", None, None);
    let mut preds = g.predecessors("a").unwrap();
    preds.sort();
    assert_eq!(preds, vec!["a"]);
}

#[test]
fn successors_returns_none_for_unknown_node() {
    let g: Graph<(), ()> = Graph::new();
    assert_eq!(g.successors("a"), None);
}

#[test]
fn successors_includes_self_loops() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    g.set_edge("a", "a", None, None);
    let mut succs = g.successors("a").unwrap();
    succs.sort();
    assert_eq!(succs, vec!["a", "b"]);
}

#[test]
fn successors_returns_empty_for_leaf() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    let succs = g.successors("c").unwrap();
    assert!(succs.is_empty());
}

#[test]
fn neighbors_returns_none_for_unknown_node() {
    let g: Graph<(), ()> = Graph::new();
    assert_eq!(g.neighbors("a"), None);
}

#[test]
fn neighbors_includes_self_loop() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    g.set_edge("a", "a", None, None);
    let mut neighbors = g.neighbors("a").unwrap();
    neighbors.sort();
    assert_eq!(neighbors, vec!["a", "b"]);
}

// ============================================================
// Graph - Additional isLeaf tests
// ============================================================

#[test]
fn is_leaf_false_for_connected_undirected() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_node("a", None);
    g.set_node("b", None);
    g.set_edge("a", "b", None, None);
    assert!(!g.is_leaf("b"));
}

#[test]
fn is_leaf_true_for_unconnected_undirected() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_node("a", None);
    assert!(g.is_leaf("a"));
}

#[test]
fn is_leaf_true_for_unconnected_directed() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    assert!(g.is_leaf("a"));
}

#[test]
fn is_leaf_false_for_predecessor_directed() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    g.set_node("b", None);
    g.set_edge("a", "b", None, None);
    assert!(!g.is_leaf("a"));
}

#[test]
fn is_leaf_true_for_successor_directed() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    g.set_node("b", None);
    g.set_edge("a", "b", None, None);
    assert!(g.is_leaf("b"));
}

// ============================================================
// Graph - Additional edges tests
// ============================================================

#[test]
fn edges_empty_graph() {
    let g: Graph<(), ()> = Graph::new();
    assert!(g.edges().is_empty());
}

// ============================================================
// Graph - Additional setPath tests
// ============================================================

#[test]
fn set_path_with_label() {
    let mut g: Graph<(), i32> = Graph::new();
    g.set_path(&["a", "b", "c"], Some(42));
    assert_eq!(g.edge("a", "b", None), Some(&42));
    assert_eq!(g.edge("b", "c", None), Some(&42));
}

// ============================================================
// Graph - Additional setEdge tests
// ============================================================

#[test]
fn set_edge_creates_multi_edge_only_in_multigraph() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    g.set_edge("a", "b", None, Some("name"));
    assert!(!g.has_edge("a", "b", None));
    assert!(g.has_edge("a", "b", Some("name")));
}

#[test]
#[should_panic]
fn set_edge_panics_for_named_edge_on_non_multigraph() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, Some("name"));
}

#[test]
fn set_edge_directed_treats_opposite_as_distinct() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    assert!(g.has_edge("a", "b", None));
    assert!(!g.has_edge("b", "a", None));
}

#[test]
fn set_edge_undirected_handles_both_directions() {
    let mut g: Graph<(), i32> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_edge("a", "b", Some(42), None);
    assert_eq!(g.edge("a", "b", None), Some(&42));
    assert_eq!(g.edge("b", "a", None), Some(&42));
}

// ============================================================
// Graph - Additional removeEdge tests
// ============================================================

#[test]
fn remove_edge_no_effect_when_absent() {
    let mut g: Graph<(), ()> = Graph::new();
    g.remove_edge("a", "b", None);
    assert!(!g.has_edge("a", "b", None));
    assert_eq!(g.edge_count(), 0);
}

#[test]
fn remove_edge_correctly_removes_neighbors() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.remove_edge("a", "b", None);
    assert_eq!(g.successors("a").unwrap(), Vec::<String>::new());
    assert_eq!(g.neighbors("a").unwrap(), Vec::<String>::new());
    assert_eq!(g.predecessors("b").unwrap(), Vec::<String>::new());
    assert_eq!(g.neighbors("b").unwrap(), Vec::<String>::new());
}

#[test]
fn remove_edge_correctly_decrements_neighbor_count_multigraph() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    g.set_edge("a", "b", None, None);
    g.set_edge("a", "b", None, Some("foo"));
    g.remove_edge("a", "b", None);
    assert!(g.has_edge("a", "b", Some("foo")));
    assert_eq!(g.successors("a").unwrap(), vec!["b"]);
    assert_eq!(g.neighbors("a").unwrap(), vec!["b"]);
    assert_eq!(g.predecessors("b").unwrap(), vec!["a"]);
    assert_eq!(g.neighbors("b").unwrap(), vec!["a"]);
}

#[test]
fn remove_edge_works_with_undirected() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_edge("h", "g", None, None);
    g.remove_edge("g", "h", None);
    assert_eq!(g.neighbors("g").unwrap(), Vec::<String>::new());
    assert_eq!(g.neighbors("h").unwrap(), Vec::<String>::new());
}

// ============================================================
// Graph - Additional inEdges tests
// ============================================================

#[test]
fn in_edges_returns_none_for_unknown_node() {
    let g: Graph<(), ()> = Graph::new();
    assert_eq!(g.in_edges("a", None), None);
}

#[test]
fn in_edges_returns_empty_for_source_node() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    let in_e = g.in_edges("a", None).unwrap();
    assert!(in_e.is_empty());
}

#[test]
fn in_edges_works_for_multigraph() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    g.set_edge("a", "b", None, None);
    g.set_edge("a", "b", None, Some("bar"));
    g.set_edge("a", "b", None, Some("foo"));
    let in_e = g.in_edges("a", None).unwrap();
    assert!(in_e.is_empty());
    let in_b = g.in_edges("b", None).unwrap();
    assert_eq!(in_b.len(), 3);
}

// ============================================================
// Graph - Additional outEdges tests
// ============================================================

#[test]
fn out_edges_returns_none_for_unknown_node() {
    let g: Graph<(), ()> = Graph::new();
    assert_eq!(g.out_edges("a", None), None);
}

#[test]
fn out_edges_returns_all_outgoing() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    let out_a = g.out_edges("a", None).unwrap();
    assert_eq!(out_a.len(), 1);
    assert_eq!(out_a[0].v, "a");
    assert_eq!(out_a[0].w, "b");
    let out_b = g.out_edges("b", None).unwrap();
    assert_eq!(out_b.len(), 1);
    let out_c = g.out_edges("c", None).unwrap();
    assert!(out_c.is_empty());
}

#[test]
fn out_edges_works_for_multigraph() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    g.set_edge("a", "b", None, None);
    g.set_edge("a", "b", None, Some("bar"));
    g.set_edge("a", "b", None, Some("foo"));
    let out_a = g.out_edges("a", None).unwrap();
    assert_eq!(out_a.len(), 3);
    let out_b = g.out_edges("b", None).unwrap();
    assert!(out_b.is_empty());
}

// ============================================================
// Graph - Additional nodeEdges tests
// ============================================================

#[test]
fn node_edges_returns_none_for_unknown_node() {
    let g: Graph<(), ()> = Graph::new();
    assert_eq!(g.node_edges("a", None), None);
}

#[test]
fn node_edges_returns_all_incident_edges() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    let ne_a = g.node_edges("a", None).unwrap();
    assert_eq!(ne_a.len(), 1);
    let ne_b = g.node_edges("b", None).unwrap();
    assert_eq!(ne_b.len(), 2);
    let ne_c = g.node_edges("c", None).unwrap();
    assert_eq!(ne_c.len(), 1);
}

#[test]
fn node_edges_works_for_multigraph() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    g.set_edge("a", "b", None, None);
    g.set_edge("a", "b", None, Some("bar"));
    g.set_edge("a", "b", None, Some("foo"));
    let ne_a = g.node_edges("a", None).unwrap();
    assert_eq!(ne_a.len(), 3);
    let ne_b = g.node_edges("b", None).unwrap();
    assert_eq!(ne_b.len(), 3);
}

// ============================================================
// Graph - Undirected additional tests
// ============================================================

#[test]
fn undirected_in_edges_and_out_edges_are_same() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_edge("a", "b", None, None);
    let in_b = g.in_edges("b", None).unwrap();
    let out_b = g.out_edges("b", None).unwrap();
    assert_eq!(in_b.len(), out_b.len());
}

#[test]
fn undirected_edge_retrieval_either_direction() {
    let mut g: Graph<(), i32> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_edge("a", "b", Some(42), None);
    assert_eq!(g.edge("a", "b", None), Some(&42));
    assert_eq!(g.edge("b", "a", None), Some(&42));
}

#[test]
fn undirected_neighbors_returns_all() {
    let mut g: Graph<(), ()> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_edge("a", "b", None, None);
    g.set_edge("b", "c", None, None);
    let mut neighbors = g.neighbors("b").unwrap();
    neighbors.sort();
    assert_eq!(neighbors, vec!["a", "c"]);
}

// ============================================================
// Graph - setDefaultEdgeLabel tests
// ============================================================

#[test]
fn set_default_edge_label_does_not_change_existing() {
    let mut g: Graph<(), String> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_default_edge_label(|_| "foo".to_string());
    assert_eq!(g.edge("a", "b", None), None);
}

#[test]
fn set_default_edge_label_not_used_with_explicit_value() {
    let mut g: Graph<(), String> = Graph::new();
    g.set_default_edge_label(|_| "foo".to_string());
    g.set_edge("a", "b", Some("bar".to_string()), None);
    assert_eq!(g.edge("a", "b", None), Some(&"bar".to_string()));
}

// ============================================================
// Graph - setDefaultNodeLabel tests
// ============================================================

#[test]
fn set_default_node_label_does_not_change_existing() {
    let mut g: Graph<String, ()> = Graph::new();
    g.set_node("a", None);
    g.set_default_node_label(|_| "foo".to_string());
    // "a" already existed without a value; adding the default label factory does not update it
    assert_eq!(g.node("a"), None);
}

#[test]
fn set_default_node_label_not_used_with_explicit_value() {
    let mut g: Graph<String, ()> = Graph::new();
    g.set_default_node_label(|_| "foo".to_string());
    g.set_node("a", Some("bar".to_string()));
    assert_eq!(g.node("a"), Some(&"bar".to_string()));
}

// ============================================================
// Algorithm - Additional dijkstra tests
// ============================================================

#[test]
fn dijkstra_distance_zero_for_source() {
    let mut g: Graph<(), f64> = Graph::new();
    g.set_node("source", None);
    let result = alg::dijkstra(&g, "source", |w| *w);
    assert_eq!(result["source"].0, 0.0);
}

#[test]
fn dijkstra_infinity_for_unconnected() {
    let mut g: Graph<(), f64> = Graph::new();
    g.set_node("a", None);
    g.set_node("b", None);
    let result = alg::dijkstra(&g, "a", |w| *w);
    assert_eq!(result["a"].0, 0.0);
    assert!(result["b"].0.is_infinite());
}

#[test]
fn dijkstra_distance_and_path() {
    let mut g: Graph<(), f64> = Graph::new();
    g.set_edge("a", "b", Some(1.0), None);
    g.set_edge("b", "c", Some(1.0), None);
    g.set_edge("b", "d", Some(1.0), None);
    let result = alg::dijkstra(&g, "a", |w| *w);
    assert_eq!(result["a"].0, 0.0);
    assert_eq!(result["b"].0, 1.0);
    assert_eq!(result["c"].0, 2.0);
    assert_eq!(result["d"].0, 2.0);
    assert_eq!(result["b"].1, Some("a".to_string()));
    assert_eq!(result["c"].1, Some("b".to_string()));
}

#[test]
fn dijkstra_works_for_undirected() {
    let mut g: Graph<(), f64> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_edge("a", "b", Some(1.0), None);
    g.set_edge("b", "c", Some(2.0), None);
    g.set_edge("b", "d", Some(3.0), None);
    let result = alg::dijkstra(&g, "a", |w| *w);
    assert_eq!(result["a"].0, 0.0);
    assert_eq!(result["b"].0, 1.0);
    assert_eq!(result["c"].0, 3.0);
    assert_eq!(result["d"].0, 4.0);
}

// ============================================================
// Algorithm - Additional Prim tests
// ============================================================

#[test]
fn prim_empty_graph() {
    let g: Graph<(), f64> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    let mst = alg::prim(&g, |w| *w);
    assert_eq!(mst.node_count(), 0);
    assert_eq!(mst.edge_count(), 0);
}

#[test]
fn prim_single_node() {
    let mut g: Graph<(), f64> = Graph::with_options(GraphOptions {
        directed: false,
        ..Default::default()
    });
    g.set_node("a", None);
    let mst = alg::prim(&g, |w| *w);
    assert_eq!(mst.node_count(), 1);
    assert_eq!(mst.edge_count(), 0);
}

// ============================================================
// Algorithm - Additional components tests
// ============================================================

#[test]
fn components_single_node() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_node("a", None);
    let comps = alg::components(&g);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].len(), 1);
}

// ============================================================
// Algorithm - Additional find_cycles tests
// ============================================================

#[test]
fn find_cycles_finds_single_cycle() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c", "a"], None);
    let cycles = alg::find_cycles(&g);
    assert_eq!(cycles.len(), 1);
    let mut cycle = cycles[0].clone();
    cycle.sort();
    assert_eq!(cycle, vec!["a", "b", "c"]);
}

#[test]
fn find_cycles_finds_two_separate_cycles() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "a"], None);
    g.set_path(&["c", "d", "c"], None);
    let mut cycles = alg::find_cycles(&g);
    assert_eq!(cycles.len(), 2);
    for cycle in &mut cycles {
        cycle.sort();
    }
    cycles.sort();
    assert_eq!(cycles[0], vec!["a", "b"]);
    assert_eq!(cycles[1], vec!["c", "d"]);
}

// ============================================================
// Algorithm - Additional tarjan tests
// ============================================================

#[test]
fn tarjan_chain_all_singletons() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_path(&["a", "b", "c"], None);
    let sccs = alg::tarjan(&g);
    assert_eq!(sccs.len(), 3);
    for scc in &sccs {
        assert_eq!(scc.len(), 1);
    }
}

// ============================================================
// Algorithm - Additional preorder / postorder tests
// ============================================================

#[test]
fn preorder_multiple_roots() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("c", "d", None, None);
    let result = alg::preorder(&g, &["a", "c"]);
    assert_eq!(result.len(), 4);
    // a should come before b, c should come before d
    let pos = |v: &str| result.iter().position(|x| x == v).unwrap();
    assert!(pos("a") < pos("b"));
    assert!(pos("c") < pos("d"));
}

#[test]
fn postorder_multiple_roots() {
    let mut g: Graph<(), ()> = Graph::new();
    g.set_edge("a", "b", None, None);
    g.set_edge("c", "d", None, None);
    let result = alg::postorder(&g, &["a", "c"]);
    assert_eq!(result.len(), 4);
    // b should come before a, d should come before c
    let pos = |v: &str| result.iter().position(|x| x == v).unwrap();
    assert!(pos("b") < pos("a"));
    assert!(pos("d") < pos("c"));
}
