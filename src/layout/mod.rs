//! Dagre layout algorithm: Sugiyama-style hierarchical graph layout.
//!
//! The layout pipeline:
//! 1. Cycle removal (make graph acyclic)
//! 2. Rank assignment (network simplex)
//! 3. Ordering (crossing minimization)
//! 4. Coordinate assignment (Brandes-Kopf)

pub mod acyclic;
pub mod add_border_segments;
pub mod coordinate_system;
pub mod normalize;
pub mod rank;
pub mod order;
pub mod parent_dummy_chains;
pub mod position;
pub mod nesting_graph;
pub mod util;
pub mod types;

use crate::graph::{Edge, Graph};
use types::*;

/// Run the complete dagre layout algorithm on a graph.
///
/// Input: a `Graph<NodeLabel, EdgeLabel>` where each node has `width` and `height`,
/// and each edge has `minlen` and `weight`.
///
/// Output: each node will have `x`, `y`, `rank`, and `order` set.
/// Each edge will have `points` set with the waypoints.
pub fn layout(g: &mut Graph<NodeLabel, EdgeLabel>, opts: Option<LayoutOptions>) {
    let opts = opts.unwrap_or_default();

    // Store options into a GraphLabel and set it as the graph label
    let mut gl = GraphLabel::default();
    gl.rankdir = opts.rankdir;
    gl.align = opts.align;
    gl.nodesep = opts.nodesep;
    gl.edgesep = opts.edgesep;
    gl.ranksep = opts.ranksep;
    gl.marginx = opts.marginx;
    gl.marginy = opts.marginy;
    gl.acyclicer = opts.acyclicer;
    gl.ranker = opts.ranker;
    gl.rank_align = opts.rank_align;
    gl.compound = g.is_compound();
    g.set_graph_label(gl);

    // 1. Make space for edge labels
    make_space_for_edge_labels(g);

    // 2. Remove self edges
    remove_self_edges(g);

    // 3. Make graph acyclic
    let acyclicer = g.graph_label::<GraphLabel>().and_then(|gl| gl.acyclicer);
    acyclic::run(g, acyclicer);

    // 4-5. Nesting graph (if compound) + rank assignment
    let compound = g.graph_label::<GraphLabel>().map_or(false, |gl| gl.compound);
    if compound {
        let nesting_root = nesting_graph::run(g);
        if let Some(ref root) = nesting_root {
            // Rank on non-compound version
            let ranker = g.graph_label::<GraphLabel>().map_or(Ranker::NetworkSimplex, |gl| gl.ranker);
            let mut ncg = util::as_non_compound_graph(g);
            rank::rank(&mut ncg, ranker);
            // Transfer ranks back
            for v in ncg.nodes() {
                if let Some(node) = ncg.node(&v) {
                    if let Some(rank) = node.rank {
                        if let Some(gn) = g.node_mut(&v) {
                            gn.rank = Some(rank);
                        }
                    }
                }
            }
            if let Some(gl) = g.graph_label_mut::<GraphLabel>() {
                gl.nesting_root = Some(root.clone());
            }
        }
    } else {
        let ranker = g.graph_label::<GraphLabel>().map_or(Ranker::NetworkSimplex, |gl| gl.ranker);
        rank::rank(g, ranker);
    }

    // 6. Inject edge label proxies
    inject_edge_label_proxies(g);

    // 7. Remove empty ranks
    util::remove_empty_ranks(g);

    // 8. Nesting graph cleanup (if compound)
    if compound {
        let nesting_root = g
            .graph_label::<GraphLabel>()
            .and_then(|gl| gl.nesting_root.clone());
        if let Some(ref root) = nesting_root {
            nesting_graph::cleanup(g, root);
        }
    }

    // 9. Normalize ranks
    util::normalize_ranks(g);

    // 10. Assign rank min/max for compound nodes
    assign_rank_min_max(g);

    // 11. Remove edge label proxies
    remove_edge_label_proxies(g);

    // 12. Normalize long edges
    let mut dummy_chains = Vec::new();
    normalize::run(g, &mut dummy_chains);
    if let Some(gl) = g.graph_label_mut::<GraphLabel>() {
        gl.dummy_chains = dummy_chains.clone();
    }

    // 13. Parent dummy chains (if compound)
    if compound {
        parent_dummy_chains::parent_dummy_chains(g);
    }

    // 14. Add border segments (if compound)
    if compound {
        add_border_segments::add_border_segments(g);
    }

    // 15. Order (minimize crossings)
    order::order(g);

    // 16. Insert self edges
    insert_self_edges(g);

    // 17. Coordinate system adjust
    coordinate_system::adjust(g);

    // 18. Position (assign coordinates)
    position::position(g);

    // 19. Position self edges
    position_self_edges(g);

    // 20. Remove border nodes
    remove_border_nodes(g);

    // 21. Denormalize: restore original edges, collect edge points
    let chains = g
        .graph_label::<GraphLabel>()
        .map(|gl| gl.dummy_chains.clone())
        .unwrap_or_default();
    normalize::undo(g, &chains);

    // 22. Fix up edge label coords
    fixup_edge_label_coords(g);

    // 23. Coordinate system undo
    coordinate_system::undo(g);

    // 24. Translate graph
    translate_graph(g);

    // 25. Assign node intersects
    assign_node_intersects(g);

    // 26. Reverse points for reversed edges
    reverse_points_for_reversed_edges(g);

    // 27. Undo cycle removal
    acyclic::undo(g);
}

// ============================================================
// Helper functions ported from dagre.js layout.ts
// ============================================================

/// Halve ranksep in graph label. For each edge: double minlen.
/// If labelpos != Center and rankdir is TB/BT, add labeloffset to edge width;
/// if LR/RL, add to height.
fn make_space_for_edge_labels(g: &mut Graph<NodeLabel, EdgeLabel>) {
    let (rankdir, ranksep) = {
        let gl = g.graph_label::<GraphLabel>();
        (
            gl.map_or(RankDir::TB, |l| l.rankdir),
            gl.map_or(50.0, |l| l.ranksep),
        )
    };

    // Halve ranksep
    if let Some(gl) = g.graph_label_mut::<GraphLabel>() {
        gl.ranksep = ranksep / 2.0;
    }

    for e in g.edges() {
        if let Some(label) = g.edge_mut(&e.v, &e.w, e.name.as_deref()) {
            label.minlen *= 2;

            if label.labelpos != LabelPos::Center {
                match rankdir {
                    RankDir::TB | RankDir::BT => {
                        label.width += label.label_offset;
                    }
                    RankDir::LR | RankDir::RL => {
                        label.height += label.label_offset;
                    }
                }
            }
        }
    }
}

/// For edges where v==w, save {edge, label} onto the node's self_edges Vec,
/// then remove edge.
fn remove_self_edges(g: &mut Graph<NodeLabel, EdgeLabel>) {
    let self_edges: Vec<(Edge, EdgeLabel)> = g
        .edges()
        .into_iter()
        .filter(|e| e.v == e.w)
        .filter_map(|e| {
            let label = g.edge(&e.v, &e.w, e.name.as_deref()).cloned()?;
            Some((e, label))
        })
        .collect();

    for (e, label) in self_edges {
        g.remove_edge(&e.v, &e.w, e.name.as_deref());
        let self_edge = SelfEdge {
            e: e.clone(),
            label,
        };
        if let Some(node) = g.node_mut(&e.v) {
            node.self_edges.push(self_edge);
        }
    }
}

/// After ordering, re-insert self edges as dummy nodes with type "selfedge"
/// using add_dummy_node.
fn insert_self_edges(g: &mut Graph<NodeLabel, EdgeLabel>) {
    let layers = util::build_layer_matrix(g);

    for layer in &layers {
        let mut order_shift = 0usize;
        for (i, v) in layer.iter().enumerate() {
            if v.is_empty() {
                continue;
            }

            // Update order with accumulated shift
            if let Some(node) = g.node_mut(v) {
                node.order = Some(i + order_shift);
            }

            let self_edges: Vec<SelfEdge> = g
                .node(v)
                .map(|n| n.self_edges.clone())
                .unwrap_or_default();

            let node_rank = g.node(v).and_then(|n| n.rank);

            for se in &self_edges {
                let mut attrs = NodeLabel::default();
                attrs.width = se.label.width;
                attrs.height = se.label.height;
                attrs.rank = node_rank;
                attrs.order = Some(i + (order_shift += 1, order_shift).1);
                attrs.self_edge_data_e = Some(se.e.clone());
                attrs.self_edge_data_label = Some(se.label.clone());

                let dummy = util::add_dummy_node(g, "selfedge", attrs, "_se");
                let _ = dummy;
            }
        }
    }
}

/// For edges with non-zero width AND height, create dummy node at rank midpoint
/// with type "edge-proxy".
fn inject_edge_label_proxies(g: &mut Graph<NodeLabel, EdgeLabel>) {
    let edges: Vec<Edge> = g.edges();

    for e in edges {
        let (_width, _height, v_rank, w_rank) = {
            let label = match g.edge(&e.v, &e.w, e.name.as_deref()) {
                Some(l) => l,
                None => continue,
            };
            if label.width == 0.0 || label.height == 0.0 {
                continue;
            }
            let vr = g.node(&e.v).and_then(|n| n.rank).unwrap_or(0);
            let wr = g.node(&e.w).and_then(|n| n.rank).unwrap_or(0);
            (label.width, label.height, vr, wr)
        };

        let mid_rank = (w_rank - v_rank) / 2 + v_rank;

        // Set label_rank on the edge
        if let Some(label) = g.edge_mut(&e.v, &e.w, e.name.as_deref()) {
            label.label_rank = Some(mid_rank as f64);
        }
    }
}

/// Remove "edge-proxy" dummy nodes, setting label_rank on the associated edge.
fn remove_edge_label_proxies(g: &mut Graph<NodeLabel, EdgeLabel>) {
    let proxy_nodes: Vec<String> = g
        .nodes()
        .into_iter()
        .filter(|v| {
            g.node(v)
                .map_or(false, |n| n.dummy.as_deref() == Some("edge-proxy"))
        })
        .collect();

    for v in proxy_nodes {
        let rank = g.node(&v).and_then(|n| n.rank);
        let edge_obj = g.node(&v).and_then(|n| n.edge_obj.clone());

        if let Some(eo) = edge_obj {
            if let Some(label) = g.edge_mut(&eo.v, &eo.w, eo.name.as_deref()) {
                label.label_rank = rank.map(|r| r as f64);
            }
        }

        g.remove_node(&v);
    }
}

/// For nodes with border_top set, set min_rank = rank of border_top node,
/// max_rank = rank of border_bottom node.
fn assign_rank_min_max(g: &mut Graph<NodeLabel, EdgeLabel>) {
    // Collect the data first to avoid borrow conflicts
    let assignments: Vec<(String, i32, i32)> = g
        .nodes()
        .iter()
        .filter_map(|v| {
            let node = g.node(v)?;
            let bt = node.border_top.as_ref()?;
            let bb = node.border_bottom.as_ref()?;
            let min_r = g.node(bt).and_then(|n| n.rank)?;
            let max_r = g.node(bb).and_then(|n| n.rank)?;
            Some((v.clone(), min_r, max_r))
        })
        .collect();

    for (v, min_r, max_r) in assignments {
        if let Some(node) = g.node_mut(&v) {
            node.min_rank = Some(min_r);
            node.max_rank = Some(max_r);
        }
    }
}

/// Find bounding box of all nodes and edge labels, shift everything so
/// min coords = margin, set graph width/height.
fn translate_graph(g: &mut Graph<NodeLabel, EdgeLabel>) {
    let (marginx, marginy) = g
        .graph_label::<GraphLabel>()
        .map_or((0.0, 0.0), |gl| (gl.marginx, gl.marginy));

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    // Compute bounding box from nodes
    for v in g.nodes() {
        if let Some(node) = g.node(&v) {
            if let (Some(x), Some(y)) = (node.x, node.y) {
                let hw = node.width / 2.0;
                let hh = node.height / 2.0;
                min_x = min_x.min(x - hw);
                max_x = max_x.max(x + hw);
                min_y = min_y.min(y - hh);
                max_y = max_y.max(y + hh);
            }
        }
    }

    // Also consider edge label positions
    for e in g.edges() {
        if let Some(label) = g.edge(&e.v, &e.w, e.name.as_deref()) {
            if let (Some(x), Some(y)) = (label.x, label.y) {
                let hw = label.width / 2.0;
                let hh = label.height / 2.0;
                min_x = min_x.min(x - hw);
                max_x = max_x.max(x + hw);
                min_y = min_y.min(y - hh);
                max_y = max_y.max(y + hh);
            }
            for pt in &label.points {
                min_x = min_x.min(pt.x);
                max_x = max_x.max(pt.x);
                min_y = min_y.min(pt.y);
                max_y = max_y.max(pt.y);
            }
        }
    }

    if min_x == f64::INFINITY {
        return;
    }

    let dx = marginx - min_x;
    let dy = marginy - min_y;

    // Shift all nodes
    for v in g.nodes() {
        if let Some(node) = g.node_mut(&v) {
            if let Some(ref mut x) = node.x {
                *x += dx;
            }
            if let Some(ref mut y) = node.y {
                *y += dy;
            }
        }
    }

    // Shift all edge labels and points
    for e in g.edges() {
        if let Some(label) = g.edge_mut(&e.v, &e.w, e.name.as_deref()) {
            if let Some(ref mut x) = label.x {
                *x += dx;
            }
            if let Some(ref mut y) = label.y {
                *y += dy;
            }
            for pt in &mut label.points {
                pt.x += dx;
                pt.y += dy;
            }
        }
    }

    // Set graph dimensions
    let graph_width = max_x - min_x + 2.0 * marginx;
    let graph_height = max_y - min_y + 2.0 * marginy;
    if let Some(gl) = g.graph_label_mut::<GraphLabel>() {
        gl.width = graph_width;
        gl.height = graph_height;
    }
}

/// For each edge, compute intersection of edge path with source and target node
/// boundaries using intersect_rect. Prepend/append these points.
fn assign_node_intersects(g: &mut Graph<NodeLabel, EdgeLabel>) {
    for e in g.edges() {
        // Get source and target node labels
        let v_node = g.node(&e.v).cloned();
        let w_node = g.node(&e.w).cloned();

        let (src_point, tgt_point) = {
            let label = match g.edge(&e.v, &e.w, e.name.as_deref()) {
                Some(l) => l,
                None => continue,
            };

            // For source intersection: use the first point in the path or the target position
            let first = label.points.first().cloned().unwrap_or_else(|| {
                w_node.as_ref().map_or(
                    Point::new(0.0, 0.0),
                    |n| Point::new(n.x.unwrap_or(0.0), n.y.unwrap_or(0.0)),
                )
            });

            // For target intersection: use the last point in the path or the source position
            let last = label.points.last().cloned().unwrap_or_else(|| {
                v_node.as_ref().map_or(
                    Point::new(0.0, 0.0),
                    |n| Point::new(n.x.unwrap_or(0.0), n.y.unwrap_or(0.0)),
                )
            });

            let src = v_node
                .as_ref()
                .map(|n| util::intersect_rect(n, &first));
            let tgt = w_node
                .as_ref()
                .map(|n| util::intersect_rect(n, &last));

            (src, tgt)
        };

        if let Some(label) = g.edge_mut(&e.v, &e.w, e.name.as_deref()) {
            if let Some(pt) = src_point {
                label.points.insert(0, pt);
            }
            if let Some(pt) = tgt_point {
                label.points.push(pt);
            }
        }
    }
}

/// For edges with x set, adjust x based on labelpos (l/r) and labeloffset.
fn fixup_edge_label_coords(g: &mut Graph<NodeLabel, EdgeLabel>) {
    for e in g.edges() {
        if let Some(label) = g.edge_mut(&e.v, &e.w, e.name.as_deref()) {
            if label.x.is_some() {
                match label.labelpos {
                    LabelPos::Left => {
                        label.x = label.x.map(|x| x - label.width / 2.0 - label.label_offset);
                    }
                    LabelPos::Right => {
                        label.x = label.x.map(|x| x + label.width / 2.0 + label.label_offset);
                    }
                    LabelPos::Center => {
                        // No adjustment needed
                    }
                }
            }
        }
    }
}

/// For edges with reversed=true, reverse the points array.
fn reverse_points_for_reversed_edges(g: &mut Graph<NodeLabel, EdgeLabel>) {
    for e in g.edges() {
        if let Some(label) = g.edge_mut(&e.v, &e.w, e.name.as_deref()) {
            if label.reversed {
                label.points.reverse();
            }
        }
    }
}

/// For container nodes, compute width/height/x/y from border nodes.
/// Then remove all "border" dummy nodes.
fn remove_border_nodes(g: &mut Graph<NodeLabel, EdgeLabel>) {
    // First compute container node dimensions from border nodes
    let nodes: Vec<String> = g.nodes();
    for v in &nodes {
        let node = match g.node(v) {
            Some(n) => n.clone(),
            None => continue,
        };

        if node.border_left.is_empty() {
            continue;
        }

        // Compute bounding box from border nodes
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for bl in &node.border_left {
            if let Some(bn) = g.node(bl) {
                if let (Some(x), Some(y)) = (bn.x, bn.y) {
                    min_x = min_x.min(x - bn.width / 2.0);
                    min_y = min_y.min(y - bn.height / 2.0);
                }
            }
        }
        for br in &node.border_right {
            if let Some(bn) = g.node(br) {
                if let (Some(x), Some(y)) = (bn.x, bn.y) {
                    max_x = max_x.max(x + bn.width / 2.0);
                    max_y = max_y.max(y + bn.height / 2.0);
                }
            }
        }

        if min_x != f64::INFINITY && max_x != f64::NEG_INFINITY {
            let padding = node.padding;
            if let Some(node) = g.node_mut(v) {
                node.width = max_x - min_x + padding;
                node.height = max_y - min_y + padding;
                node.x = Some((min_x + max_x) / 2.0);
                node.y = Some((min_y + max_y) / 2.0);
            }
        }
    }

    // Remove all border dummy nodes
    let border_nodes: Vec<String> = g
        .nodes()
        .into_iter()
        .filter(|v| {
            g.node(v)
                .map_or(false, |n| n.dummy.as_deref() == Some("border"))
        })
        .collect();

    for v in border_nodes {
        g.remove_node(&v);
    }
}

/// For "selfedge" dummy nodes, create curved path points and set on the edge label.
fn position_self_edges(g: &mut Graph<NodeLabel, EdgeLabel>) {
    let self_edge_nodes: Vec<String> = g
        .nodes()
        .into_iter()
        .filter(|v| {
            g.node(v)
                .map_or(false, |n| n.dummy.as_deref() == Some("selfedge"))
        })
        .collect();

    for v in self_edge_nodes {
        let se_node = match g.node(&v).cloned() {
            Some(n) => n,
            None => continue,
        };

        let edge_desc = match se_node.self_edge_data_e.clone() {
            Some(e) => e,
            None => continue,
        };

        let se_label = se_node.self_edge_data_label.clone().unwrap_or_default();

        // Get the original node that has the self edge
        let orig_node = match g.node(&edge_desc.v).cloned() {
            Some(n) => n,
            None => continue,
        };

        let node_x = orig_node.x.unwrap_or(0.0);
        let node_y = orig_node.y.unwrap_or(0.0);
        let node_hw = orig_node.width / 2.0;
        let se_x = se_node.x.unwrap_or(0.0);
        let se_y = se_node.y.unwrap_or(0.0);

        // Create curved self-edge path
        let points = vec![
            Point::new(se_x - node_hw, se_y - se_label.height / 2.0),
            Point::new(se_x - node_hw, se_y + se_label.height / 2.0),
            Point::new(node_x - node_hw, node_y),
        ];

        // Set on the edge label
        if let Some(label) = g.edge_mut(&edge_desc.v, &edge_desc.w, edge_desc.name.as_deref()) {
            label.points = points;
            label.x = Some(se_x - node_hw);
            label.y = Some(se_y);
        }

        // Also restore the edge if it doesn't exist yet
        if !g.has_edge(&edge_desc.v, &edge_desc.w, edge_desc.name.as_deref()) {
            let mut label = se_label;
            label.points = vec![
                Point::new(se_x - node_hw, se_y - label.height / 2.0),
                Point::new(se_x - node_hw, se_y + label.height / 2.0),
                Point::new(node_x - node_hw, node_y),
            ];
            label.x = Some(se_x - node_hw);
            label.y = Some(se_y);
            g.set_edge(
                edge_desc.v.clone(),
                edge_desc.w.clone(),
                Some(label),
                edge_desc.name.as_deref(),
            );
        }

        g.remove_node(&v);
    }
}

#[cfg(test)]
mod tests;
