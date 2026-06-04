# Future Integration: ternary-network

## Current State
Provides ternary-weighted graph analysis with edge weights in {-1, 0, +1}, including community detection, centrality measures, shortest paths, and small-world detection on ternary networks.

## Integration Opportunities

### With ternary-room / construct-core
Room connectivity is naturally a ternary graph: rooms can have positive connections (shared resources), zero connections (unrelated), or negative connections (mutual exclusion). `TernaryNetwork` can model which rooms an agent can walk between, with `TernaryWeight::Negative` encoding rooms that conflict (e.g., a "debug" room and a "production" room). Community detection via modularity optimization directly maps to room clustering — which rooms form natural neighborhoods in PLATO?

### With ternary-cell
Cell grids use `TernaryMessenger` for inter-cell signaling. `TernaryNetwork` provides the topology those messages traverse. A cell grid IS a ternary network where adjacency weights encode influence strength. Community detection on the cell grid identifies emergent agent coalitions.

### With ternary-protocol
Protocol messages route between agents. Network-level routing using shortest-path on the ternary graph would enable efficient message delivery with awareness of positive/negative relationships between agents.

## Potential in Mature Systems
In room-as-codespace, each codespace is a node. Ternary network analysis determines: which rooms cluster into "campuses" (colocated Codespaces on same hardware), which rooms share ensigns, and how agents navigate between them. Centrality measures identify which rooms are hubs — candidates for always-on Codespaces vs. spin-up-on-demand.

## Cross-Pollination Ideas
- Small-world detection on room networks could identify when adding a single shortcut room dramatically reduces agent navigation distance
- Negative edges as "anti-patterns": rooms that should never share an agent simultaneously
- Ternary modularity as a metric for healthy fleet compartmentalization — isolation without fragmentation

## Dependencies for Next Steps
- ternary-room needs a `RoomGraph` trait that `TernaryNetwork` implements
- construct-core `Construct` needs a `neighbors()` method returning ternary-weighted connections
- ternary-protocol needs routing integration hooks
