//! Brandes-Köpf coordinate assignment algorithm.
//!
//! "Fast and Simple Horizontal Coordinate Assignment" — the algorithm runs
//! 4 times (up/down × left/right), producing one x-coordinate mapping each
//! time, then balances the results by taking the average of the two median
//! values for every node.

use std::collections::HashMap;

use log::trace;

use crate::graph::Graph;
use crate::layout::types::{
    Align, BorderType, EdgeLabel, GraphLabel, LabelPos, NodeLabel,
};
use crate::layout::util::build_layer_matrix;

/// Merged conflict set – keyed by the pair (min(v,w), max(v,w)).
type Conflicts = HashMap<String, HashMap<String, bool>>;

/// Node-id -> x-coordinate.
type PositionMap = HashMap<String, f64>;

/// Alignment result from `vertical_alignment`.
pub(crate) struct AlignmentResult {
    pub(crate) root: HashMap<String, String>,
    pub(crate) align: HashMap<String, String>,
}

/// 4-direction keyed x-coordinate maps: "ul", "ur", "dl", "dr".
type XssMap = HashMap<String, PositionMap>;

// ---------------------------------------------------------------------------
// Public entry
// ---------------------------------------------------------------------------

/// Main entry: compute x-coordinates for every node using the BK algorithm.
pub(crate) fn position_x(g: &Graph<NodeLabel, EdgeLabel>) -> PositionMap {
    let layering = build_layer_matrix(g);
    let mut conflicts = find_type1_conflicts(g, &layering);
    // Merge type-2 conflicts into the same map.
    let type2 = find_type2_conflicts(g, &layering);
    for (k, inner) in type2 {
        let entry = conflicts.entry(k).or_default();
        for (k2, v2) in inner {
            entry.insert(k2, v2);
        }
    }

    let mut xss: XssMap = HashMap::new();

    for vert in &["u", "d"] {
        let adjusted_base: Vec<Vec<String>> = if *vert == "u" {
            layering.clone()
        } else {
            layering.iter().rev().cloned().collect()
        };

        for horiz in &["l", "r"] {
            let adjusted: Vec<Vec<String>> = if *horiz == "r" {
                adjusted_base
                    .iter()
                    .map(|layer| layer.iter().rev().cloned().collect())
                    .collect()
            } else {
                adjusted_base.clone()
            };

            let neighbor_fn: Box<dyn Fn(&str) -> Vec<String>> = if *vert == "u" {
                Box::new(|v: &str| g.predecessors(v).unwrap_or_default())
            } else {
                Box::new(|v: &str| g.successors(v).unwrap_or_default())
            };

            let alignment = vertical_alignment(g, &adjusted, &conflicts, &*neighbor_fn);
            let mut xs = horizontal_compaction(
                g,
                &adjusted,
                &alignment.root,
                &alignment.align,
                *horiz == "r",
            );

            if *horiz == "r" {
                for val in xs.values_mut() {
                    *val = -*val;
                }
            }

            let key = format!("{}{}", vert, horiz);
            trace!("BK alignment {}: {} nodes", key, xs.len());
            xss.insert(key, xs);
        }
    }

    let (smallest_key, smallest) = find_smallest_width_alignment(g, &xss);
    align_coordinates(&mut xss, &smallest_key, &smallest);

    let graph_label = graph_label(g);
    let align_opt = graph_label.and_then(|gl| gl.align);
    balance(&xss, align_opt)
}

// ---------------------------------------------------------------------------
// Type-1 conflicts
// ---------------------------------------------------------------------------

/// Find type-1 conflicts: non-inner segment edges that cross inner segments.
/// An inner segment is an edge where both endpoints are dummy nodes.
pub(crate) fn find_type1_conflicts(
    g: &Graph<NodeLabel, EdgeLabel>,
    layering: &[Vec<String>],
) -> Conflicts {
    let mut conflicts: Conflicts = HashMap::new();

    if layering.is_empty() {
        return conflicts;
    }

    let mut prev_layer: &Vec<String> = &layering[0];
    for layer in layering.iter().skip(1) {
        let mut k0: usize = 0;
        let mut scan_pos: usize = 0;
        let prev_layer_length = prev_layer.len();
        let last_node = layer.last().cloned().unwrap_or_default();

        for (i, v) in layer.iter().enumerate() {
            let w = find_other_inner_segment_node(g, v);
            let k1: usize = if let Some(ref w_id) = w {
                if let Some(nl) = g.node(w_id) {
                    nl.order.unwrap_or(prev_layer_length)
                } else {
                    prev_layer_length
                }
            } else {
                prev_layer_length
            };

            if w.is_some() || *v == last_node {
                for scan_node in layer.iter().skip(scan_pos).take(i + 1 - scan_pos) {
                    if let Some(preds) = g.predecessors(scan_node) {
                        for u in &preds {
                            if let Some(u_label) = g.node(u) {
                                let u_pos = u_label.order.unwrap_or(0);
                                if (u_pos < k0 || k1 < u_pos)
                                    && !(u_label.dummy.is_some()
                                        && g.node(scan_node)
                                            .map_or(false, |sn| sn.dummy.is_some()))
                                {
                                    add_conflict(&mut conflicts, u, scan_node);
                                }
                            }
                        }
                    }
                }
                scan_pos = i + 1;
                k0 = k1;
            }
        }

        prev_layer = layer;
    }

    conflicts
}

// ---------------------------------------------------------------------------
// Type-2 conflicts
// ---------------------------------------------------------------------------

/// Find type-2 conflicts: dummy-to-dummy edges that cross border segments.
pub(crate) fn find_type2_conflicts(
    g: &Graph<NodeLabel, EdgeLabel>,
    layering: &[Vec<String>],
) -> Conflicts {
    let mut conflicts: Conflicts = HashMap::new();

    if layering.is_empty() {
        return conflicts;
    }

    let mut prev_layer: &Vec<String> = &layering[0];
    for layer in layering.iter().skip(1) {
        let north = prev_layer;
        let south = layer;
        let mut prev_north_pos: isize = -1;
        let mut next_north_pos: isize = -1;
        let mut south_pos: usize = 0;

        for (south_lookahead, v) in south.iter().enumerate() {
            if let Some(nl) = g.node(v) {
                if nl.dummy.as_deref() == Some("border") {
                    if let Some(preds) = g.predecessors(v) {
                        if let Some(first_pred) = preds.first() {
                            if let Some(pred_label) = g.node(first_pred) {
                                next_north_pos = pred_label.order.unwrap_or(0) as isize;
                                scan_type2(
                                    g,
                                    &mut conflicts,
                                    south,
                                    south_pos,
                                    south_lookahead,
                                    prev_north_pos,
                                    next_north_pos,
                                );
                                south_pos = south_lookahead;
                                prev_north_pos = next_north_pos;
                            }
                        }
                    }
                }
            }
            // Final scan after every node (matches the JS forEach structure:
            // the scan call is inside the forEach but outside the if-border block).
            scan_type2(
                g,
                &mut conflicts,
                south,
                south_pos,
                south.len(),
                next_north_pos,
                north.len() as isize,
            );
        }

        prev_layer = layer;
    }

    conflicts
}

fn scan_type2(
    g: &Graph<NodeLabel, EdgeLabel>,
    conflicts: &mut Conflicts,
    south: &[String],
    south_pos: usize,
    south_end: usize,
    prev_north_border: isize,
    next_north_border: isize,
) {
    for i in south_pos..south_end {
        if let Some(v) = south.get(i) {
            if let Some(nl) = g.node(v) {
                if nl.dummy.is_some() {
                    if let Some(preds) = g.predecessors(v) {
                        for u in &preds {
                            if let Some(u_node) = g.node(u) {
                                if u_node.dummy.is_some() {
                                    let u_order = u_node.order.unwrap_or(0) as isize;
                                    if u_order < prev_north_border
                                        || u_order > next_north_border
                                    {
                                        add_conflict(conflicts, u, v);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Conflict helpers
// ---------------------------------------------------------------------------

/// If v is a dummy node with a dummy predecessor, return that predecessor.
fn find_other_inner_segment_node(
    g: &Graph<NodeLabel, EdgeLabel>,
    v: &str,
) -> Option<String> {
    if let Some(nl) = g.node(v) {
        if nl.dummy.is_some() {
            if let Some(preds) = g.predecessors(v) {
                return preds
                    .into_iter()
                    .find(|u| g.node(u).map_or(false, |ul| ul.dummy.is_some()));
            }
        }
    }
    None
}

/// Record a conflict between nodes v and w (order-independent).
pub(crate) fn add_conflict(conflicts: &mut Conflicts, v: &str, w: &str) {
    let (v, w) = if v > w { (w, v) } else { (v, w) };
    conflicts
        .entry(v.to_string())
        .or_default()
        .insert(w.to_string(), true);
}

/// Check if a conflict exists between nodes v and w.
pub(crate) fn has_conflict(conflicts: &Conflicts, v: &str, w: &str) -> bool {
    let (v, w) = if v > w { (w, v) } else { (v, w) };
    conflicts.get(v).map_or(false, |inner| inner.contains_key(w))
}

// ---------------------------------------------------------------------------
// Vertical alignment
// ---------------------------------------------------------------------------

/// Attempt to align nodes into vertical "blocks" by connecting each node to
/// one of its median neighbors, skipping conflict edges and already-blocked
/// positions.
pub(crate) fn vertical_alignment(
    _g: &Graph<NodeLabel, EdgeLabel>,
    layering: &[Vec<String>],
    conflicts: &Conflicts,
    neighbor_fn: &dyn Fn(&str) -> Vec<String>,
) -> AlignmentResult {
    let mut root: HashMap<String, String> = HashMap::new();
    let mut align: HashMap<String, String> = HashMap::new();
    let mut pos: HashMap<String, usize> = HashMap::new();

    // Cache positions from layering order (may differ from node.order when
    // the layering has been reversed for a particular sweep direction).
    for layer in layering {
        for (order, v) in layer.iter().enumerate() {
            root.insert(v.clone(), v.clone());
            align.insert(v.clone(), v.clone());
            pos.insert(v.clone(), order);
        }
    }

    for layer in layering {
        let mut prev_idx: isize = -1;
        for v in layer {
            let ws_raw = neighbor_fn(v);
            if ws_raw.is_empty() {
                continue;
            }
            let mut ws = ws_raw;
            ws.sort_by(|a, b| {
                let pa = pos.get(a.as_str()).copied().unwrap_or(0);
                let pb = pos.get(b.as_str()).copied().unwrap_or(0);
                pa.cmp(&pb)
            });

            let mp: f64 = (ws.len() as f64 - 1.0) / 2.0;
            let lo = mp.floor() as usize;
            let hi = mp.ceil() as usize;

            for i in lo..=hi {
                if let Some(w) = ws.get(i) {
                    let pos_w = match pos.get(w.as_str()) {
                        Some(&p) => p,
                        None => continue,
                    };

                    if align.get(v.as_str()).map(|a| a.as_str()) == Some(v.as_str())
                        && prev_idx < pos_w as isize
                        && !has_conflict(conflicts, v, w)
                    {
                        if let Some(root_w) = root.get(w.as_str()).cloned() {
                            align.insert(w.clone(), v.clone());
                            let r = root_w.clone();
                            root.insert(v.clone(), r.clone());
                            align.insert(v.clone(), r);
                            prev_idx = pos_w as isize;
                        }
                    }
                }
            }
        }
    }

    AlignmentResult { root, align }
}

// ---------------------------------------------------------------------------
// Horizontal compaction
// ---------------------------------------------------------------------------

/// Compute final x-coordinates by building a "block graph" and performing
/// two sweeps: first forward (smallest coordinates) then backward (tighten
/// unused space).
pub(crate) fn horizontal_compaction(
    g: &Graph<NodeLabel, EdgeLabel>,
    layering: &[Vec<String>],
    root: &HashMap<String, String>,
    align: &HashMap<String, String>,
    reverse_sep: bool,
) -> PositionMap {
    let mut xs: PositionMap = HashMap::new();
    let block_g = build_block_graph(g, layering, root, reverse_sep);
    let border_type = if reverse_sep {
        BorderType::Left
    } else {
        BorderType::Right
    };

    // Iterative post-order traversal via an explicit stack.
    fn iterate(
        block_g: &Graph<(), f64>,
        xs: &mut PositionMap,
        set_xs_func: &dyn Fn(&Graph<(), f64>, &mut PositionMap, &str),
        next_nodes_func: &dyn Fn(&Graph<(), f64>, &str) -> Vec<String>,
    ) {
        let mut stack: Vec<String> = block_g.nodes();
        let mut visited: HashMap<String, bool> = HashMap::new();

        while let Some(elem) = stack.pop() {
            if visited.contains_key(&elem) {
                set_xs_func(block_g, xs, &elem);
            } else {
                visited.insert(elem.clone(), true);
                stack.push(elem.clone());
                for next_elem in next_nodes_func(block_g, &elem) {
                    stack.push(next_elem);
                }
            }
        }
    }

    // Pass 1: assign smallest coordinates (forward from predecessors).
    let pass1 = |block_g: &Graph<(), f64>, xs: &mut PositionMap, elem: &str| {
        if let Some(in_edges) = block_g.in_edges(elem, None) {
            if in_edges.is_empty() {
                xs.insert(elem.to_string(), 0.0);
            } else {
                let val = in_edges.iter().fold(0.0_f64, |acc, e| {
                    let xs_v = xs.get(&e.v).copied().unwrap_or(0.0);
                    let edge_weight = block_g
                        .edge(&e.v, &e.w, e.name.as_deref())
                        .copied()
                        .unwrap_or(0.0);
                    acc.max(xs_v + edge_weight)
                });
                xs.insert(elem.to_string(), val);
            }
        } else {
            xs.insert(elem.to_string(), 0.0);
        }
    };

    let predecessors_fn = |block_g: &Graph<(), f64>, elem: &str| -> Vec<String> {
        block_g.predecessors(elem).unwrap_or_default()
    };

    iterate(&block_g, &mut xs, &pass1, &predecessors_fn);

    // Pass 2: assign greatest coordinates (backward from successors).
    let pass2 = |block_g: &Graph<(), f64>, xs: &mut PositionMap, elem: &str| {
        let mut min = f64::INFINITY;
        if let Some(out_edges) = block_g.out_edges(elem, None) {
            if !out_edges.is_empty() {
                min = out_edges.iter().fold(f64::INFINITY, |acc, e| {
                    let xs_w = xs.get(&e.w).copied().unwrap_or(0.0);
                    let edge_weight = block_g
                        .edge(&e.v, &e.w, e.name.as_deref())
                        .copied()
                        .unwrap_or(0.0);
                    acc.min(xs_w - edge_weight)
                });
            }
        }

        if let Some(node) = g.node(elem) {
            if min != f64::INFINITY && node.border_type != Some(border_type) {
                let cur = xs.get(elem).copied().unwrap_or(0.0);
                xs.insert(elem.to_string(), cur.max(min));
            }
        }
    };

    let successors_fn = |block_g: &Graph<(), f64>, elem: &str| -> Vec<String> {
        block_g.successors(elem).unwrap_or_default()
    };

    iterate(&block_g, &mut xs, &pass2, &successors_fn);

    // Assign x coordinates to all nodes (non-root nodes get their root's x).
    for v in align.keys() {
        if let Some(root_v) = root.get(v) {
            let root_x = xs.get(root_v).copied().unwrap_or(0.0);
            xs.insert(v.clone(), root_x);
        }
    }

    xs
}

// ---------------------------------------------------------------------------
// Block graph construction
// ---------------------------------------------------------------------------

/// Build a graph where each block root is a node, and edges represent the
/// minimum required separation between adjacent blocks in the same layer.
fn build_block_graph(
    g: &Graph<NodeLabel, EdgeLabel>,
    layering: &[Vec<String>],
    root: &HashMap<String, String>,
    reverse_sep: bool,
) -> Graph<(), f64> {
    let mut block_g: Graph<(), f64> = Graph::new();
    let gl = graph_label(g);
    let nodesep = gl.map_or(50.0, |l| l.nodesep);
    let edgesep = gl.map_or(20.0, |l| l.edgesep);

    for layer in layering {
        let mut u: Option<&String> = None;
        for v in layer {
            if let Some(v_root) = root.get(v) {
                block_g.set_node(v_root.clone(), None);
                if let Some(u_id) = u {
                    if let Some(u_root) = root.get(u_id) {
                        let sep_val = sep(g, nodesep, edgesep, reverse_sep, v, u_id);
                        let prev_max = block_g
                            .edge(u_root, v_root, None)
                            .copied()
                            .unwrap_or(0.0);
                        let new_weight = sep_val.max(prev_max);
                        block_g.set_edge(
                            u_root.clone(),
                            v_root.clone(),
                            Some(new_weight),
                            None,
                        );
                    }
                }
                u = Some(v);
            }
        }
    }

    block_g
}

// ---------------------------------------------------------------------------
// Separation function
// ---------------------------------------------------------------------------

/// Compute the required horizontal separation between nodes v and w,
/// considering their widths, label positions, and dummy/non-dummy status.
fn sep(
    g: &Graph<NodeLabel, EdgeLabel>,
    nodesep: f64,
    edgesep: f64,
    reverse_sep: bool,
    v: &str,
    w: &str,
) -> f64 {
    let v_label = match g.node(v) {
        Some(l) => l,
        None => return 0.0,
    };
    let w_label = match g.node(w) {
        Some(l) => l,
        None => return 0.0,
    };

    let mut sum = 0.0;

    sum += v_label.width / 2.0;

    // labelpos adjustment for v
    let mut delta: Option<f64> = match v_label.labelpos {
        LabelPos::Left => Some(-v_label.width / 2.0),
        LabelPos::Right => Some(v_label.width / 2.0),
        LabelPos::Center => None,
    };
    if let Some(d) = delta {
        sum += if reverse_sep { d } else { -d };
    }

    sum += (if v_label.dummy.is_some() {
        edgesep
    } else {
        nodesep
    }) / 2.0;
    sum += (if w_label.dummy.is_some() {
        edgesep
    } else {
        nodesep
    }) / 2.0;

    sum += w_label.width / 2.0;

    // labelpos adjustment for w
    delta = match w_label.labelpos {
        LabelPos::Left => Some(w_label.width / 2.0),
        LabelPos::Right => Some(-w_label.width / 2.0),
        LabelPos::Center => None,
    };
    if let Some(d) = delta {
        sum += if reverse_sep { d } else { -d };
    }

    sum
}

// ---------------------------------------------------------------------------
// Alignment & balancing
// ---------------------------------------------------------------------------

/// Return the key and alignment (PositionMap) that results in the smallest
/// total width. Iterates in the same order as dagre.js (ul, ur, dl, dr)
/// so that when two alignments have the same width, the first one wins.
pub(crate) fn find_smallest_width_alignment(
    g: &Graph<NodeLabel, EdgeLabel>,
    xss: &XssMap,
) -> (String, PositionMap) {
    let mut best_width = f64::INFINITY;
    let mut best_key: Option<&str> = None;
    let mut best_xs: Option<&PositionMap> = None;

    // Iterate in the same order as dagre.js Object.values(xss)
    for key in &["ul", "ur", "dl", "dr"] {
        let xs = match xss.get(*key) {
            Some(xs) => xs,
            None => continue,
        };

        let mut max = f64::NEG_INFINITY;
        let mut min = f64::INFINITY;

        for (v, &x) in xs {
            let half_w = node_width(g, v) / 2.0;
            let right = x + half_w;
            let left = x - half_w;
            if right > max {
                max = right;
            }
            if left < min {
                min = left;
            }
        }

        let width = max - min;
        if width < best_width {
            best_width = width;
            best_key = Some(key);
            best_xs = Some(xs);
        }
    }

    (
        best_key.unwrap_or("ul").to_string(),
        best_xs.cloned().unwrap_or_default(),
    )
}

/// Shift all alignments so that left-biased ones share their minimum x with
/// the smallest-width alignment's minimum, and right-biased ones share their
/// maximum x with the smallest-width alignment's maximum.
pub(crate) fn align_coordinates(
    xss: &mut XssMap,
    align_to_key: &str,
    align_to: &PositionMap,
) {
    if align_to.is_empty() {
        return;
    }

    let align_to_min = align_to
        .values()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let align_to_max = align_to
        .values()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    for vert in &["u", "d"] {
        for horiz in &["l", "r"] {
            let key = format!("{}{}", vert, horiz);
            // Skip the alignment we're aligning to (JS uses reference equality)
            if key == align_to_key {
                continue;
            }
            let xs = match xss.get(&key) {
                Some(xs) => xs.clone(),
                None => continue,
            };

            if xs.is_empty() {
                continue;
            }

            let xs_min = xs.values().copied().fold(f64::INFINITY, f64::min);
            let xs_max = xs.values().copied().fold(f64::NEG_INFINITY, f64::max);

            let delta = if *horiz == "l" {
                align_to_min - xs_min
            } else {
                align_to_max - xs_max
            };

            if delta.abs() > f64::EPSILON {
                let shifted: PositionMap = xs
                    .iter()
                    .map(|(k, &v)| (k.clone(), v + delta))
                    .collect();
                xss.insert(key, shifted);
            }
        }
    }
}

/// Produce the final x-coordinate for each node. If an explicit alignment is
/// specified, use that single alignment's values; otherwise take the average
/// of the two median values from the 4 alignments.
pub(crate) fn balance(xss: &XssMap, align: Option<Align>) -> PositionMap {
    let ul_map = match xss.get("ul") {
        Some(m) => m,
        None => return HashMap::new(),
    };

    ul_map
        .iter()
        .map(|(v, _)| {
            if let Some(a) = align {
                let key = match a {
                    Align::UL => "ul",
                    Align::UR => "ur",
                    Align::DL => "dl",
                    Align::DR => "dr",
                };
                if let Some(alignment) = xss.get(key) {
                    if let Some(&val) = alignment.get(v) {
                        return (v.clone(), val);
                    }
                }
            }

            // Collect values from all 4 alignments, sort, and take the average
            // of the two middle values (indices 1 and 2).
            let mut vals: Vec<f64> = xss
                .values()
                .map(|xs| xs.get(v).copied().unwrap_or(0.0))
                .collect();
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let median = (vals.get(1).copied().unwrap_or(0.0)
                + vals.get(2).copied().unwrap_or(0.0))
                / 2.0;
            (v.clone(), median)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn node_width(g: &Graph<NodeLabel, EdgeLabel>, v: &str) -> f64 {
    g.node(v).map_or(0.0, |n| n.width)
}

/// Retrieve the graph-level label, downcasting from Any.
fn graph_label(g: &Graph<NodeLabel, EdgeLabel>) -> Option<&GraphLabel> {
    g.graph_label::<GraphLabel>()
}
