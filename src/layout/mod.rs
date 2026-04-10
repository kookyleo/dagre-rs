//! Dagre layout algorithm: Sugiyama-style hierarchical graph layout.
//!
//! The layout pipeline:
//! 1. Cycle removal (make graph acyclic)
//! 2. Rank assignment (network simplex)
//! 3. Ordering (crossing minimization)
//! 4. Coordinate assignment (Brandes-Köpf)

pub mod acyclic;
pub mod normalize;
pub mod rank;
pub mod order;
pub mod position;
pub mod nesting_graph;
pub mod util;
pub mod types;

use crate::graph::Graph;
use types::*;

/// Run the complete dagre layout algorithm on a graph.
///
/// Input: a `Graph<NodeLabel, EdgeLabel>` where each node has `width` and `height`,
/// and each edge has `minlen` and `weight`.
///
/// Output: each node will have `x`, `y`, `rank`, and `order` set.
/// Each edge will have `points` set with the waypoints.
pub fn layout(g: &mut Graph<NodeLabel, EdgeLabel>) {
    // Phase 0: Make graph acyclic
    acyclic::run(g, None);

    // Phase 1: Rank assignment
    rank::rank(g, Ranker::NetworkSimplex);

    // Normalize ranks (shift to start at 0)
    util::normalize_ranks(g);
    util::remove_empty_ranks(g);

    // Phase 2: Normalize long edges (break into unit-length segments)
    let mut dummy_chains = Vec::new();
    normalize::run(g, &mut dummy_chains);

    // Phase 3: Order (minimize crossings)
    order::order(g);

    // Phase 4: Position (assign coordinates)
    position::position(g);

    // Denormalize: restore original edges, collect edge points
    normalize::undo(g, &dummy_chains);

    // Undo cycle removal
    acyclic::undo(g);
}

#[cfg(test)]
mod tests;

