# Track D: Traverse

The task, the input files and the exact output format are in the [spec](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/d-traverse.md).

## Approach

Rust implementation using Dijkstra's algorithm with a binary heap (priority queue). Loads the entire graph into memory as an adjacency list. For each query, runs a fresh Dijkstra search from the source node, stopping as soon as the target is settled. Uses standard library HashMap for distance tracking and saturating arithmetic to handle large weights safely.

## Dependencies

None (uses only Rust standard library)

## With another 24 hours

Would explore bidirectional Dijkstra or A* with heuristics for faster query processing. Could also consider preprocessing with landmark-based routing.
