//! Parent dummy chains: assigns correct parent subgraph to dummy nodes
//! along edges that cross subgraph boundaries.
//!
//! Ported from dagre.js parent-dummy-chains.ts

use std::collections::HashMap;
use crate::graph::Graph;
use super::types::*;

/// Walk dummy chains and set their parent to the appropriate subgraph
/// using LCA-based path finding with postorder numbering.
pub(crate) fn parent_dummy_chains(g: &mut Graph<NodeLabel, EdgeLabel>) {
    let dummy_chains: Vec<String> = g
        .graph_label::<GraphLabel>()
        .map(|gl| gl.dummy_chains.clone())
        .unwrap_or_default();

    if dummy_chains.is_empty() {
        return;
    }

    // Build postorder numbering for the compound hierarchy
    let (post_lim, post_low) = build_postorder(g);

    for chain_start in &dummy_chains {
        let mut v = chain_start.clone();

        // Get the edge endpoints for the original edge this dummy chain represents
        let node = match g.node(&v) {
            Some(n) => n.clone(),
            None => continue,
        };
        let edge_obj = match &node.edge_obj {
            Some(e) => e.clone(),
            None => continue,
        };

        // Find LCA path between the source and target
        let path = find_path(g, &post_lim, &post_low, &edge_obj.v, &edge_obj.w);
        let mut path_idx = 0;
        let mut ascending = true;

        // Walk the dummy chain
        loop {
            let succs = g.successors(&v).unwrap_or_default();
            let w = match succs.into_iter().find(|s| {
                g.node(s).map_or(false, |n| n.dummy.is_some())
            }) {
                Some(w) => w,
                None => break,
            };

            if ascending {
                // Walk up the path (from source side) until we find the
                // right parent for the current rank
                let w_rank = g.node(&w).and_then(|n| n.rank).unwrap_or(0);
                while path_idx < path.ascending.len() {
                    let parent_node = &path.ascending[path_idx];
                    let (parent_min, parent_max) = get_rank_range(g, parent_node);
                    if w_rank >= parent_min && w_rank <= parent_max {
                        break;
                    }
                    path_idx += 1;
                }

                if path_idx < path.ascending.len() {
                    g.set_parent(&w, Some(&path.ascending[path_idx]));
                }

                if path_idx >= path.ascending.len() {
                    ascending = false;
                    // Switch to descending path
                    path_idx = path.descending.len().saturating_sub(1);
                }
            }

            if !ascending {
                // Walk down the path (to target side)
                let w_rank = g.node(&w).and_then(|n| n.rank).unwrap_or(0);
                while path_idx > 0 {
                    let parent_node = &path.descending[path_idx];
                    let (parent_min, parent_max) = get_rank_range(g, parent_node);
                    if w_rank >= parent_min && w_rank <= parent_max {
                        break;
                    }
                    path_idx = path_idx.saturating_sub(1);
                }

                if path_idx < path.descending.len() {
                    g.set_parent(&w, Some(&path.descending[path_idx]));
                }
            }

            v = w;
        }
    }
}

fn get_rank_range(g: &Graph<NodeLabel, EdgeLabel>, v: &str) -> (i32, i32) {
    let node = g.node(v);
    let min = node.and_then(|n| n.min_rank).or_else(|| node.and_then(|n| n.rank)).unwrap_or(0);
    let max = node.and_then(|n| n.max_rank).or_else(|| node.and_then(|n| n.rank)).unwrap_or(0);
    (min, max)
}

struct LcaPath {
    ascending: Vec<String>,
    descending: Vec<String>,
}

/// Build postorder numbering of the compound hierarchy.
/// Returns (lim map, low map).
fn build_postorder(
    g: &Graph<NodeLabel, EdgeLabel>,
) -> (HashMap<String, usize>, HashMap<String, usize>) {
    let mut lim: HashMap<String, usize> = HashMap::new();
    let mut low: HashMap<String, usize> = HashMap::new();
    let mut counter = 0usize;

    fn dfs(
        g: &Graph<NodeLabel, EdgeLabel>,
        v: &str,
        counter: &mut usize,
        lim: &mut HashMap<String, usize>,
        low: &mut HashMap<String, usize>,
    ) {
        let low_val = *counter;
        let children = g.children(Some(v));
        for child in &children {
            dfs(g, child, counter, lim, low);
        }
        low.insert(v.to_string(), low_val);
        lim.insert(v.to_string(), *counter);
        *counter += 1;
    }

    let roots = g.children(None);
    for root in &roots {
        dfs(g, root, &mut counter, &mut lim, &mut low);
    }

    (lim, low)
}

/// Find the path from v to w through their LCA in the compound hierarchy.
fn find_path(
    g: &Graph<NodeLabel, EdgeLabel>,
    lim: &HashMap<String, usize>,
    low: &HashMap<String, usize>,
    v: &str,
    w: &str,
) -> LcaPath {
    let mut v_path = Vec::new();
    let mut w_path = Vec::new();
    let mut v_cur = v.to_string();

    // Walk v up to the LCA
    let w_lim = lim.get(w).copied().unwrap_or(0);
    let w_low = low.get(w).copied().unwrap_or(0);

    // Ascend from v until we find an ancestor that contains w
    while !is_descendant(lim, low, &v_cur, w_lim, w_low) {
        if let Some(parent) = g.parent(&v_cur) {
            v_path.push(parent.to_string());
            v_cur = parent.to_string();
        } else {
            break;
        }
    }

    // Now ascend from w until we reach the same LCA
    let lca = v_cur.clone();
    let lca_lim = lim.get(&lca).copied().unwrap_or(0);
    let lca_low = low.get(&lca).copied().unwrap_or(0);

    let mut w_cur = w.to_string();
    #[allow(unused_assignments)]
    while w_cur != lca {
        if let Some(parent) = g.parent(&w_cur) {
            let p = parent.to_string();
            if p == lca || (lim.get(&p).copied().unwrap_or(0) == lca_lim
                && low.get(&p).copied().unwrap_or(0) == lca_low)
            {
                w_path.push(p);
                break;
            }
            w_path.push(p.clone());
            w_cur = p;
        } else {
            break;
        }
    }

    LcaPath {
        ascending: v_path,
        descending: w_path,
    }
}

fn is_descendant(
    lim: &HashMap<String, usize>,
    low: &HashMap<String, usize>,
    ancestor: &str,
    node_lim: usize,
    node_low: usize,
) -> bool {
    let a_low = low.get(ancestor).copied().unwrap_or(0);
    let a_lim = lim.get(ancestor).copied().unwrap_or(0);
    a_low <= node_low && node_lim <= a_lim
}
