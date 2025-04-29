// src/graph.rs

use std::collections::{HashMap, VecDeque};
use chrono::NaiveDate;
use csv::ReaderBuilder;
use petgraph::graph::{UnGraph, NodeIndex};
use itertools::Itertools;

pub type Graph = UnGraph<(NaiveDate, String), ()>;

/// Build a graph where each unique (day, area) is a node,
/// and nodes are connected if they occur on the same day.
pub fn build_graph(path: &str) -> Result<Graph, Box<dyn std::error::Error>> {
    // 1) Read all (DAY, AREA_NAME) pairs
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(path)?;
    let mut entries = Vec::new();
    for result in rdr.records() {
        let record = result?;
        let day = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")?;
        let area = record[1].to_string();
        entries.push((day, area));
    }

    // 2) Create graph & index map
    let mut graph = Graph::new_undirected();
    let mut idx_map: HashMap<(NaiveDate, String), NodeIndex> = HashMap::new();
    for (day, area) in &entries {
        idx_map.entry((day.clone(), area.clone()))
            .or_insert_with(|| graph.add_node((day.clone(), area.clone())));
    }

    // 3) Group by day and fully connect each day's nodes
    let mut daily: HashMap<NaiveDate, Vec<NodeIndex>> = HashMap::new();
    for ((day, _), &idx) in &idx_map {
        daily.entry(day.clone()).or_default().push(idx);
    }
    for nodes in daily.values() {
        for (a, b) in nodes.iter().tuple_combinations() {
            if graph.find_edge(*a, *b).is_none() {
                graph.add_edge(*a, *b, ());
            }
        }
    }

    Ok(graph)
}

/// Perform a BFS from `start` and return a map of distances to every reachable node.
pub fn bfs_distances(
    graph: &Graph,
    start: NodeIndex,
) -> HashMap<NodeIndex, usize> {
    let mut dist: HashMap<NodeIndex, usize> = HashMap::new();
    let mut queue: VecDeque<NodeIndex> = VecDeque::new();

    dist.insert(start, 0);
    queue.push_back(start);

    while let Some(node) = queue.pop_front() {
        let d = dist[&node];
        for nbr in graph.neighbors(node) {
            if !dist.contains_key(&nbr) {
                dist.insert(nbr, d + 1);
                queue.push_back(nbr);
            }
        }
    }

    dist
}
