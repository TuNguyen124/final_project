// src/analysis.rs

use crate::graph::{Graph, bfs_distances};
use petgraph::algo::connected_components;
use std::collections::HashMap;

/// Returns a map: degree → count of nodes with that degree.
pub fn degree_distribution(graph: &Graph) -> HashMap<usize, usize> {
    let mut counts = HashMap::new();
    for node in graph.node_indices() {
        let deg = graph.neighbors(node).count();
        *counts.entry(deg).or_default() += 1;
    }
    counts
}

/// Computes the average shortest-path length (all-pairs) via BFS.
pub fn avg_shortest_path(graph: &Graph) -> f64 {
    let mut total = 0u64;
    let mut pairs = 0u64;
    for start in graph.node_indices() {
        let dm = bfs_distances(graph, start);
        for &d in dm.values() {
            total += d as u64;
            pairs += 1;
        }
    }
    total as f64 / pairs as f64
}

/// Computes closeness centrality for all nodes, returns the top `n` highest.
pub fn closeness_centrality(
    graph: &Graph,
    n: usize,
) -> Vec<((String, String), f64)> {
    let mut scores: Vec<((String, String), f64)> = graph
        .node_indices()
        .map(|node| {
            let dm = bfs_distances(graph, node);
            let sum: f64 = dm.values().map(|&d| d as f64).sum();
            let score = if sum > 0.0 {
                (graph.node_count() as f64 - 1.0) / sum
            } else {
                0.0
            };
            let (day, area) = &graph[node];
            ((day.to_string(), area.clone()), score)
        })
        .collect();
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scores.into_iter().take(n).collect()
}

/// Returns the number of connected components in the graph.
pub fn component_count(graph: &Graph) -> usize {
    connected_components(graph)
}
