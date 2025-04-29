//! DS210 Final Project (Modular): LA Crime‐Graph Analysis
//!
//! - Builds graph from `data/day_area.csv` (one node per (DAY,AREA_NAME))
//! - Computes degree distribution, BFS‐based avg‐path, closeness, and components
//! - Exports `report/metrics.json` and `report/degree_counts.csv`

mod graph;
mod analysis;

use crate::graph::build_graph;
use crate::analysis::{
    degree_distribution,
    avg_shortest_path,
    closeness_centrality,
    component_count,
};
use itertools::Itertools;
use csv::Writer;
use serde_json::json;
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    // Build the graph
    let graph = build_graph("data/day_area.csv")?;
    println!(
        "Graph built: {} nodes, {} edges",
        graph.node_count(),
        graph.edge_count()
    );

    // Degree distribution
    let deg_dist = degree_distribution(&graph);
    println!("Degree distribution:");
    for (d, cnt) in deg_dist.iter().sorted_by_key(|&(d, _)| *d) {
        println!("  {} → {}", d, cnt);
    }

    // Average shortest-path length
    let avg = avg_shortest_path(&graph);
    println!("Avg shortest-path length: {:.3}", avg);

    // Top-5 closeness centrality
    let top5 = closeness_centrality(&graph, 5);
    println!("Top 5 closeness centrality:");
    for ((day, area), score) in &top5 {
        println!("  {} | {} → {:.4}", day, area, score);
    }

    // Connected components
    let comps = component_count(&graph);
    println!("Connected components: {}", comps);

    // Write JSON metrics
    fs::create_dir_all("report")?;
    let metrics = json!({
        "nodes": graph.node_count(),
        "edges": graph.edge_count(),
        "avg_path": avg,
        "top5_closeness": top5,
        "components": comps
    });
    fs::write(
        "report/metrics.json",
        serde_json::to_string_pretty(&metrics)?,
    )?;
    println!("report/metrics.json written");

    // Write degree_counts.csv
    let mut wtr = Writer::from_path("report/degree_counts.csv")?;
    wtr.write_record(&["degree", "count"])?;
    for (d, cnt) in deg_dist {
        wtr.write_record(&[d.to_string(), cnt.to_string()])?;
    }
    wtr.flush()?;
    println!("report/degree_counts.csv written");

    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::graph::{build_graph, bfs_distances, Graph};
    use crate::analysis::{degree_distribution, component_count};
    use chrono::NaiveDate;

    #[test]
    fn test_bfs_chain() {
        let mut g: Graph = Graph::new_undirected();
        let date = NaiveDate::from_ymd_opt(2025, 1, 1)
            .expect("valid date");
        let a = g.add_node((date, "A".to_string()));
        let b = g.add_node((date, "B".to_string()));
        let c = g.add_node((date, "C".to_string()));
        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        let dist = bfs_distances(&g, a);
        assert_eq!(dist.get(&a), Some(&0));
        assert_eq!(dist.get(&b), Some(&1));
        assert_eq!(dist.get(&c), Some(&2));
    }

    #[test]
    fn test_degree_distribution_triangle() {
        let mut g: Graph = Graph::new_undirected();
        let date = NaiveDate::from_ymd_opt(2025, 1, 1)
            .expect("valid date");
        let a = g.add_node((date, "A".to_string()));
        let b = g.add_node((date, "B".to_string()));
        let c = g.add_node((date, "C".to_string()));
        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        g.add_edge(c, a, ());
        let deg = degree_distribution(&g);
        assert_eq!(deg.get(&2), Some(&3));
    }

    #[test]
    fn test_component_count_isolated() {
        let mut g: Graph = Graph::new_undirected();
        let date = NaiveDate::from_ymd_opt(2025, 1, 1)
            .expect("valid date");
        g.add_node((date, "X".to_string()));
        g.add_node((date, "Y".to_string()));
        assert_eq!(component_count(&g), 2);
    }

    #[test]
    fn test_build_graph_tiny() {
        let data = "DAY,AREA_NAME\n\
                    2025-04-01,A\n\
                    2025-04-01,B\n\
                    2025-04-02,A\n";
        let tmp = std::env::temp_dir().join("test_day_area.csv");
        std::fs::write(&tmp, data).unwrap();
        let graph: Graph = build_graph(tmp.to_str().unwrap()).unwrap();
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 1);
    }
}
