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
    layout(&mut g);

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
    layout(&mut g);

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
    layout(&mut g);

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
    layout(&mut g);

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
    layout(&mut g);

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
    layout(&mut g);

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
