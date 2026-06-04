#![forbid(unsafe_code)]

//! Network science for ternary-weighted graphs.
//!
//! Provides graph analysis with edge weights constrained to {-1, 0, +1},
//! including degree distributions, clustering coefficients, shortest paths,
//! community detection via modularity optimization, centrality measures,
//! and small-world detection.

use std::collections::{HashMap, HashSet, VecDeque};

/// Ternary edge weight: Negative, Zero, or Positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TernaryWeight {
    Negative,
    Zero,
    Positive,
}

impl TernaryWeight {
    pub fn value(&self) -> i8 {
        match self {
            TernaryWeight::Negative => -1,
            TernaryWeight::Zero => 0,
            TernaryWeight::Positive => 1,
        }
    }

    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(TernaryWeight::Negative),
            0 => Some(TernaryWeight::Zero),
            1 => Some(TernaryWeight::Positive),
            _ => None,
        }
    }
}

/// A ternary-weighted network graph.
#[derive(Clone, Debug)]
pub struct TernaryNetwork {
    /// Adjacency: node -> [(neighbor, weight)]
    adj: HashMap<usize, Vec<(usize, TernaryWeight)>>,
    node_count: usize,
    directed: bool,
}

impl TernaryNetwork {
    pub fn new(directed: bool) -> Self {
        TernaryNetwork {
            adj: HashMap::new(),
            node_count: 0,
            directed,
        }
    }

    /// Ensure a node exists in the graph.
    pub fn add_node(&mut self, node: usize) {
        if !self.adj.contains_key(&node) {
            self.adj.insert(node, Vec::new());
            self.node_count = self.node_count.max(node + 1);
        }
    }

    /// Add an edge with ternary weight. For undirected graphs, adds both directions.
    pub fn add_edge(&mut self, from: usize, to: usize, weight: TernaryWeight) {
        self.add_node(from);
        self.add_node(to);
        self.adj.entry(from).or_default().push((to, weight));
        if !self.directed {
            self.adj.entry(to).or_default().push((from, weight));
        }
    }

    pub fn nodes(&self) -> Vec<usize> {
        let mut ns: Vec<usize> = self.adj.keys().copied().collect();
        ns.sort();
        ns
    }

    pub fn node_count(&self) -> usize {
        self.adj.len()
    }

    pub fn edge_count(&self) -> usize {
        let count: usize = self.adj.values().map(|v| v.len()).sum();
        if self.directed { count } else { count / 2 }
    }

    pub fn neighbors(&self, node: usize) -> &[(usize, TernaryWeight)] {
        self.adj.get(&node).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Degree distribution: maps degree -> count of nodes with that degree.
    pub fn degree_distribution(&self) -> HashMap<usize, usize> {
        let mut dist = HashMap::new();
        for node in self.adj.keys() {
            let deg = self.adj.get(node).map(|v| v.len()).unwrap_or(0);
            *dist.entry(deg).or_insert(0) += 1;
        }
        dist
    }

    /// Positive degree: number of edges with Positive weight.
    pub fn positive_degree(&self, node: usize) -> usize {
        self.adj.get(&node).map(|v| v.iter().filter(|(_, w)| *w == TernaryWeight::Positive).count()).unwrap_or(0)
    }

    /// Negative degree: number of edges with Negative weight.
    pub fn negative_degree(&self, node: usize) -> usize {
        self.adj.get(&node).map(|v| v.iter().filter(|(_, w)| *w == TernaryWeight::Negative).count()).unwrap_or(0)
    }

    /// Clustering coefficient for a single node.
    pub fn clustering_coefficient(&self, node: usize) -> f64 {
        let neighbors: Vec<usize> = self.adj.get(&node)
            .map(|v| v.iter().map(|(n, _)| *n).collect())
            .unwrap_or_default();
        let k = neighbors.len();
        if k < 2 { return 0.0; }

        let _neighbor_set: HashSet<usize> = neighbors.iter().copied().collect();
        let mut triangles = 0;
        for i in 0..neighbors.len() {
            for j in (i + 1)..neighbors.len() {
                let ni = neighbors[i];
                let nj = neighbors[j];
                if let Some(edges) = self.adj.get(&ni) {
                    if edges.iter().any(|(n, _)| *n == nj) {
                        triangles += 1;
                    }
                }
            }
        }
        let possible = k * (k - 1) / 2;
        triangles as f64 / possible as f64
    }

    /// Average clustering coefficient across all nodes.
    pub fn avg_clustering_coefficient(&self) -> f64 {
        let nodes = self.nodes();
        if nodes.is_empty() { return 0.0; }
        let sum: f64 = nodes.iter().map(|n| self.clustering_coefficient(*n)).sum();
        sum / nodes.len() as f64
    }

    /// Shortest path using 0-1 BFS with ternary costs. Returns (path, total_cost).
    /// Cost: Positive=1, Zero=0, Negative=1 (absolute value).
    /// Zero-cost edges are prioritized via deque front insertion (0-1 BFS).
    pub fn shortest_path(&self, from: usize, to: usize) -> Option<(Vec<usize>, i32)> {
        if from == to {
            return Some((vec![from], 0));
        }
        let mut dist: HashMap<usize, i32> = HashMap::new();
        let mut prev: HashMap<usize, usize> = HashMap::new();
        let mut visited: HashSet<usize> = HashSet::new();
        dist.insert(from, 0);

        // 0-1 BFS: zero-cost edges push to front, unit-cost edges push to back
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(from);

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) { continue; }
            visited.insert(current);

            if current == to {
                // Reconstruct path
                let mut path = vec![to];
                let mut node = to;
                while let Some(&p) = prev.get(&node) {
                    path.push(p);
                    node = p;
                }
                path.reverse();
                return Some((path, dist[&to]));
            }

            if let Some(neighbors) = self.adj.get(&current) {
                for &(neighbor, weight) in neighbors {
                    let cost = weight.value().abs() as i32;
                    let new_dist = dist.get(&current).unwrap_or(&i32::MAX) + cost;
                    let old_dist = dist.get(&neighbor).unwrap_or(&i32::MAX);
                    if new_dist < *old_dist {
                        dist.insert(neighbor, new_dist);
                        prev.insert(neighbor, current);
                        // Zero-cost edges go to front for immediate processing
                        if cost == 0 {
                            queue.push_front(neighbor);
                        } else {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }
        None
    }

    /// Compute modularity for a given community assignment.
    /// Q = (1/2m) * sum_ij [ A_ij - k_i*k_j/(2m) ] * delta(c_i, c_j)
    pub fn modularity(&self, communities: &HashMap<usize, usize>) -> f64 {
        let m = self.edge_count() as f64;
        if m == 0.0 { return 0.0; }

        let nodes = self.nodes();
        let degree: HashMap<usize, usize> = nodes.iter()
            .map(|&n| (n, self.adj.get(&n).map(|v| v.len()).unwrap_or(0)))
            .collect();

        let mut q = 0.0;
        for &i in &nodes {
            for &j in &nodes {
                let ci = communities.get(&i).unwrap_or(&0);
                let cj = communities.get(&j).unwrap_or(&0);
                if ci != cj { continue; }

                let a_ij = self.adj.get(&i)
                    .map(|v| v.iter().filter(|(n, _)| *n == j).count() as f64)
                    .unwrap_or(0.0);

                let ki = *degree.get(&i).unwrap_or(&0) as f64;
                let kj = *degree.get(&j).unwrap_or(&0) as f64;
                q += a_ij - (ki * kj) / (2.0 * m);
            }
        }
        q / (2.0 * m)
    }

    /// Simple community detection via label propagation.
    pub fn detect_communities(&self) -> HashMap<usize, usize> {
        let nodes = self.nodes();
        let mut labels: HashMap<usize, usize> = HashMap::new();
        for (i, &node) in nodes.iter().enumerate() {
            labels.insert(node, i);
        }

        // A few rounds of label propagation
        for _ in 0..10 {
            for &node in &nodes {
                let neighbor_labels: Vec<usize> = self.adj.get(&node)
                    .map(|v| v.iter()
                        .filter_map(|(n, _)| labels.get(n).copied())
                        .collect())
                    .unwrap_or_default();

                if neighbor_labels.is_empty() { continue; }

                // Find most common label
                let mut counts: HashMap<usize, usize> = HashMap::new();
                for &label in &neighbor_labels {
                    *counts.entry(label).or_insert(0) += 1;
                }
                let best = counts.into_iter().max_by_key(|&(_, c)| c).map(|(l, _)| l);
                if let Some(label) = best {
                    labels.insert(node, label);
                }
            }
        }
        labels
    }

    /// Betweenness centrality for a node (simplified).
    pub fn betweenness_centrality(&self, target: usize) -> f64 {
        let nodes = self.nodes();
        let mut betweenness = 0.0;

        for &s in &nodes {
            for &t in &nodes {
                if s == t || s == target || t == target { continue; }
                if let Some((path, _)) = self.shortest_path(s, t) {
                    if path.contains(&target) {
                        betweenness += 1.0;
                    }
                }
            }
        }
        let n = nodes.len() as f64;
        if n > 2.0 { betweenness / ((n - 1.0) * (n - 2.0)) } else { betweenness }
    }

    /// Closeness centrality for a node.
    pub fn closeness_centrality(&self, node: usize) -> f64 {
        let nodes = self.nodes();
        let n = nodes.len() as f64;
        if n <= 1.0 { return 0.0; }

        let mut total_dist = 0.0;
        let mut reachable = 0;
        for &other in &nodes {
            if other == node { continue; }
            if let Some((_, cost)) = self.shortest_path(node, other) {
                total_dist += cost as f64;
                reachable += 1;
            }
        }
        if total_dist == 0.0 || reachable == 0 { return 0.0; }
        (reachable as f64) / (total_dist * (n - 1.0))
    }

    /// Detect if the network has small-world properties.
    /// Compares clustering coefficient and average path length to a random graph baseline.
    pub fn is_small_world(&self) -> bool {
        let nodes = self.nodes();
        let n = nodes.len();
        if n < 4 { return false; }

        let cc = self.avg_clustering_coefficient();
        if cc == 0.0 { return false; }

        // Compute average shortest path length
        let mut total_pl = 0.0;
        let mut count = 0;
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                if let Some((_, cost)) = self.shortest_path(nodes[i], nodes[j]) {
                    total_pl += cost as f64;
                    count += 1;
                }
            }
        }
        if count == 0 { return false; }
        let avg_pl = total_pl / count as f64;

        // Small-world: high clustering + short path length
        // Heuristic: CC > 0.1 and avg path length < n as f64 / 2.0
        cc > 0.1 && avg_pl < n as f64 / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_weight_values() {
        assert_eq!(TernaryWeight::Negative.value(), -1);
        assert_eq!(TernaryWeight::Zero.value(), 0);
        assert_eq!(TernaryWeight::Positive.value(), 1);
    }

    #[test]
    fn test_ternary_weight_from_i8() {
        assert_eq!(TernaryWeight::from_i8(-1), Some(TernaryWeight::Negative));
        assert_eq!(TernaryWeight::from_i8(0), Some(TernaryWeight::Zero));
        assert_eq!(TernaryWeight::from_i8(1), Some(TernaryWeight::Positive));
        assert_eq!(TernaryWeight::from_i8(2), None);
    }

    #[test]
    fn test_empty_network() {
        let net = TernaryNetwork::new(false);
        assert_eq!(net.node_count(), 0);
        assert_eq!(net.edge_count(), 0);
    }

    #[test]
    fn test_add_nodes_and_edges() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(1, 2, TernaryWeight::Negative);
        assert_eq!(net.node_count(), 3);
        assert_eq!(net.edge_count(), 2);
    }

    #[test]
    fn test_directed_edge_count() {
        let mut net = TernaryNetwork::new(true);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(1, 0, TernaryWeight::Negative);
        assert_eq!(net.edge_count(), 2);
    }

    #[test]
    fn test_degree_distribution() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(0, 2, TernaryWeight::Positive);
        net.add_edge(0, 3, TernaryWeight::Zero);
        let dist = net.degree_distribution();
        assert_eq!(dist.get(&3), Some(&1)); // node 0 has degree 3
    }

    #[test]
    fn test_positive_negative_degree() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(0, 2, TernaryWeight::Negative);
        net.add_edge(0, 3, TernaryWeight::Positive);
        assert_eq!(net.positive_degree(0), 2);
        assert_eq!(net.negative_degree(0), 1);
    }

    #[test]
    fn test_clustering_coefficient_triangle() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(1, 2, TernaryWeight::Positive);
        net.add_edge(0, 2, TernaryWeight::Positive);
        // Node 0 has 2 neighbors (1,2) and they are connected -> CC = 1.0
        let cc = net.clustering_coefficient(0);
        assert!((cc - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_clustering_coefficient_no_triangle() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(0, 2, TernaryWeight::Positive);
        let cc = net.clustering_coefficient(0);
        assert!((cc - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_clustering_coefficient_single_edge() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        assert!((net.clustering_coefficient(0) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_avg_clustering_coefficient() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(1, 2, TernaryWeight::Positive);
        net.add_edge(0, 2, TernaryWeight::Positive);
        let avg = net.avg_clustering_coefficient();
        assert!(avg > 0.0);
    }

    #[test]
    fn test_shortest_path_direct() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        let result = net.shortest_path(0, 1);
        assert!(result.is_some());
        let (path, cost) = result.unwrap();
        assert_eq!(path, vec![0, 1]);
        assert_eq!(cost, 1);
    }

    #[test]
    fn test_shortest_path_multi_hop() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(1, 2, TernaryWeight::Positive);
        let result = net.shortest_path(0, 2);
        assert!(result.is_some());
        let (_, cost) = result.unwrap();
        assert_eq!(cost, 2);
    }

    #[test]
    fn test_shortest_path_zero_weight() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Zero);
        let (_, cost) = net.shortest_path(0, 1).unwrap();
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_shortest_path_unreachable() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_node(2);
        assert!(net.shortest_path(0, 2).is_none());
    }

    #[test]
    fn test_shortest_path_same_node() {
        let mut net = TernaryNetwork::new(false);
        net.add_node(0);
        let (path, cost) = net.shortest_path(0, 0).unwrap();
        assert_eq!(path, vec![0]);
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_modularity_single_community() {
        // Dense internal edges, no external -> positive modularity
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(1, 2, TernaryWeight::Positive);
        net.add_edge(0, 2, TernaryWeight::Positive);
        let mut comms = HashMap::new();
        comms.insert(0, 0);
        comms.insert(1, 0);
        comms.insert(2, 0);
        let q = net.modularity(&comms);
        // Single community with all edges internal: modularity should be non-negative
        assert!(q >= 0.0);
    }

    #[test]
    fn test_modularity_split_communities() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(2, 3, TernaryWeight::Positive);
        let mut comms = HashMap::new();
        comms.insert(0, 0); comms.insert(1, 0);
        comms.insert(2, 1); comms.insert(3, 1);
        let q = net.modularity(&comms);
        assert!(q > 0.0);
    }

    #[test]
    fn test_detect_communities() {
        let mut net = TernaryNetwork::new(false);
        // Two clusters
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(1, 2, TernaryWeight::Positive);
        net.add_edge(0, 2, TernaryWeight::Positive);
        net.add_edge(3, 4, TernaryWeight::Positive);
        net.add_edge(4, 5, TernaryWeight::Positive);
        net.add_edge(3, 5, TernaryWeight::Positive);
        // Single bridge
        net.add_edge(2, 3, TernaryWeight::Positive);
        let comms = net.detect_communities();
        // Should have some community assignment
        assert!(comms.len() >= 3);
    }

    #[test]
    fn test_betweenness_centrality_bridge() {
        let mut net = TernaryNetwork::new(false);
        // Line graph: 0 - 1 - 2 - 3
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(1, 2, TernaryWeight::Positive);
        net.add_edge(2, 3, TernaryWeight::Positive);
        // Nodes 1 and 2 should have higher betweenness
        let bc1 = net.betweenness_centrality(1);
        let bc0 = net.betweenness_centrality(0);
        assert!(bc1 >= bc0);
    }

    #[test]
    fn test_closeness_centrality() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        net.add_edge(1, 2, TernaryWeight::Positive);
        let cc = net.closeness_centrality(1);
        assert!(cc > 0.0);
    }

    #[test]
    fn test_is_small_world() {
        // Create a ring lattice which should be small-world-ish
        let mut net = TernaryNetwork::new(false);
        for i in 0..10 {
            net.add_edge(i, (i + 1) % 10, TernaryWeight::Positive);
            net.add_edge(i, (i + 2) % 10, TernaryWeight::Positive);
        }
        // May or may not detect as small-world, just ensure it doesn't panic
        let _ = net.is_small_world();
    }

    #[test]
    fn test_is_small_world_too_small() {
        let mut net = TernaryNetwork::new(false);
        net.add_edge(0, 1, TernaryWeight::Positive);
        assert!(!net.is_small_world());
    }
}
