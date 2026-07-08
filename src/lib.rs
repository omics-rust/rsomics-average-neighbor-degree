//! Average neighbor degree — value-exact port of `networkx.average_neighbor_degree`.
//!
//! For each node v: `sum(degree(w) for w in neighbors(v)) / degree(v)`.
//! Isolated nodes (degree 0) get 0.0.
//!
//! Nodes are interned to integer indices for O(m) cache-friendly computation.

use std::collections::HashMap;

/// Parsed graph: intern table + integer adjacency.
pub struct Graph {
    /// Intern table: string node → index.
    pub node_to_idx: HashMap<String, usize>,
    /// Reverse: index → original string.
    pub idx_to_node: Vec<String>,
    /// Adjacency lists (deduped, undirected).
    pub adj: Vec<Vec<usize>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            node_to_idx: HashMap::new(),
            idx_to_node: Vec::new(),
            adj: Vec::new(),
        }
    }

    fn intern(&mut self, name: &str) -> usize {
        if let Some(&idx) = self.node_to_idx.get(name) {
            return idx;
        }
        let idx = self.idx_to_node.len();
        self.node_to_idx.insert(name.to_owned(), idx);
        self.idx_to_node.push(name.to_owned());
        self.adj.push(Vec::new());
        idx
    }

    /// Add an undirected edge; parallel edges are silently ignored (matching
    /// `nx.Graph` which is a simple graph — `read_edgelist` dedupes via Graph).
    pub fn add_edge(&mut self, u: &str, v: &str) {
        let ui = self.intern(u);
        let vi = self.intern(v);
        if ui == vi {
            // self-loop: nx.Graph stores it once in adj[u][u]
            if !self.adj[ui].contains(&vi) {
                self.adj[ui].push(vi);
            }
            return;
        }
        if !self.adj[ui].contains(&vi) {
            self.adj[ui].push(vi);
            self.adj[vi].push(ui);
        }
    }

    pub fn n(&self) -> usize {
        self.idx_to_node.len()
    }

    /// Degree of node `i` matching `nx.Graph.degree`: a self-loop contributes 2.
    /// Adjacency stores a self-loop once, so add 1 back for it.
    pub fn degree(&self, i: usize) -> usize {
        self.adj[i].len() + usize::from(self.adj[i].contains(&i))
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse an edge-list from text (whitespace-delimited `u v` lines).
///
/// A `#` begins a comment anywhere in the line: `nx.parse_edgelist` truncates
/// at the first `#` before tokenising, so `1 2#note` is edge (1,2) and `0 #x`
/// is a single token (skipped). Blank/all-comment lines are skipped.
pub fn parse_edgelist(text: &str) -> Graph {
    let mut g = Graph::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let u = match parts.next() {
            Some(s) => s,
            None => continue,
        };
        let v = match parts.next() {
            Some(s) => s,
            None => continue,
        };
        g.add_edge(u, v);
    }
    g
}

/// Compute average neighbor degree for every node.
///
/// Matches `networkx.average_neighbor_degree(G)` exactly:
///   - degree(v) counts a self-loop twice (as `nx.Graph.degree` does)
///   - avg[v] = sum(degree(w) for w in adj[v]) / degree(v), where a self-loop
///     lists v as its own neighbor once
///   - isolated nodes → 0.0
///
/// Output is sorted lexicographically by original node name.
pub fn average_neighbor_degree(g: &Graph) -> Vec<(String, f64)> {
    let n = g.n();
    let deg: Vec<usize> = (0..n).map(|i| g.degree(i)).collect();

    let mut result: Vec<(String, f64)> = (0..n)
        .map(|i| {
            let d = deg[i];
            let avg = if d == 0 {
                0.0
            } else {
                // sum neighbor degrees then divide — integer/integer → exact f64
                let s: usize = g.adj[i].iter().map(|&j| deg[j]).sum();
                s as f64 / d as f64
            };
            (g.idx_to_node[i].clone(), avg)
        })
        .collect();

    // lexicographic sort matches documented output order
    result.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    result
}
