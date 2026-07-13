//! Smoke test that mirrors the README "Quick Start" example verbatim.
//!
//! Guards against the documented API drifting from the real implementation:
//! every public item referenced in the README (TernaryNetwork::new,
//! add_edge, positive_degree, negative_degree, clustering_coefficient,
//! shortest_path, detect_communities, betweenness_centrality, modularity,
//! TernaryWeight) must exist and behave as documented.

use std::collections::HashMap;

use ternary_network::{TernaryNetwork, TernaryWeight};

#[test]
fn readme_quick_start_undirected() {
    // Build a ternary-weighted network
    let mut net = TernaryNetwork::new(false); // undirected
    net.add_edge(0, 1, TernaryWeight::Positive); // allies
    net.add_edge(1, 2, TernaryWeight::Positive);
    net.add_edge(0, 2, TernaryWeight::Positive); // triangle
    net.add_edge(2, 3, TernaryWeight::Negative); // antagonism

    // Degree analysis
    assert_eq!(net.positive_degree(0), 2);
    assert_eq!(net.negative_degree(2), 1);

    // Clustering coefficient (node 0: 2 neighbors, 1 triangle -> 1.0)
    let cc = net.clustering_coefficient(0);
    assert!((cc - 1.0).abs() < 1e-9);

    // Shortest path
    let (path, cost) = net.shortest_path(0, 3).unwrap();
    assert!(path.contains(&2));
    // README doesn't assert cost, but 0->2->3 is two unit-cost edges.
    assert_eq!(cost, 2);
    assert_eq!(path.first(), Some(&0));
    assert_eq!(path.last(), Some(&3));

    // Community detection returns a complete labeling.
    let communities = net.detect_communities();
    assert_eq!(communities.len(), net.node_count());

    // Centrality: node 2 is the bridge
    let bc = net.betweenness_centrality(2);
    assert!(bc >= 0.0);

    // Modularity of a community assignment
    let mut comms = HashMap::new();
    comms.insert(0, 0);
    comms.insert(1, 0);
    comms.insert(2, 0);
    comms.insert(3, 1);
    let q = net.modularity(&comms);
    assert!(q.is_finite());
}

#[test]
fn readme_quick_start_directed() {
    // Directed network
    let mut directed = TernaryNetwork::new(true);
    directed.add_edge(0, 1, TernaryWeight::Positive);
    directed.add_edge(1, 0, TernaryWeight::Negative); // asymmetric

    // Asymmetric adjacency must be preserved in directed mode:
    // 0 -> 1 is Positive, 1 -> 0 is Negative, so the two directions differ.
    assert_eq!(directed.positive_degree(0), 1);
    assert_eq!(directed.negative_degree(0), 0);
    assert_eq!(directed.positive_degree(1), 0);
    assert_eq!(directed.negative_degree(1), 1);
    assert_eq!(directed.edge_count(), 2);
}

#[test]
fn readme_api_overview_methods_exist() {
    // Exercises every method listed in the README "API Overview" table.
    let mut net = TernaryNetwork::new(false);
    net.add_edge(0, 1, TernaryWeight::Positive);
    net.add_edge(1, 2, TernaryWeight::Zero);
    net.add_edge(0, 2, TernaryWeight::Negative);

    let _ = net.node_count();
    let _ = net.edge_count();
    let _ = net.degree_distribution();
    assert_eq!(net.positive_degree(0), 1);
    assert_eq!(net.negative_degree(0), 1);
    let _ = net.clustering_coefficient(0);
    let _ = net.avg_clustering_coefficient();
    let _ = net.shortest_path(0, 2);
    let _ = net.detect_communities();
    let comms: HashMap<usize, usize> = net
        .nodes()
        .iter()
        .enumerate()
        .map(|(i, &n)| (n, i))
        .collect();
    let _ = net.modularity(&comms);
    let _ = net.betweenness_centrality(1);
    let _ = net.closeness_centrality(1);
    let _ = net.is_small_world();
}
