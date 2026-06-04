# ternary-network

Network science for ternary-weighted graphs — shortest paths, clustering, centrality, community detection, and small-world detection with edges constrained to {-1, 0, +1}.

## Why This Exists

Real-world networks carry signed relationships: trust/distrust, allies/enemies, promote/inhibit. Most graph libraries reduce this to positive weights, losing critical structural information. This crate models networks where every edge carries one of three values: **negative** (-1, antagonism), **zero** (0, no connection), or **positive** (+1, cooperation). This ternary edge model captures the fundamental triad of social, biological, and economic interactions without the noise of continuous weights.

The ternary constraint isn't just simplification — it's a feature. When edges can only be {-1, 0, +1}, structural balance theory applies directly: a triangle with an odd number of negative edges is "balanced" (stable), while unbalanced triangles create tension. This crate computes the classic network science metrics (degree, clustering, centrality, modularity) adapted for signed, ternary-weighted graphs.

This crate is part of the **Negative Space Intelligence** ecosystem.

## Core Concepts

- **TernaryWeight** — Edge weight: `Negative` (-1), `Zero` (0), or `Positive` (+1). Represents antagonism, neutrality, or cooperation.
- **TernaryNetwork** — Adjacency-list graph with ternary-weighted edges. Supports directed and undirected modes.
- **Degree Metrics** — Standard degree, positive degree, and negative degree per node. Full degree distribution.
- **Clustering Coefficient** — Per-node and average, counting triangles among neighbors.
- **Shortest Path** — BFS-based pathfinding with ternary cost: Positive/Negative edges cost 1, Zero edges cost 0 (free).
- **Community Detection** — Label propagation algorithm adapted for ternary networks.
- **Centrality** — Betweenness centrality (bridge detection) and closeness centrality (reachability).
- **Modularity** — Newman-style modularity for evaluating community structure.
- **Small-World Detection** — Identifies networks with high clustering and short average path length.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-network = "0.1"
```

```rust
use ternary_network::*;

// Build a ternary-weighted network
let mut net = TernaryNetwork::new(false); // undirected
net.add_edge(0, 1, TernaryWeight::Positive);  // allies
net.add_edge(1, 2, TernaryWeight::Positive);
net.add_edge(0, 2, TernaryWeight::Positive);  // triangle
net.add_edge(2, 3, TernaryWeight::Negative);  // antagonism

// Degree analysis
assert_eq!(net.positive_degree(0), 2);
assert_eq!(net.negative_degree(2), 1);

// Clustering coefficient (node 0: 2 neighbors, 1 triangle → 1.0)
let cc = net.clustering_coefficient(0);
assert!((cc - 1.0).abs() < 1e-9);

// Shortest path
let (path, cost) = net.shortest_path(0, 3).unwrap();
assert!(path.contains(&2));

// Community detection
let communities = net.detect_communities();

// Centrality: node 2 is the bridge
let bc = net.betweenness_centrality(2);

// Modularity of a community assignment
let mut comms = std::collections::HashMap::new();
comms.insert(0, 0); comms.insert(1, 0);
comms.insert(2, 0); comms.insert(3, 1);
let q = net.modularity(&comms);

// Directed network
let mut directed = TernaryNetwork::new(true);
directed.add_edge(0, 1, TernaryWeight::Positive);
directed.add_edge(1, 0, TernaryWeight::Negative); // asymmetric
```

## API Overview

### TernaryNetwork
| Method | Description |
|---|---|
| `new(directed)` | Create empty graph |
| `add_edge(from, to, weight)` | Add ternary-weighted edge |
| `node_count()` / `edge_count()` | Size queries |
| `degree_distribution()` | Histogram: degree → count |
| `positive_degree(node)` / `negative_degree(node)` | Signed degree |
| `clustering_coefficient(node)` | Local triangle density |
| `avg_clustering_coefficient()` | Network-wide average |
| `shortest_path(from, to)` | BFS with ternary costs → (path, cost) |
| `detect_communities()` | Label propagation → node → community |
| `modularity(communities)` | Community quality score |
| `betweenness_centrality(node)` | Bridge importance |
| `closeness_centrality(node)` | Reachability score |
| `is_small_world()` | High clustering + short paths |

## How It Works

The network uses an adjacency list representation where each edge carries a `TernaryWeight`. For undirected graphs, both directions are stored automatically. Path costs treat `Zero` edges as free (cost 0), while `Positive` and `Negative` edges both cost 1 — this reflects the fact that antagonistic relationships are still connections that can be traversed.

Community detection uses iterative label propagation: each node adopts the most common label among its neighbors, repeating until stable. This is fast (near-linear time) and produces reasonable partitions, though results can vary with node ordering. Modularity scoring follows Newman's formulation: Q = (1/2m) Σ[Aᵢⱼ − kᵢkⱼ/(2m)] δ(cᵢ,cⱼ), adapted for the ternary adjacency matrix.

Betweenness centrality counts how many shortest paths pass through a given node, normalized by the total possible. Closeness centrality measures the inverse average distance to all reachable nodes. Both metrics reveal structural importance in signed networks.

## Use Cases

1. **Social network analysis** — Model trust/distrust networks where positive edges are friendships and negative edges are adversarial relationships. Structural balance theory predicts stability from the sign patterns.

2. **Biological regulatory networks** — Gene regulation involves activation (+1) and inhibition (-1). Ternary edges model these pathways naturally, with community detection revealing functional modules.

3. **Economic networks** — Supply chain relationships as ternary signals (complementary/neutral/competitive). Centrality identifies critical suppliers; modularity reveals trade blocs.

4. **Security and access control** — Model allow/deny/neutral permissions as ternary edges in a policy graph. Shortest path analysis reveals transitive trust chains.

## Ecosystem

| Crate | Relationship |
|---|---|
| `ternary-cell` | Cell signaling produces ternary networks between cells |
| `ternary-attention` | Attention weights can be modeled as ternary graph edges |
| `ternary-locks` | Lock dependency graphs are a specialized form of ternary network |
| `ternary-bayesian` | Bayesian network structure relates to graph topology |
| `ternary-econ` | Market agent interactions form ternary-weighted networks |

## License

MIT
