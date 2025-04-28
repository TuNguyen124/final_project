//! DS210 Final Project: LA Crime-Graph Analysis
//!
//! Loads deduped (DAY,AREA_NAME) data from `data/day_area.csv`,
//! builds an undirected neighborhood graph, computes degree distribution,
//! average shortest-paths via BFS, closeness centrality, and connected components.
//! Exports `report/metrics.json` and `report/degree_counts.csv`.

use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use chrono::NaiveDate;
use csv::{ReaderBuilder, Writer};
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::algo::connected_components;
use itertools::Itertools;
use serde_json::json;

/// Compute shortest-path distances from `start` to all reachable nodes via BFS.
///
/// # Inputs
/// - `graph`: the undirected neighborhood graph
/// - `start`: the NodeIndex to begin from
///
/// # Returns
/// A map from each reachable node to its distance (in hops) from `start`.
fn bfs_distances(
    graph: &UnGraph<String, ()>,
    start: NodeIndex,
) -> HashMap<NodeIndex, usize> {
    let mut dist: HashMap<NodeIndex, usize> = HashMap::new();
    let mut queue: VecDeque<NodeIndex> = VecDeque::new();

    // distance to self is 0
    dist.insert(start, 0);
    queue.push_back(start);

    // breadth-first search
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

/// Main pipeline:
/// 1) Read `data/day_area.csv` into `daily` map.
/// 2) Build an undirected graph of neighborhoods.
/// 3) Compute degree distribution.
/// 4) Compute average shortest-path (all-pairs via BFS).
/// 5) Compute closeness centrality (top 5).
/// 6) Count connected components.
/// 7) Write out JSON & CSV reports.
fn main() -> Result<(), Box<dyn Error>> {
    // Step 1: Read deduped (DAY, AREA) pairs
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path("data/day_area.csv")?;
    let mut daily: HashMap<NaiveDate, HashSet<String>> = HashMap::new();
    for result in rdr.records() {
        let record = result?;
        // record[0] = "YYYY-MM-DD", record[1] = AREA_NAME
        let day = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")?;
        let area = record[1].to_string();
        daily.entry(day).or_default().insert(area);
    }

    // Step 2: Build the graph
    let mut graph = UnGraph::<String, ()>::new_undirected();
    let mut idx_map: HashMap<String, NodeIndex> = HashMap::new();

    // 2a) Add one node per neighborhood
    for areas in daily.values() {
        for area in areas {
            idx_map
                .entry(area.clone())
                .or_insert_with(|| graph.add_node(area.clone()));
        }
    }
    // 2b) Connect co-occurring neighborhoods
    for areas in daily.values() {
        for (a, b) in areas.iter().tuple_combinations() {
            let ia = *idx_map.get(a).unwrap();
            let ib = *idx_map.get(b).unwrap();
            if graph.find_edge(ia, ib).is_none() {
                graph.add_edge(ia, ib, ());
            }
        }
    }
    println!("Graph built: {} nodes, {} edges",
             graph.node_count(), graph.edge_count());

    // Step 3: Degree distribution
    let mut degree_counts: HashMap<usize, usize> = HashMap::new();
    for node in graph.node_indices() {
        let deg = graph.neighbors(node).count();
        *degree_counts.entry(deg).or_default() += 1;
    }
    println!("Degree distribution (degree → count):");
    for (deg, cnt) in degree_counts.iter().sorted_by_key(|&(d, _)| *d) {
        println!("  {} → {}", deg, cnt);
    }

    // Step 4: Average shortest-path length via BFS for each start
    let mut total_dist = 0u64;
    let mut total_pairs = 0u64;
    for start in graph.node_indices() {
        let dist_map = bfs_distances(&graph, start);
        for &d in dist_map.values() {
            total_dist += d as u64;
            total_pairs += 1;
        }
    }
    let avg_path = total_dist as f64 / total_pairs as f64;
    println!("Avg shortest-path length: {:.3}", avg_path);

    // Step 5: Closeness centrality (top 5)
    let mut closeness: Vec<(String, f64)> = graph
        .node_indices()
        .map(|node| {
            let dm = bfs_distances(&graph, node);
            let sum: f64 = dm.values().map(|&d| d as f64).sum();
            let c = if sum > 0.0 {
                (graph.node_count() as f64 - 1.0) / sum
            } else {
                0.0
            };
            (graph[node].clone(), c)
        })
        .collect();
    closeness.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let top5_close = closeness.iter().take(5).cloned().collect::<Vec<_>>();
    println!("Top 5 closeness centrality:");
    for (name, score) in &top5_close {
        println!("  {} = {:.4}", name, score);
    }

    // Step 6: Connected components
    let num_comms = connected_components(&graph);
    println!("Number of connected components: {}", num_comms);

    // Step 7: Write reports
    std::fs::create_dir_all("report")?;
    // 7a) metrics.json
    let metrics = json!({
        "nodes": graph.node_count(),
        "edges": graph.edge_count(),
        "avg_path": avg_path,
        "top_closeness": top5_close,
        "num_components": num_comms
    });
    std::fs::write(
        "report/metrics.json",
        serde_json::to_string_pretty(&metrics)?,
    )?;
    println!("report/metrics.json written");

    // 7b) degree_counts.csv
    let mut wtr = Writer::from_path("report/degree_counts.csv")?;
    wtr.write_record(&["degree","count"])?;
    for (deg, cnt) in degree_counts {
        wtr.write_record(&[deg.to_string(), cnt.to_string()])?;
    }
    wtr.flush()?;
    println!("report/degree_counts.csv written");

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::algo::connected_components;

    #[test]
    fn test_bfs_chain() {

        let mut g = UnGraph::<String, ()>::new_undirected();
        let a = g.add_node("A".into());
        let b = g.add_node("B".into());
        let c = g.add_node("C".into());
        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        let dist = bfs_distances(&g, a);
        assert_eq!(dist.get(&a), Some(&0));
        assert_eq!(dist.get(&b), Some(&1));
        assert_eq!(dist.get(&c), Some(&2));
    }

    #[test]
    fn test_connected_components() {
    
        let mut g = UnGraph::<String, ()>::new_undirected();
        g.add_node("X".into());
        g.add_node("Y".into());
        assert_eq!(connected_components(&g), 2);
    }
}
