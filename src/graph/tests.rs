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
