//! Graph algorithms: topological sort, DFS traversal, connected components, etc.

use super::Graph;
use std::collections::{HashMap, HashSet, VecDeque};

/// Error returned when a cycle is detected during topological sort.
#[derive(Debug)]
pub struct CycleError;

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graph contains a cycle")
    }
}

impl std::error::Error for CycleError {}

/// Topological sort of a directed graph. Returns an error if the graph has a cycle.
pub fn topsort<N, E>(g: &Graph<N, E>) -> Result<Vec<String>, CycleError> {
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    let mut result = Vec::new();

    fn visit<N, E>(
        g: &Graph<N, E>,
        v: &str,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) -> Result<(), CycleError> {
        if stack.contains(v) {
            return Err(CycleError);
        }
        if visited.contains(v) {
            return Ok(());
        }
        stack.insert(v.to_string());
        visited.insert(v.to_string());

        if let Some(preds) = g.predecessors(v) {
            for pred in preds {
                visit(g, &pred, visited, stack, result)?;
            }
        }

        stack.remove(v);
        result.push(v.to_string());
        Ok(())
    }

    let nodes = g.nodes();
    for v in &nodes {
        visit(g, v, &mut visited, &mut stack, &mut result)?;
    }

    Ok(result)
}

/// Check if a directed graph is acyclic.
pub fn is_acyclic<N, E>(g: &Graph<N, E>) -> bool {
    topsort(g).is_ok()
}

/// Find all cycles in the graph. Returns a list of cycles, each being a list of node IDs.
pub fn find_cycles<N, E>(g: &Graph<N, E>) -> Vec<Vec<String>> {
    let sccs = tarjan(g);
    sccs.into_iter()
        .filter(|scc| scc.len() > 1 || {
            let v = &scc[0];
            g.has_edge(v, v, None)
        })
        .collect()
}

/// Tarjan's strongly connected components algorithm.
pub fn tarjan<N, E>(g: &Graph<N, E>) -> Vec<Vec<String>> {
    let mut index = 0u32;
    let mut stack = Vec::new();
    let mut on_stack = HashSet::new();
    let mut indices: HashMap<String, u32> = HashMap::new();
    let mut lowlinks: HashMap<String, u32> = HashMap::new();
    let mut result = Vec::new();

    fn strongconnect<N, E>(
        g: &Graph<N, E>,
        v: &str,
        index: &mut u32,
        stack: &mut Vec<String>,
        on_stack: &mut HashSet<String>,
        indices: &mut HashMap<String, u32>,
        lowlinks: &mut HashMap<String, u32>,
        result: &mut Vec<Vec<String>>,
    ) {
        indices.insert(v.to_string(), *index);
        lowlinks.insert(v.to_string(), *index);
        *index += 1;
        stack.push(v.to_string());
        on_stack.insert(v.to_string());

        if let Some(succs) = g.successors(v) {
            for w in succs {
                if !indices.contains_key(&w) {
                    strongconnect(g, &w, index, stack, on_stack, indices, lowlinks, result);
                    let lw = lowlinks[&w];
                    let lv = lowlinks.get_mut(v).unwrap();
                    *lv = (*lv).min(lw);
                } else if on_stack.contains(&w) {
                    let iw = indices[&w];
                    let lv = lowlinks.get_mut(v).unwrap();
                    *lv = (*lv).min(iw);
                }
            }
        }

        if lowlinks[v] == indices[v] {
            let mut scc = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(&w);
                scc.push(w.clone());
                if w == v {
                    break;
                }
            }
            result.push(scc);
        }
    }

    for v in g.nodes() {
        if !indices.contains_key(&v) {
            strongconnect(
                g,
                &v,
                &mut index,
                &mut stack,
                &mut on_stack,
                &mut indices,
                &mut lowlinks,
                &mut result,
            );
        }
    }

    result
}

/// Depth-first search traversal.
pub fn dfs<N, E>(g: &Graph<N, E>, roots: &[&str], order: DfsOrder) -> Vec<String> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();

    fn do_dfs<N, E>(
        g: &Graph<N, E>,
        v: &str,
        order: DfsOrder,
        visited: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) {
        if visited.contains(v) {
            return;
        }
        visited.insert(v.to_string());

        if order == DfsOrder::Pre {
            result.push(v.to_string());
        }

        if let Some(neighbors) = g.successors(v) {
            for w in neighbors {
                do_dfs(g, &w, order, visited, result);
            }
        }

        if order == DfsOrder::Post {
            result.push(v.to_string());
        }
    }

    for root in roots {
        do_dfs(g, root, order, &mut visited, &mut result);
    }

    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DfsOrder {
    Pre,
    Post,
}

/// Preorder DFS traversal.
pub fn preorder<N, E>(g: &Graph<N, E>, roots: &[&str]) -> Vec<String> {
    dfs(g, roots, DfsOrder::Pre)
}

/// Postorder DFS traversal.
pub fn postorder<N, E>(g: &Graph<N, E>, roots: &[&str]) -> Vec<String> {
    dfs(g, roots, DfsOrder::Post)
}

/// Find weakly connected components. Returns a list of components,
/// each being a list of node IDs.
pub fn components<N, E>(g: &Graph<N, E>) -> Vec<Vec<String>> {
    let mut visited = HashSet::new();
    let mut result = Vec::new();

    for v in g.nodes() {
        if visited.contains(&v) {
            continue;
        }
        let mut component = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(v.clone());
        visited.insert(v);

        while let Some(node) = queue.pop_front() {
            component.push(node.clone());
            if let Some(neighbors) = g.neighbors(&node) {
                for w in neighbors {
                    if !visited.contains(&w) {
                        visited.insert(w.clone());
                        queue.push_back(w);
                    }
                }
            }
        }
        result.push(component);
    }

    result
}

/// Dijkstra's single-source shortest paths.
/// Returns a map from node to (distance, predecessor).
pub fn dijkstra<N, E>(
    g: &Graph<N, E>,
    source: &str,
    weight_fn: impl Fn(&E) -> f64,
) -> HashMap<String, (f64, Option<String>)> {
    let mut dist: HashMap<String, (f64, Option<String>)> = HashMap::new();
    let mut pq = PriorityQueue::new();

    dist.insert(source.to_string(), (0.0, None));
    pq.insert(source.to_string(), 0.0);

    for v in g.nodes() {
        if v != source {
            dist.insert(v.clone(), (f64::INFINITY, None));
            pq.insert(v, f64::INFINITY);
        }
    }

    while let Some((u, d)) = pq.extract_min() {
        if d == f64::INFINITY {
            break;
        }
        let edges = if g.is_directed() {
            g.out_edges(&u, None)
        } else {
            g.node_edges(&u, None)
        };
        if let Some(edges) = edges {
            for e in edges {
                let w = if e.v == u { &e.w } else { &e.v };
                if let Some(label) = g.edge(&e.v, &e.w, e.name.as_deref()) {
                    let alt = d + weight_fn(label);
                    if alt < dist.get(w).map_or(f64::INFINITY, |d| d.0) {
                        dist.insert(w.clone(), (alt, Some(u.clone())));
                        pq.decrease(w, alt);
                    }
                }
            }
        }
    }

    dist
}

/// Prim's minimum spanning tree for undirected graphs.
/// Returns a new undirected graph representing the MST.
pub fn prim<N, E>(g: &Graph<N, E>, weight_fn: impl Fn(&E) -> f64) -> Graph<(), f64>
where
    N: Clone,
{
    let mut result = Graph::<(), f64>::with_options(super::GraphOptions {
        directed: false,
        multigraph: false,
        compound: false,
    });

    let nodes = g.nodes();
    if nodes.is_empty() {
        return result;
    }

    let start = &nodes[0];
    let mut in_mst = HashSet::new();
    let mut pq = PriorityQueue::new();

    for v in &nodes {
        result.set_node(v.clone(), None);
        pq.insert(v.clone(), f64::INFINITY);
    }
    pq.decrease(start, 0.0);

    let mut parents: HashMap<String, String> = HashMap::new();
    let mut weights: HashMap<String, f64> = HashMap::new();

    while let Some((u, _)) = pq.extract_min() {
        in_mst.insert(u.clone());

        if let Some(parent) = parents.get(&u) {
            let w = weights.get(&u).copied().unwrap_or(0.0);
            result.set_edge(parent.clone(), u.clone(), Some(w), None);
        }

        let edges = g.node_edges(&u, None);
        if let Some(edges) = edges {
            for e in edges {
                let w = if e.v == u { &e.w } else { &e.v };
                if in_mst.contains(w) {
                    continue;
                }
                if let Some(label) = g.edge(&e.v, &e.w, e.name.as_deref()) {
                    let edge_weight = weight_fn(label);
                    let current = weights.get(w).copied().unwrap_or(f64::INFINITY);
                    if edge_weight < current {
                        parents.insert(w.clone(), u.clone());
                        weights.insert(w.clone(), edge_weight);
                        pq.decrease(w, edge_weight);
                    }
                }
            }
        }
    }

    result
}

/// Simple priority queue for Dijkstra/Prim.
struct PriorityQueue {
    entries: HashMap<String, f64>,
}

impl PriorityQueue {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn insert(&mut self, key: String, priority: f64) {
        self.entries.insert(key, priority);
    }

    fn decrease(&mut self, key: &str, priority: f64) {
        if let Some(p) = self.entries.get_mut(key) {
            if priority < *p {
                *p = priority;
            }
        }
    }

    fn extract_min(&mut self) -> Option<(String, f64)> {
        if self.entries.is_empty() {
            return None;
        }
        let (key, priority) = self
            .entries
            .iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, v)| (k.clone(), *v))?;
        self.entries.remove(&key);
        Some((key, priority))
    }
}
