// src/main.rs

use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::algo::connected_components;
use itertools::Itertools;
use serde_json::json;

fn main() -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path("data/day_area.csv")?;
    let mut daily: HashMap<NaiveDate, HashSet<String>> = HashMap::new();

    for result in rdr.records() {
        let record = result?;
        let day = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")?;
        let area = record[1].to_string();
        daily.entry(day).or_default().insert(area);
    }

    let mut graph = UnGraph::<String, ()>::new_undirected();
    let mut idx_map: HashMap<String, NodeIndex> = HashMap::new();

    for areas in daily.values() {
        for area in areas {
            idx_map
                .entry(area.clone())
                .or_insert_with(|| graph.add_node(area.clone()));
        }
    }
    for areas in daily.values() {
        for (a, b) in areas.iter().tuple_combinations() {
            let ia = *idx_map.get(a).unwrap();
            let ib = *idx_map.get(b).unwrap();
            if graph.find_edge(ia, ib).is_none() {
                graph.add_edge(ia, ib, ());
            }
        }
    }

    println!(
        "Graph built: {} nodes, {} edges",
        graph.node_count(),
        graph.edge_count()
    );

    // ── Step 3: Degree distribution ─────────────────────────────────────────
    let mut degree_counts: HashMap<usize, usize> = HashMap::new();
    for node in graph.node_indices() {
        let deg = graph.neighbors(node).count();
        *degree_counts.entry(deg).or_default() += 1;
    }
    println!("Degree distribution (degree → count):");
    for (deg, cnt) in degree_counts.iter().sorted_by_key(|&(d, _)| *d) {
        println!("  {} → {}", deg, cnt);
    }

    let mut total_dist = 0u64;
    let mut total_pairs = 0u64;

    for start in graph.node_indices() {
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

        for &d in dist.values() {
            total_dist += d as u64;
            total_pairs += 1;
        }
    }

    let avg_path = total_dist as f64 / total_pairs as f64;
    println!("Avg shortest-path length: {:.3}", avg_path);

    let mut closeness: Vec<(String, f64)> = graph
        .node_indices()
        .map(|node| {
            let mut dist: HashMap<NodeIndex, usize> = HashMap::new();
            let mut queue: VecDeque<NodeIndex> = VecDeque::new();
            dist.insert(node, 0);
            queue.push_back(node);
            while let Some(n) = queue.pop_front() {
                let d = dist[&n];
                for nbr in graph.neighbors(n) {
                    if !dist.contains_key(&nbr) {
                        dist.insert(nbr, d + 1);
                        queue.push_back(nbr);
                    }
                }
            }
            let sum: f64 = dist.values().map(|&d| d as f64).sum();
            let c = if sum > 0.0 {
                (graph.node_count() as f64 - 1.0) / sum
            } else {
                0.0
            };
            (graph[node].clone(), c)
        })
        .collect();

    closeness.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let top5_close = closeness.into_iter().take(5).collect::<Vec<_>>();

    println!("Top 5 closeness centrality:");
    for (name, score) in &top5_close {
        println!("  {} → {:.4}", name, score);
    }

    let num_comms = connected_components(&graph);
    println!("Number of connected components: {}", num_comms);

    std::fs::create_dir_all("report")?;
    let metrics = json!({
        "nodes": graph.node_count(),
        "edges": graph.edge_count(),
        "avg_path": avg_path,
        "top_closeness": top5_close,
        "num_components": num_comms
    });
    std::fs::write("report/metrics.json", serde_json::to_string_pretty(&metrics)?)?;
    println!("report/metrics.json written");

    let mut wtr = csv::Writer::from_path("report/degree_counts.csv")?;
    wtr.write_record(&["degree","count"])?;
    for (deg, cnt) in degree_counts {
        wtr.write_record(&[deg.to_string(), cnt.to_string()])?;
    }
    wtr.flush()?;
    println!("report/degree_counts.csv written");

    Ok(())
}
