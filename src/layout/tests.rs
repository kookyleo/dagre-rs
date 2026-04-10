//! Layout tests ported from dagre.js test suite.

use super::*;
use crate::graph::{Graph, GraphOptions};

fn make_graph() -> Graph<NodeLabel, EdgeLabel> {
    Graph::with_options(GraphOptions {
        directed: true,
        multigraph: true,
        compound: true,
    })
}

fn set_node(g: &mut Graph<NodeLabel, EdgeLabel>, v: &str, w: f64, h: f64) {
    let mut label = NodeLabel::default();
    label.width = w;
    label.height = h;
    g.set_node(v.to_string(), Some(label));
}

fn set_edge(g: &mut Graph<NodeLabel, EdgeLabel>, v: &str, w: &str) {
    g.set_edge(v.to_string(), w.to_string(), Some(EdgeLabel::default()), None);
}

// ============================================================
// Acyclic module tests
// ============================================================

#[test]
fn acyclic_does_not_change_acyclic_graph() {
    let mut g = make_graph();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_edge(&mut g, "a", "b");

    acyclic::run(&mut g, None);
    assert!(g.has_edge("a", "b", None));
}

#[test]
fn acyclic_run_and_undo_restores_graph() {
    let mut g = make_graph();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "b", "a");

    let orig_edge_count = g.edge_count();
    acyclic::run(&mut g, None);
    acyclic::undo(&mut g);
    assert_eq!(g.edge_count(), orig_edge_count);
}

// ============================================================
// Rank module tests
// ============================================================

#[test]
fn rank_assigns_rank_to_single_node() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    rank::rank(&mut g, Ranker::LongestPath);
    assert!(g.node("a").unwrap().rank.is_some());
}

#[test]
fn rank_assigns_ranks_to_chain() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_node(&mut g, "c", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "b", "c");

    rank::rank(&mut g, Ranker::LongestPath);
    let ra = g.node("a").unwrap().rank.unwrap();
    let rb = g.node("b").unwrap().rank.unwrap();
    let rc = g.node("c").unwrap().rank.unwrap();

    assert!(ra < rb);
    assert!(rb < rc);
}

#[test]
fn rank_respects_minlen() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    let mut el = EdgeLabel::default();
    el.minlen = 3;
    g.set_edge("a".to_string(), "b".to_string(), Some(el), None);

    rank::rank(&mut g, Ranker::LongestPath);
    let ra = g.node("a").unwrap().rank.unwrap();
    let rb = g.node("b").unwrap().rank.unwrap();
    assert!(rb - ra >= 3);
}

// ============================================================
// Normalize module tests
// ============================================================

#[test]
fn normalize_does_not_change_short_edges() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("b").unwrap().rank = Some(1);

    let mut chains = Vec::new();
    normalize::run(&mut g, &mut chains);
    assert!(g.has_edge("a", "b", None));
    assert!(chains.is_empty());
}

#[test]
fn normalize_splits_long_edge() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("b").unwrap().rank = Some(3);

    let mut chains = Vec::new();
    normalize::run(&mut g, &mut chains);

    // Original edge should be gone
    assert!(!g.has_edge("a", "b", None));
    // Should have dummy chains
    assert_eq!(chains.len(), 1);
    // Should have more nodes now (2 dummies for span of 3)
    assert!(g.node_count() > 2);
}

// ============================================================
// Order module tests
// ============================================================

#[test]
fn order_assigns_order_to_nodes() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_node(&mut g, "c", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "a", "c");

    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("b").unwrap().rank = Some(1);
    g.node_mut("c").unwrap().rank = Some(1);

    order::order(&mut g);

    assert!(g.node("a").unwrap().order.is_some());
    assert!(g.node("b").unwrap().order.is_some());
    assert!(g.node("c").unwrap().order.is_some());
}

// ============================================================
// Cross count tests
// ============================================================

#[test]
fn cross_count_zero_for_no_crossings() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_node(&mut g, "c", 10.0, 10.0);
    set_node(&mut g, "d", 10.0, 10.0);
    set_edge(&mut g, "a", "c");
    set_edge(&mut g, "b", "d");

    g.node_mut("a").unwrap().order = Some(0);
    g.node_mut("b").unwrap().order = Some(1);
    g.node_mut("c").unwrap().order = Some(0);
    g.node_mut("d").unwrap().order = Some(1);

    let layering = vec![
        vec!["a".to_string(), "b".to_string()],
        vec!["c".to_string(), "d".to_string()],
    ];

    let cc = order::cross_count::cross_count(&g, &layering);
    assert_eq!(cc, 0);
}

#[test]
fn cross_count_one_crossing() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_node(&mut g, "c", 10.0, 10.0);
    set_node(&mut g, "d", 10.0, 10.0);
    // a->d and b->c creates one crossing
    set_edge(&mut g, "a", "d");
    set_edge(&mut g, "b", "c");

    g.node_mut("a").unwrap().order = Some(0);
    g.node_mut("b").unwrap().order = Some(1);
    g.node_mut("c").unwrap().order = Some(0);
    g.node_mut("d").unwrap().order = Some(1);

    let layering = vec![
        vec!["a".to_string(), "b".to_string()],
        vec!["c".to_string(), "d".to_string()],
    ];

    let cc = order::cross_count::cross_count(&g, &layering);
    assert_eq!(cc, 1);
}

// ============================================================
// Position module tests
// ============================================================

#[test]
fn position_assigns_coordinates() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 20.0);
    set_node(&mut g, "b", 50.0, 20.0);
    set_edge(&mut g, "a", "b");

    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("a").unwrap().order = Some(0);
    g.node_mut("b").unwrap().rank = Some(1);
    g.node_mut("b").unwrap().order = Some(0);

    position::position(&mut g);

    assert!(g.node("a").unwrap().x.is_some());
    assert!(g.node("a").unwrap().y.is_some());
    assert!(g.node("b").unwrap().x.is_some());
    assert!(g.node("b").unwrap().y.is_some());
}

#[test]
fn position_y_increases_with_rank() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 20.0);
    set_node(&mut g, "b", 50.0, 20.0);
    set_edge(&mut g, "a", "b");

    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("a").unwrap().order = Some(0);
    g.node_mut("b").unwrap().rank = Some(1);
    g.node_mut("b").unwrap().order = Some(0);

    position::position(&mut g);

    let ya = g.node("a").unwrap().y.unwrap();
    let yb = g.node("b").unwrap().y.unwrap();
    assert!(yb > ya, "y of rank 1 ({}) should be > y of rank 0 ({})", yb, ya);
}

// ============================================================
// Full layout integration tests
// ============================================================

#[test]
fn layout_single_node() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 100.0);
    layout(&mut g, None);

    let a = g.node("a").unwrap();
    assert!(a.x.is_some(), "node should have x coordinate");
    assert!(a.y.is_some(), "node should have y coordinate");
}

#[test]
fn layout_two_connected_nodes() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 100.0);
    set_node(&mut g, "b", 75.0, 200.0);
    set_edge(&mut g, "a", "b");
    layout(&mut g, None);

    let a = g.node("a").unwrap();
    let b = g.node("b").unwrap();
    assert!(a.x.is_some());
    assert!(a.y.is_some());
    assert!(b.x.is_some());
    assert!(b.y.is_some());

    // a should be above b (smaller y in TB layout)
    assert!(a.y.unwrap() < b.y.unwrap());
}

#[test]
fn layout_diamond_graph() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 50.0);
    set_node(&mut g, "b", 50.0, 50.0);
    set_node(&mut g, "c", 50.0, 50.0);
    set_node(&mut g, "d", 50.0, 50.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "a", "c");
    set_edge(&mut g, "b", "d");
    set_edge(&mut g, "c", "d");
    layout(&mut g, None);

    // All nodes should have coordinates
    for v in &["a", "b", "c", "d"] {
        let n = g.node(v).unwrap();
        assert!(n.x.is_some(), "{} should have x", v);
        assert!(n.y.is_some(), "{} should have y", v);
    }

    // a should be above b, c
    let ya = g.node("a").unwrap().y.unwrap();
    let yb = g.node("b").unwrap().y.unwrap();
    let yc = g.node("c").unwrap().y.unwrap();
    let yd = g.node("d").unwrap().y.unwrap();
    assert!(ya < yb);
    assert!(ya < yc);
    assert!(yb < yd);
    assert!(yc < yd);
}

#[test]
fn layout_chain_graph() {
    let mut g = Graph::new();
    for v in &["a", "b", "c", "d", "e"] {
        set_node(&mut g, v, 30.0, 20.0);
    }
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "b", "c");
    set_edge(&mut g, "c", "d");
    set_edge(&mut g, "d", "e");
    layout(&mut g, None);

    // All y values should be strictly increasing
    let ys: Vec<f64> = ["a", "b", "c", "d", "e"]
        .iter()
        .map(|v| g.node(v).unwrap().y.unwrap())
        .collect();

    for i in 1..ys.len() {
        assert!(
            ys[i] > ys[i - 1],
            "y[{}]={} should be > y[{}]={}",
            i,
            ys[i],
            i - 1,
            ys[i - 1]
        );
    }
}

#[test]
fn layout_handles_cycle() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 30.0, 20.0);
    set_node(&mut g, "b", 30.0, 20.0);
    set_node(&mut g, "c", 30.0, 20.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "b", "c");
    set_edge(&mut g, "c", "a");
    layout(&mut g, None);

    // All nodes should have coordinates despite the cycle
    for v in &["a", "b", "c"] {
        let n = g.node(v).unwrap();
        assert!(n.x.is_some(), "{} should have x", v);
        assert!(n.y.is_some(), "{} should have y", v);
    }
}

#[test]
fn layout_handles_disconnected_components() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 30.0, 20.0);
    set_node(&mut g, "b", 30.0, 20.0);
    set_node(&mut g, "c", 30.0, 20.0);
    set_node(&mut g, "d", 30.0, 20.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "c", "d");
    layout(&mut g, None);

    for v in &["a", "b", "c", "d"] {
        let n = g.node(v).unwrap();
        assert!(n.x.is_some(), "{} should have x", v);
        assert!(n.y.is_some(), "{} should have y", v);
    }
}

// ============================================================
// Utility tests
// ============================================================

#[test]
fn intersect_rect_top() {
    let mut rect = NodeLabel::default();
    rect.x = Some(0.0);
    rect.y = Some(0.0);
    rect.width = 100.0;
    rect.height = 100.0;

    let point = Point::new(0.0, -200.0);
    let result = util::intersect_rect(&rect, &point);
    assert!((result.x - 0.0).abs() < 0.01);
    assert!((result.y - (-50.0)).abs() < 0.01);
}

#[test]
fn intersect_rect_right() {
    let mut rect = NodeLabel::default();
    rect.x = Some(0.0);
    rect.y = Some(0.0);
    rect.width = 100.0;
    rect.height = 100.0;

    let point = Point::new(200.0, 0.0);
    let result = util::intersect_rect(&rect, &point);
    assert!((result.x - 50.0).abs() < 0.01);
    assert!((result.y - 0.0).abs() < 0.01);
}

#[test]
fn build_layer_matrix_produces_correct_layers() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::new();
    for (v, rank, order) in &[("a", 0, 0), ("b", 0, 1), ("c", 1, 0)] {
        let mut label = NodeLabel::default();
        label.rank = Some(*rank);
        label.order = Some(*order);
        g.set_node(v.to_string(), Some(label));
    }

    let layers = util::build_layer_matrix(&g);
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0], vec!["a", "b"]);
    assert_eq!(layers[1], vec!["c"]);
}

#[test]
fn normalize_ranks_shifts_to_zero() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::new();
    for (v, rank) in &[("a", 5), ("b", 7), ("c", 9)] {
        let mut label = NodeLabel::default();
        label.rank = Some(*rank);
        g.set_node(v.to_string(), Some(label));
    }

    util::normalize_ranks(&mut g);
    assert_eq!(g.node("a").unwrap().rank, Some(0));
    assert_eq!(g.node("b").unwrap().rank, Some(2));
    assert_eq!(g.node("c").unwrap().rank, Some(4));
}

// ============================================================
// Additional acyclic tests
// ============================================================

#[test]
fn acyclic_does_not_change_diamond_acyclic_graph() {
    let mut g = make_graph();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_node(&mut g, "c", 10.0, 10.0);
    set_node(&mut g, "d", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "a", "c");
    set_edge(&mut g, "b", "d");
    set_edge(&mut g, "c", "d");

    acyclic::run(&mut g, None);

    assert!(g.has_edge("a", "b", None));
    assert!(g.has_edge("a", "c", None));
    assert!(g.has_edge("b", "d", None));
    assert!(g.has_edge("c", "d", None));
}

#[test]
fn acyclic_breaks_cycles() {
    let mut g = make_graph();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_node(&mut g, "c", 10.0, 10.0);
    set_node(&mut g, "d", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "b", "c");
    set_edge(&mut g, "c", "d");
    set_edge(&mut g, "d", "a");

    acyclic::run(&mut g, None);

    // After acyclic run, graph should have no cycles
    let cycles = crate::graph::alg::find_cycles(&g);
    assert!(cycles.is_empty(), "graph should be acyclic after run, found cycles: {:?}", cycles);
}

#[test]
fn acyclic_undo_preserves_edge_labels() {
    let mut g = make_graph();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    let mut el = EdgeLabel::default();
    el.minlen = 2;
    el.weight = 3;
    g.set_edge("a".to_string(), "b".to_string(), Some(el), None);

    acyclic::run(&mut g, None);
    acyclic::undo(&mut g);

    let label = g.edge("a", "b", None).unwrap();
    assert_eq!(label.minlen, 2);
    assert_eq!(label.weight, 3);
}

#[test]
fn acyclic_run_and_undo_restores_reversed_edges() {
    let mut g = make_graph();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    let mut el_ab = EdgeLabel::default();
    el_ab.minlen = 2;
    el_ab.weight = 3;
    g.set_edge("a".to_string(), "b".to_string(), Some(el_ab), None);
    let mut el_ba = EdgeLabel::default();
    el_ba.minlen = 3;
    el_ba.weight = 4;
    g.set_edge("b".to_string(), "a".to_string(), Some(el_ba), None);

    acyclic::run(&mut g, None);
    acyclic::undo(&mut g);

    let ab = g.edge("a", "b", None).unwrap();
    assert_eq!(ab.minlen, 2);
    assert_eq!(ab.weight, 3);
    let ba = g.edge("b", "a", None).unwrap();
    assert_eq!(ba.minlen, 3);
    assert_eq!(ba.weight, 4);
    assert_eq!(g.edge_count(), 2);
}

// ============================================================
// Additional normalize tests
// ============================================================

#[test]
fn normalize_splits_two_layer_edge_into_segments() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("b").unwrap().rank = Some(2);

    let mut chains = Vec::new();
    normalize::run(&mut g, &mut chains);

    // a should have exactly 1 successor (a dummy node)
    let succs = g.successors("a").unwrap();
    assert_eq!(succs.len(), 1);
    let dummy = &succs[0];
    let dummy_node = g.node(dummy).unwrap();
    assert_eq!(dummy_node.dummy.as_deref(), Some("edge"));
    assert_eq!(dummy_node.rank, Some(1));

    // The dummy should point to b
    let dummy_succs = g.successors(dummy).unwrap();
    assert_eq!(dummy_succs, vec!["b"]);

    assert_eq!(chains.len(), 1);
    assert_eq!(chains[0], *dummy);
}

#[test]
fn normalize_assigns_zero_dims_to_dummy_nodes() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    let mut el = EdgeLabel::default();
    el.width = 10.0;
    el.height = 10.0;
    g.set_edge("a".to_string(), "b".to_string(), Some(el), None);
    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("b").unwrap().rank = Some(2);

    let mut chains = Vec::new();
    normalize::run(&mut g, &mut chains);

    let succs = g.successors("a").unwrap();
    assert_eq!(succs.len(), 1);
    let dummy_node = g.node(&succs[0]).unwrap();
    assert_eq!(dummy_node.width, 0.0);
    assert_eq!(dummy_node.height, 0.0);
}

#[test]
fn normalize_preserves_edge_weight() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    let mut el = EdgeLabel::default();
    el.weight = 2;
    g.set_edge("a".to_string(), "b".to_string(), Some(el), None);
    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("b").unwrap().rank = Some(2);

    let mut chains = Vec::new();
    normalize::run(&mut g, &mut chains);

    let succs = g.successors("a").unwrap();
    assert_eq!(succs.len(), 1);
    let edge_label = g.edge("a", &succs[0], None).unwrap();
    assert_eq!(edge_label.weight, 2);
}

#[test]
fn normalize_undo_reverses_run() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("b").unwrap().rank = Some(2);

    let mut chains = Vec::new();
    normalize::run(&mut g, &mut chains);
    normalize::undo(&mut g, &chains);

    // Original edge should be restored
    assert!(g.has_edge("a", "b", None));
    assert_eq!(g.node("a").unwrap().rank, Some(0));
    assert_eq!(g.node("b").unwrap().rank, Some(2));
}

#[test]
fn normalize_undo_collects_points() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("b").unwrap().rank = Some(2);

    let mut chains = Vec::new();
    normalize::run(&mut g, &mut chains);

    // Assign coordinates to the dummy node
    let dummy_id = {
        let succs = g.successors("a").unwrap();
        succs[0].clone()
    };
    g.node_mut(&dummy_id).unwrap().x = Some(5.0);
    g.node_mut(&dummy_id).unwrap().y = Some(10.0);

    normalize::undo(&mut g, &chains);

    let edge_label = g.edge("a", "b", None).unwrap();
    assert_eq!(edge_label.points.len(), 1);
    assert_eq!(edge_label.points[0].x, 5.0);
    assert_eq!(edge_label.points[0].y, 10.0);
}

#[test]
fn normalize_undo_merges_multiple_points() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("b").unwrap().rank = Some(4);

    let mut chains = Vec::new();
    normalize::run(&mut g, &mut chains);

    // Assign coordinates to the 3 dummy nodes
    let succs_a = g.successors("a").unwrap();
    assert_eq!(succs_a.len(), 1);
    let d1 = succs_a[0].clone();
    g.node_mut(&d1).unwrap().x = Some(5.0);
    g.node_mut(&d1).unwrap().y = Some(10.0);

    let succs_d1 = g.successors(&d1).unwrap();
    assert_eq!(succs_d1.len(), 1);
    let d2 = succs_d1[0].clone();
    g.node_mut(&d2).unwrap().x = Some(20.0);
    g.node_mut(&d2).unwrap().y = Some(25.0);

    let succs_d2 = g.successors(&d2).unwrap();
    assert_eq!(succs_d2.len(), 1);
    let d3 = succs_d2[0].clone();
    g.node_mut(&d3).unwrap().x = Some(100.0);
    g.node_mut(&d3).unwrap().y = Some(200.0);

    normalize::undo(&mut g, &chains);

    let edge_label = g.edge("a", "b", None).unwrap();
    assert_eq!(edge_label.points.len(), 3);
    assert_eq!(edge_label.points[0].x, 5.0);
    assert_eq!(edge_label.points[0].y, 10.0);
    assert_eq!(edge_label.points[1].x, 20.0);
    assert_eq!(edge_label.points[1].y, 25.0);
    assert_eq!(edge_label.points[2].x, 100.0);
    assert_eq!(edge_label.points[2].y, 200.0);
}

// ============================================================
// Additional rank tests
// ============================================================

#[test]
fn rank_network_simplex_single_node() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    rank::rank(&mut g, Ranker::NetworkSimplex);
    util::normalize_ranks(&mut g);
    assert_eq!(g.node("a").unwrap().rank, Some(0));
}

#[test]
fn rank_network_simplex_two_nodes() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    rank::rank(&mut g, Ranker::NetworkSimplex);
    util::normalize_ranks(&mut g);
    assert_eq!(g.node("a").unwrap().rank, Some(0));
    assert_eq!(g.node("b").unwrap().rank, Some(1));
}

#[test]
fn rank_network_simplex_diamond() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_node(&mut g, "c", 10.0, 10.0);
    set_node(&mut g, "d", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "a", "c");
    set_edge(&mut g, "b", "d");
    set_edge(&mut g, "c", "d");

    rank::rank(&mut g, Ranker::NetworkSimplex);
    util::normalize_ranks(&mut g);

    let ra = g.node("a").unwrap().rank.unwrap();
    let rb = g.node("b").unwrap().rank.unwrap();
    let rc = g.node("c").unwrap().rank.unwrap();
    let rd = g.node("d").unwrap().rank.unwrap();
    assert_eq!(ra, 0);
    assert_eq!(rb, 1);
    assert_eq!(rc, 1);
    assert_eq!(rd, 2);
}

#[test]
fn rank_network_simplex_uses_minlen() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_node(&mut g, "c", 10.0, 10.0);
    set_node(&mut g, "d", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    let mut el = EdgeLabel::default();
    el.minlen = 2;
    g.set_edge("a".to_string(), "c".to_string(), Some(EdgeLabel::default()), None);
    g.set_edge("c".to_string(), "d".to_string(), Some(el), None);
    set_edge(&mut g, "b", "d");

    rank::rank(&mut g, Ranker::NetworkSimplex);
    util::normalize_ranks(&mut g);

    let ra = g.node("a").unwrap().rank.unwrap();
    let rc = g.node("c").unwrap().rank.unwrap();
    let rd = g.node("d").unwrap().rank.unwrap();
    assert!(rd - rc >= 2, "d.rank - c.rank should be >= 2, got {} - {} = {}", rd, rc, rd - rc);
    assert_eq!(ra, 0);
}

#[test]
fn rank_network_simplex_gansner_graph() {
    let mut g = Graph::new();
    for v in &["a", "b", "c", "d", "e", "f", "g", "h"] {
        set_node(&mut g, v, 10.0, 10.0);
    }
    // a -> b -> c -> d -> h
    // a -> e -> g -> h
    // a -> f -> g
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "b", "c");
    set_edge(&mut g, "c", "d");
    set_edge(&mut g, "d", "h");
    set_edge(&mut g, "a", "e");
    set_edge(&mut g, "e", "g");
    set_edge(&mut g, "g", "h");
    set_edge(&mut g, "a", "f");
    set_edge(&mut g, "f", "g");

    rank::rank(&mut g, Ranker::NetworkSimplex);
    util::normalize_ranks(&mut g);

    assert_eq!(g.node("a").unwrap().rank, Some(0));
    assert_eq!(g.node("b").unwrap().rank, Some(1));
    assert_eq!(g.node("c").unwrap().rank, Some(2));
    assert_eq!(g.node("d").unwrap().rank, Some(3));
    assert_eq!(g.node("h").unwrap().rank, Some(4));
    assert_eq!(g.node("e").unwrap().rank, Some(1));
    assert_eq!(g.node("f").unwrap().rank, Some(1));
    assert_eq!(g.node("g").unwrap().rank, Some(2));
}

// ============================================================
// Additional cross count tests
// ============================================================

#[test]
fn cross_count_zero_for_empty_layering() {
    let g: Graph<NodeLabel, EdgeLabel> = Graph::new();
    let cc = order::cross_count::cross_count(&g, &[]);
    assert_eq!(cc, 0);
}

#[test]
fn cross_count_weighted_crossing() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::new();
    set_node(&mut g, "a1", 10.0, 10.0);
    set_node(&mut g, "a2", 10.0, 10.0);
    set_node(&mut g, "b1", 10.0, 10.0);
    set_node(&mut g, "b2", 10.0, 10.0);
    let mut el1 = EdgeLabel::default();
    el1.weight = 2;
    g.set_edge("a1".to_string(), "b1".to_string(), Some(el1), None);
    let mut el2 = EdgeLabel::default();
    el2.weight = 3;
    g.set_edge("a2".to_string(), "b2".to_string(), Some(el2), None);

    g.node_mut("a1").unwrap().order = Some(0);
    g.node_mut("a2").unwrap().order = Some(1);
    g.node_mut("b1").unwrap().order = Some(1);
    g.node_mut("b2").unwrap().order = Some(0);

    let layering = vec![
        vec!["a1".to_string(), "a2".to_string()],
        vec!["b2".to_string(), "b1".to_string()],
    ];

    let cc = order::cross_count::cross_count(&g, &layering);
    assert_eq!(cc, 6);
}

#[test]
fn cross_count_across_multiple_layers() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::new();
    for v in &["a1", "a2", "b1", "b2", "c1", "c2"] {
        set_node(&mut g, v, 10.0, 10.0);
    }
    set_edge(&mut g, "a1", "b1");
    set_edge(&mut g, "b1", "c1");
    set_edge(&mut g, "a2", "b2");
    set_edge(&mut g, "b2", "c2");

    g.node_mut("a1").unwrap().order = Some(0);
    g.node_mut("a2").unwrap().order = Some(1);
    g.node_mut("b1").unwrap().order = Some(1);
    g.node_mut("b2").unwrap().order = Some(0);
    g.node_mut("c1").unwrap().order = Some(0);
    g.node_mut("c2").unwrap().order = Some(1);

    let layering = vec![
        vec!["a1".to_string(), "a2".to_string()],
        vec!["b2".to_string(), "b1".to_string()],
        vec!["c1".to_string(), "c2".to_string()],
    ];

    let cc = order::cross_count::cross_count(&g, &layering);
    // 1 crossing in layer 0->1, 1 crossing in layer 1->2
    assert_eq!(cc, 2);
}

#[test]
fn cross_count_graph_one() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::new();
    for v in &["a", "b", "c", "d", "e", "f", "i"] {
        set_node(&mut g, v, 10.0, 10.0);
    }
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "b", "c");
    set_edge(&mut g, "d", "e");
    set_edge(&mut g, "e", "c");
    set_edge(&mut g, "a", "f");
    set_edge(&mut g, "f", "i");
    set_edge(&mut g, "a", "e");

    // First ordering: [a, d], [b, e, f], [c, i]
    g.node_mut("a").unwrap().order = Some(0);
    g.node_mut("d").unwrap().order = Some(1);
    g.node_mut("b").unwrap().order = Some(0);
    g.node_mut("e").unwrap().order = Some(1);
    g.node_mut("f").unwrap().order = Some(2);
    g.node_mut("c").unwrap().order = Some(0);
    g.node_mut("i").unwrap().order = Some(1);

    let layering = vec![
        vec!["a".to_string(), "d".to_string()],
        vec!["b".to_string(), "e".to_string(), "f".to_string()],
        vec!["c".to_string(), "i".to_string()],
    ];

    let cc = order::cross_count::cross_count(&g, &layering);
    assert_eq!(cc, 1);

    // Second ordering: [d, a], [e, b, f], [c, i] - should have 0 crossings
    g.node_mut("a").unwrap().order = Some(1);
    g.node_mut("d").unwrap().order = Some(0);
    g.node_mut("b").unwrap().order = Some(1);
    g.node_mut("e").unwrap().order = Some(0);
    g.node_mut("f").unwrap().order = Some(2);
    g.node_mut("c").unwrap().order = Some(0);
    g.node_mut("i").unwrap().order = Some(1);

    let layering2 = vec![
        vec!["d".to_string(), "a".to_string()],
        vec!["e".to_string(), "b".to_string(), "f".to_string()],
        vec!["c".to_string(), "i".to_string()],
    ];

    let cc2 = order::cross_count::cross_count(&g, &layering2);
    assert_eq!(cc2, 0);
}

// ============================================================
// Additional utility tests - simplify
// ============================================================

#[test]
fn simplify_copies_no_multiedge() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    let mut el = EdgeLabel::default();
    el.weight = 1;
    el.minlen = 1;
    g.set_edge("a".to_string(), "b".to_string(), Some(el), None);

    let g2 = util::simplify(&g);
    let label = g2.edge("a", "b", None).unwrap();
    assert_eq!(label.weight, 1);
    assert_eq!(label.minlen, 1);
    assert_eq!(g2.edge_count(), 1);
}

#[test]
fn simplify_collapses_multiedges() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    let mut el1 = EdgeLabel::default();
    el1.weight = 1;
    el1.minlen = 1;
    g.set_edge("a".to_string(), "b".to_string(), Some(el1), None);
    let mut el2 = EdgeLabel::default();
    el2.weight = 2;
    el2.minlen = 2;
    g.set_edge("a".to_string(), "b".to_string(), Some(el2), Some("multi"));

    let g2 = util::simplify(&g);
    assert!(!g2.is_multigraph());
    let label = g2.edge("a", "b", None).unwrap();
    assert_eq!(label.weight, 3);
    assert_eq!(label.minlen, 2);
    assert_eq!(g2.edge_count(), 1);
}

// ============================================================
// Additional utility tests - asNonCompoundGraph
// ============================================================

#[test]
fn as_non_compound_copies_all_nodes() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::with_options(GraphOptions {
        compound: true,
        multigraph: true,
        ..Default::default()
    });
    set_node(&mut g, "a", 50.0, 20.0);
    set_node(&mut g, "b", 30.0, 10.0);

    let g2 = util::as_non_compound_graph(&g);
    assert!(g2.has_node("a"));
    assert!(g2.has_node("b"));
}

#[test]
fn as_non_compound_copies_all_edges() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::with_options(GraphOptions {
        compound: true,
        multigraph: true,
        ..Default::default()
    });
    set_node(&mut g, "a", 50.0, 20.0);
    set_node(&mut g, "b", 30.0, 10.0);
    set_edge(&mut g, "a", "b");

    let g2 = util::as_non_compound_graph(&g);
    assert!(g2.has_edge("a", "b", None));
}

#[test]
fn as_non_compound_does_not_copy_compound_nodes() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::with_options(GraphOptions {
        compound: true,
        multigraph: true,
        ..Default::default()
    });
    set_node(&mut g, "a", 50.0, 20.0);
    g.set_parent("a", Some("sg1"));

    let g2 = util::as_non_compound_graph(&g);
    assert!(!g2.is_compound());
    assert!(g2.has_node("a"));
    // sg1 has children, so it should NOT be in the non-compound graph
    // (as_non_compound_graph skips nodes that have children)
    assert!(!g2.has_node("sg1"));
}

// ============================================================
// Additional utility tests - successorWeights / predecessorWeights
// ============================================================

#[test]
fn successor_weights_maps_correctly() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_node(&mut g, "c", 10.0, 10.0);
    set_node(&mut g, "d", 10.0, 10.0);
    let mut el1 = EdgeLabel::default();
    el1.weight = 2;
    g.set_edge("a".to_string(), "b".to_string(), Some(el1), None);
    let mut el2 = EdgeLabel::default();
    el2.weight = 1;
    g.set_edge("b".to_string(), "c".to_string(), Some(el2), None);
    let mut el3 = EdgeLabel::default();
    el3.weight = 2;
    g.set_edge("b".to_string(), "c".to_string(), Some(el3), Some("multi"));
    let mut el4 = EdgeLabel::default();
    el4.weight = 1;
    g.set_edge("b".to_string(), "d".to_string(), Some(el4), Some("multi"));

    let sw = util::successor_weights(&g);
    assert_eq!(sw["a"]["b"], 2);
    assert_eq!(sw["b"]["c"], 3);
    assert_eq!(sw["b"]["d"], 1);
    assert!(sw["c"].is_empty());
    assert!(sw["d"].is_empty());
}

#[test]
fn predecessor_weights_maps_correctly() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::with_options(GraphOptions {
        multigraph: true,
        ..Default::default()
    });
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_node(&mut g, "c", 10.0, 10.0);
    set_node(&mut g, "d", 10.0, 10.0);
    let mut el1 = EdgeLabel::default();
    el1.weight = 2;
    g.set_edge("a".to_string(), "b".to_string(), Some(el1), None);
    let mut el2 = EdgeLabel::default();
    el2.weight = 1;
    g.set_edge("b".to_string(), "c".to_string(), Some(el2), None);
    let mut el3 = EdgeLabel::default();
    el3.weight = 2;
    g.set_edge("b".to_string(), "c".to_string(), Some(el3), Some("multi"));
    let mut el4 = EdgeLabel::default();
    el4.weight = 1;
    g.set_edge("b".to_string(), "d".to_string(), Some(el4), Some("multi"));

    let pw = util::predecessor_weights(&g);
    assert!(pw["a"].is_empty());
    assert_eq!(pw["b"]["a"], 2);
    assert_eq!(pw["c"]["b"], 3);
    assert_eq!(pw["d"]["b"], 1);
}

// ============================================================
// Additional utility tests - intersectRect
// ============================================================

#[test]
fn intersect_rect_bottom() {
    let mut rect = NodeLabel::default();
    rect.x = Some(0.0);
    rect.y = Some(0.0);
    rect.width = 100.0;
    rect.height = 100.0;

    let point = Point::new(0.0, 200.0);
    let result = util::intersect_rect(&rect, &point);
    assert!((result.x - 0.0).abs() < 0.01);
    assert!((result.y - 50.0).abs() < 0.01);
}

#[test]
fn intersect_rect_left() {
    let mut rect = NodeLabel::default();
    rect.x = Some(0.0);
    rect.y = Some(0.0);
    rect.width = 100.0;
    rect.height = 100.0;

    let point = Point::new(-200.0, 0.0);
    let result = util::intersect_rect(&rect, &point);
    assert!((result.x - (-50.0)).abs() < 0.01);
    assert!((result.y - 0.0).abs() < 0.01);
}

#[test]
fn intersect_rect_touches_border() {
    let mut rect = NodeLabel::default();
    rect.x = Some(0.0);
    rect.y = Some(0.0);
    rect.width = 1.0;
    rect.height = 1.0;

    // Test various points and verify the result touches the border
    for &(px, py) in &[(2.0, 6.0), (2.0, -6.0), (6.0, 2.0), (-6.0, 2.0), (5.0, 0.0), (0.0, 5.0)] {
        let point = Point::new(px, py);
        let cross = util::intersect_rect(&rect, &point);
        // Either x or y should be at the border
        let at_x_border = (cross.x.abs() - 0.5).abs() < 0.01;
        let at_y_border = (cross.y.abs() - 0.5).abs() < 0.01;
        assert!(at_x_border || at_y_border,
            "Point ({}, {}) => ({}, {}) should touch border",
            px, py, cross.x, cross.y);
    }
}

// ============================================================
// Additional utility tests - buildLayerMatrix
// ============================================================

#[test]
fn build_layer_matrix_with_three_layers() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::new();
    for (v, rank, order) in &[
        ("a", 0, 0), ("b", 0, 1),
        ("c", 1, 0), ("d", 1, 1),
        ("e", 2, 0),
    ] {
        let mut label = NodeLabel::default();
        label.rank = Some(*rank);
        label.order = Some(*order);
        g.set_node(v.to_string(), Some(label));
    }

    let layers = util::build_layer_matrix(&g);
    assert_eq!(layers.len(), 3);
    assert_eq!(layers[0], vec!["a", "b"]);
    assert_eq!(layers[1], vec!["c", "d"]);
    assert_eq!(layers[2], vec!["e"]);
}

// ============================================================
// Additional utility tests - normalizeRanks
// ============================================================

#[test]
fn normalize_ranks_works_for_negative_ranks() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::new();
    for (v, rank) in &[("a", -3), ("b", -2)] {
        let mut label = NodeLabel::default();
        label.rank = Some(*rank);
        g.set_node(v.to_string(), Some(label));
    }

    util::normalize_ranks(&mut g);
    assert_eq!(g.node("a").unwrap().rank, Some(0));
    assert_eq!(g.node("b").unwrap().rank, Some(1));
}

// ============================================================
// Additional utility tests - removeEmptyRanks
// ============================================================

#[test]
fn remove_empty_ranks_removes_border_ranks() {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::new();
    let mut a = NodeLabel::default();
    a.rank = Some(0);
    g.set_node("a".to_string(), Some(a));
    let mut b = NodeLabel::default();
    b.rank = Some(4);
    g.set_node("b".to_string(), Some(b));

    util::remove_empty_ranks(&mut g);
    assert_eq!(g.node("a").unwrap().rank, Some(0));
    assert_eq!(g.node("b").unwrap().rank, Some(1));
}

// ============================================================
// Additional full layout integration tests
// ============================================================

#[test]
fn layout_self_loop() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 100.0, 100.0);
    set_edge(&mut g, "a", "a");
    layout(&mut g, None);

    let a = g.node("a").unwrap();
    assert!(a.x.is_some(), "node should have x coordinate");
    assert!(a.y.is_some(), "node should have y coordinate");
}

#[test]
fn layout_wide_node() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 1000.0, 50.0);
    set_node(&mut g, "b", 50.0, 50.0);
    set_edge(&mut g, "a", "b");
    layout(&mut g, None);

    let a = g.node("a").unwrap();
    let b = g.node("b").unwrap();
    assert!(a.x.is_some());
    assert!(a.y.is_some());
    assert!(b.x.is_some());
    assert!(b.y.is_some());
    assert!(a.y.unwrap() < b.y.unwrap());
}

#[test]
fn layout_parallel_edges() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 50.0);
    set_node(&mut g, "b", 50.0, 50.0);
    set_node(&mut g, "c", 50.0, 50.0);
    set_node(&mut g, "d", 50.0, 50.0);
    set_edge(&mut g, "a", "c");
    set_edge(&mut g, "a", "d");
    set_edge(&mut g, "b", "c");
    set_edge(&mut g, "b", "d");
    layout(&mut g, None);

    for v in &["a", "b", "c", "d"] {
        let n = g.node(v).unwrap();
        assert!(n.x.is_some(), "{} should have x", v);
        assert!(n.y.is_some(), "{} should have y", v);
    }
}

#[test]
fn layout_longer_chain() {
    let mut g = Graph::new();
    for v in &["a", "b", "c", "d", "e", "f", "g"] {
        set_node(&mut g, v, 30.0, 20.0);
    }
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "b", "c");
    set_edge(&mut g, "c", "d");
    set_edge(&mut g, "d", "e");
    set_edge(&mut g, "e", "f");
    set_edge(&mut g, "f", "g");
    layout(&mut g, None);

    let ys: Vec<f64> = ["a", "b", "c", "d", "e", "f", "g"]
        .iter()
        .map(|v| g.node(v).unwrap().y.unwrap())
        .collect();

    for i in 1..ys.len() {
        assert!(ys[i] > ys[i - 1], "y[{}]={} should be > y[{}]={}", i, ys[i], i - 1, ys[i - 1]);
    }
}

#[test]
fn layout_multiple_disconnected_chains() {
    let mut g = Graph::new();
    for v in &["a", "b", "c", "d", "e", "f"] {
        set_node(&mut g, v, 30.0, 20.0);
    }
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "b", "c");
    set_edge(&mut g, "d", "e");
    set_edge(&mut g, "e", "f");
    layout(&mut g, None);

    for v in &["a", "b", "c", "d", "e", "f"] {
        let n = g.node(v).unwrap();
        assert!(n.x.is_some(), "{} should have x", v);
        assert!(n.y.is_some(), "{} should have y", v);
    }
    // Both chains have rank ordering
    assert!(g.node("a").unwrap().y.unwrap() < g.node("b").unwrap().y.unwrap());
    assert!(g.node("d").unwrap().y.unwrap() < g.node("e").unwrap().y.unwrap());
}

#[test]
fn layout_assigns_edge_points() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 50.0);
    set_node(&mut g, "b", 50.0, 50.0);
    set_edge(&mut g, "a", "b");
    layout(&mut g, None);

    let edge_label = g.edge("a", "b", None).unwrap();
    assert!(!edge_label.points.is_empty(), "edge should have points");
}

#[test]
fn layout_all_coords_positive() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 50.0);
    set_node(&mut g, "b", 50.0, 50.0);
    set_node(&mut g, "c", 50.0, 50.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "a", "c");
    set_edge(&mut g, "b", "c");
    layout(&mut g, None);

    for v in &["a", "b", "c"] {
        let n = g.node(v).unwrap();
        assert!(n.x.unwrap() >= 0.0, "{}.x should be >= 0", v);
        assert!(n.y.unwrap() >= 0.0, "{}.y should be >= 0", v);
    }
}

#[test]
fn layout_nodes_same_rank_different_x() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 50.0);
    set_node(&mut g, "b", 50.0, 50.0);
    set_node(&mut g, "c", 50.0, 50.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "a", "c");
    layout(&mut g, None);

    // b and c are on the same rank; they should have different x coordinates
    let xb = g.node("b").unwrap().x.unwrap();
    let xc = g.node("c").unwrap().x.unwrap();
    assert!((xb - xc).abs() > 1.0, "b.x ({}) and c.x ({}) should differ", xb, xc);
}

#[test]
fn layout_long_edge_with_label() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 100.0);
    set_node(&mut g, "b", 75.0, 200.0);
    let mut el = EdgeLabel::default();
    el.width = 60.0;
    el.height = 70.0;
    el.minlen = 2;
    el.labelpos = LabelPos::Center;
    g.set_edge("a".to_string(), "b".to_string(), Some(el), None);
    layout(&mut g, None);

    let ya = g.node("a").unwrap().y.unwrap();
    let yb = g.node("b").unwrap().y.unwrap();
    let edge_label = g.edge("a", "b", None).unwrap();
    // Edge label should be between a and b
    if let (Some(ey), _) = (edge_label.y, edge_label.x) {
        assert!(ey > ya, "edge label y ({}) should be > a.y ({})", ey, ya);
        assert!(ey < yb, "edge label y ({}) should be < b.y ({})", ey, yb);
    }
}

// ============================================================
// Additional order module tests
// ============================================================

#[test]
fn order_assigns_unique_orders_per_rank() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 10.0, 10.0);
    set_node(&mut g, "b", 10.0, 10.0);
    set_node(&mut g, "c", 10.0, 10.0);
    set_node(&mut g, "d", 10.0, 10.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "a", "c");
    set_edge(&mut g, "b", "d");
    set_edge(&mut g, "c", "d");

    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("b").unwrap().rank = Some(1);
    g.node_mut("c").unwrap().rank = Some(1);
    g.node_mut("d").unwrap().rank = Some(2);

    order::order(&mut g);

    // b and c should have different orders
    let ob = g.node("b").unwrap().order.unwrap();
    let oc = g.node("c").unwrap().order.unwrap();
    assert_ne!(ob, oc, "b and c should have different orders");
}

// ============================================================
// Additional position module tests
// ============================================================

#[test]
fn position_two_nodes_same_rank_have_different_x() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 20.0);
    set_node(&mut g, "b", 50.0, 20.0);

    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("a").unwrap().order = Some(0);
    g.node_mut("b").unwrap().rank = Some(0);
    g.node_mut("b").unwrap().order = Some(1);

    position::position(&mut g);

    let xa = g.node("a").unwrap().x.unwrap();
    let xb = g.node("b").unwrap().x.unwrap();
    assert!(xb > xa, "b.x ({}) should be > a.x ({})", xb, xa);
}

#[test]
fn position_three_ranks_y_ordering() {
    let mut g = Graph::new();
    set_node(&mut g, "a", 50.0, 20.0);
    set_node(&mut g, "b", 50.0, 20.0);
    set_node(&mut g, "c", 50.0, 20.0);
    set_edge(&mut g, "a", "b");
    set_edge(&mut g, "b", "c");

    g.node_mut("a").unwrap().rank = Some(0);
    g.node_mut("a").unwrap().order = Some(0);
    g.node_mut("b").unwrap().rank = Some(1);
    g.node_mut("b").unwrap().order = Some(0);
    g.node_mut("c").unwrap().rank = Some(2);
    g.node_mut("c").unwrap().order = Some(0);

    position::position(&mut g);

    let ya = g.node("a").unwrap().y.unwrap();
    let yb = g.node("b").unwrap().y.unwrap();
    let yc = g.node("c").unwrap().y.unwrap();
    assert!(ya < yb);
    assert!(yb < yc);
}

// ============================================================
// Coordinate system tests
// ============================================================

use super::coordinate_system;

fn make_coord_graph(rankdir: RankDir) -> Graph<NodeLabel, EdgeLabel> {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::new();
    let mut label = NodeLabel::default();
    label.width = 100.0;
    label.height = 200.0;
    g.set_node("a".to_string(), Some(label));
    g.set_graph_label(GraphLabel {
        rankdir,
        ..Default::default()
    });
    g
}

#[test]
fn coord_adjust_tb_does_nothing() {
    let mut g = make_coord_graph(RankDir::TB);
    coordinate_system::adjust(&mut g);
    let a = g.node("a").unwrap();
    assert_eq!(a.width, 100.0);
    assert_eq!(a.height, 200.0);
}

#[test]
fn coord_adjust_bt_does_nothing() {
    let mut g = make_coord_graph(RankDir::BT);
    coordinate_system::adjust(&mut g);
    let a = g.node("a").unwrap();
    assert_eq!(a.width, 100.0);
    assert_eq!(a.height, 200.0);
}

#[test]
fn coord_adjust_lr_swaps_width_height() {
    let mut g = make_coord_graph(RankDir::LR);
    coordinate_system::adjust(&mut g);
    let a = g.node("a").unwrap();
    assert_eq!(a.width, 200.0);
    assert_eq!(a.height, 100.0);
}

#[test]
fn coord_adjust_rl_swaps_width_height() {
    let mut g = make_coord_graph(RankDir::RL);
    coordinate_system::adjust(&mut g);
    let a = g.node("a").unwrap();
    assert_eq!(a.width, 200.0);
    assert_eq!(a.height, 100.0);
}

fn make_coord_graph_with_pos(rankdir: RankDir) -> Graph<NodeLabel, EdgeLabel> {
    let mut g: Graph<NodeLabel, EdgeLabel> = Graph::new();
    let mut label = NodeLabel::default();
    label.width = 100.0;
    label.height = 200.0;
    label.x = Some(20.0);
    label.y = Some(40.0);
    g.set_node("a".to_string(), Some(label));
    g.set_graph_label(GraphLabel {
        rankdir,
        ..Default::default()
    });
    g
}

#[test]
fn coord_undo_tb_does_nothing() {
    let mut g = make_coord_graph_with_pos(RankDir::TB);
    coordinate_system::undo(&mut g);
    let a = g.node("a").unwrap();
    assert_eq!(a.x, Some(20.0));
    assert_eq!(a.y, Some(40.0));
    assert_eq!(a.width, 100.0);
    assert_eq!(a.height, 200.0);
}

#[test]
fn coord_undo_bt_negates_y() {
    let mut g = make_coord_graph_with_pos(RankDir::BT);
    coordinate_system::undo(&mut g);
    let a = g.node("a").unwrap();
    assert_eq!(a.x, Some(20.0));
    assert_eq!(a.y, Some(-40.0));
    assert_eq!(a.width, 100.0);
    assert_eq!(a.height, 200.0);
}

#[test]
fn coord_undo_lr_swaps_dims_and_coords() {
    let mut g = make_coord_graph_with_pos(RankDir::LR);
    coordinate_system::undo(&mut g);
    let a = g.node("a").unwrap();
    assert_eq!(a.x, Some(40.0));
    assert_eq!(a.y, Some(20.0));
    assert_eq!(a.width, 200.0);
    assert_eq!(a.height, 100.0);
}

#[test]
fn coord_undo_rl_swaps_and_negates() {
    let mut g = make_coord_graph_with_pos(RankDir::RL);
    coordinate_system::undo(&mut g);
    let a = g.node("a").unwrap();
    assert_eq!(a.x, Some(-40.0));
    assert_eq!(a.y, Some(20.0));
    assert_eq!(a.width, 200.0);
    assert_eq!(a.height, 100.0);
}
