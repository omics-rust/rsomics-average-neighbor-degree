# rsomics-average-neighbor-degree

Compute the average neighbor degree for every node in an undirected graph.

For each node v: `knn(v) = sum(degree(w) for w in neighbors(v)) / degree(v)`.
Isolated nodes emit 0.0. Output is sorted lexicographically by node name.

Matches `networkx.average_neighbor_degree` value-exactly (bit-for-bit on all
tested graphs; tolerance ≤1e-13 in worst case due to IEEE-754 division).

## Install

```
cargo install rsomics-average-neighbor-degree
```

## Usage

```
# TSV output (default)
echo "0 1
1 2
2 3" | rsomics-average-neighbor-degree
# 0	2.00000000000000000e0
# 1	1.50000000000000000e0
# 2	1.50000000000000000e0
# 3	2.00000000000000000e0

# JSON output
echo "0 1
1 2" | rsomics-average-neighbor-degree --json
```

Input format: one edge per line as `u v` (whitespace-separated string node
names). Lines starting with `#` and blank lines are skipped. Parallel edges are
silently deduplicated (matching `nx.Graph` semantics).

Output: `node<TAB>value` in `{:.17e}` format, sorted lexicographically by node
name.

## Performance

Compute-only on a 5 000-node / 25 000-edge graph (aarch64-apple-darwin,
release, single thread):

| Implementation | Mean | Ratio |
|---|---|---|
| NetworkX 3.6.1 (CPython 3.12) | 6 541 µs | 1.0× |
| rsomics-average-neighbor-degree 0.1.0 | 520 µs | **12.6×** |

Benchmark measured with Criterion (100 samples) vs `time.perf_counter` (50
samples). Graph pre-loaded; compute-only timing. Machine: Apple M2 mini.

## Algorithm

Integer-indexed adjacency: string node names are interned to `0..n` at parse
time. The hot loop operates on `Vec<Vec<usize>>` and a `Vec<usize>` degree
array — no hash-map lookups during computation.

```
deg[i] = adj[i].len()
knn[i] = if deg[i] == 0 { 0.0 }
         else { (sum of deg[j] for j in adj[i]) as f64 / deg[i] as f64 }
```

O(n + m) time, O(n + m) space.

## Origin

This crate is an independent Rust reimplementation of
`networkx.average_neighbor_degree` based on:

- The NetworkX source (BSD-3-Clause):
  `networkx/algorithms/assortativity/neighbor_degree.py`
- Black-box behavior testing against NetworkX 3.6.1

NetworkX is licensed under the BSD 3-Clause License.
<https://github.com/networkx/networkx>

License: MIT OR Apache-2.0.
