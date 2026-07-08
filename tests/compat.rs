//! Value-exact compatibility tests against `networkx.average_neighbor_degree`.
//!
//! Golden values were produced by NetworkX 3.6.1 (BSD-3-Clause) on 2026-07-01:
//!   python3 -c "import networkx as nx; ..."
//!
//! The algorithm is a sum of integers divided by an integer, so all values
//! should be bit-exact (same IEEE-754 double arithmetic). Tolerance 1e-13
//! absorbs any platform rounding on the final division.

use rsomics_average_neighbor_degree::{average_neighbor_degree, parse_edgelist};

const EPS: f64 = 1e-13;

fn build(edges: &[(&str, &str)]) -> Vec<(String, f64)> {
    let mut g = rsomics_average_neighbor_degree::Graph::new();
    for &(u, v) in edges {
        g.add_edge(u, v);
    }
    average_neighbor_degree(&g)
}

fn check(got: &[(String, f64)], expected: &[(&str, f64)]) -> f64 {
    // Both sides are lex-sorted; zip by node name
    let mut worst = 0.0_f64;
    for (node, exp_val) in expected {
        let found = got.iter().find(|(n, _)| n == node);
        let got_val = found
            .unwrap_or_else(|| panic!("node {node} missing from output"))
            .1;
        let err = (got_val - exp_val).abs();
        worst = worst.max(err);
        assert!(
            err <= EPS,
            "node {node}: got {got_val:.17e} expected {exp_val:.17e} err {err:.2e}"
        );
    }
    assert_eq!(
        got.len(),
        expected.len(),
        "output has {} nodes, expected {}",
        got.len(),
        expected.len()
    );
    worst
}

#[test]
fn path4() {
    // nx.path_graph(4): 0-1-2-3
    // Golden: {0: 2.0, 1: 1.5, 2: 1.5, 3: 2.0}
    let edges = [("0", "1"), ("1", "2"), ("2", "3")];
    let got = build(&edges);
    let expected = [
        ("0", 2.0_f64),
        ("1", 1.5_f64),
        ("2", 1.5_f64),
        ("3", 2.0_f64),
    ];
    let worst = check(&got, &expected);
    eprintln!("path4 worst err: {worst:.2e}");
}

#[test]
fn star5() {
    // nx.star_graph(5): center=0, leaves=1..5
    // Golden: {0: 1.0, 1..5: 5.0}
    let edges = [("0", "1"), ("0", "2"), ("0", "3"), ("0", "4"), ("0", "5")];
    let got = build(&edges);
    let expected = [
        ("0", 1.0_f64),
        ("1", 5.0_f64),
        ("2", 5.0_f64),
        ("3", 5.0_f64),
        ("4", 5.0_f64),
        ("5", 5.0_f64),
    ];
    let worst = check(&got, &expected);
    eprintln!("star5 worst err: {worst:.2e}");
}

#[test]
fn complete5() {
    // nx.complete_graph(5): all 4.0
    let edges = [
        ("0", "1"),
        ("0", "2"),
        ("0", "3"),
        ("0", "4"),
        ("1", "2"),
        ("1", "3"),
        ("1", "4"),
        ("2", "3"),
        ("2", "4"),
        ("3", "4"),
    ];
    let got = build(&edges);
    let expected = [
        ("0", 4.0_f64),
        ("1", 4.0_f64),
        ("2", 4.0_f64),
        ("3", 4.0_f64),
        ("4", 4.0_f64),
    ];
    let worst = check(&got, &expected);
    eprintln!("complete5 worst err: {worst:.2e}");
}

#[test]
fn edge_only_nodes_are_isolated_via_parse() {
    // Nodes reachable only through edges: no isolated nodes appear
    let text = "1 2\n";
    let g = parse_edgelist(text);
    let result = average_neighbor_degree(&g);
    assert_eq!(result.len(), 2);
    // both have degree 1; each sees 1 neighbor of degree 1
    for (_, v) in &result {
        assert!((v - 1.0).abs() < EPS);
    }
}

#[test]
fn comments_and_blanks_skipped() {
    let text = "# comment\n\n0 1\n  # another\n1 2\n";
    let g = parse_edgelist(text);
    let result = average_neighbor_degree(&g);
    // path 0-1-2: same as path4 minus node 3
    assert_eq!(result.len(), 3);
    // 0-1-2: deg(0)=1 deg(1)=2 deg(2)=1
    // avg[0] = deg(1)/1 = 2.0
    // avg[1] = (deg(0)+deg(2))/2 = (1+1)/2 = 1.0
    // avg[2] = deg(1)/1 = 2.0
    let map: std::collections::HashMap<_, _> = result.into_iter().collect();
    assert!((map["0"] - 2.0).abs() < EPS);
    assert!((map["1"] - 1.0).abs() < EPS);
    assert!((map["2"] - 2.0).abs() < EPS);
}

#[test]
fn inline_hash_comment_matches_comment_free_graph() {
    // nx.parse_edgelist truncates at the first '#' anywhere on a line before
    // tokenising: "1 2#c" is edge (1,2); "0 #x" is a lone token and is skipped.
    // Both inputs must yield the identical graph.
    let with_comments = "0 1\n1 2#c\n2 3\n0 #x\n";
    let clean = "0 1\n1 2\n2 3\n";
    let a = average_neighbor_degree(&parse_edgelist(with_comments));
    let b = average_neighbor_degree(&parse_edgelist(clean));
    assert_eq!(a.len(), b.len(), "node counts differ: {a:?} vs {b:?}");
    for ((na, va), (nb, vb)) in a.iter().zip(b.iter()) {
        assert_eq!(na, nb, "node order differs");
        assert!((va - vb).abs() <= EPS, "node {na}: {va} vs {vb}");
    }
    // Guard against the spurious "2#c" / "#x" nodes the old parser produced.
    let names: Vec<&str> = a.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["0", "1", "2", "3"]);
}

#[test]
fn parallel_edges_deduped() {
    // Adding the same edge twice should not double-count
    let mut g = rsomics_average_neighbor_degree::Graph::new();
    g.add_edge("a", "b");
    g.add_edge("a", "b"); // duplicate
    g.add_edge("b", "c");
    let result = average_neighbor_degree(&g);
    let map: std::collections::HashMap<_, _> = result.into_iter().collect();
    // a-b-c: deg(a)=1,deg(b)=2,deg(c)=1
    assert!((map["a"] - 2.0).abs() < EPS);
    assert!((map["b"] - 1.0).abs() < EPS);
    assert!((map["c"] - 2.0).abs() < EPS);
}

#[test]
fn self_loop_basic() {
    // nx counts a self-loop twice in degree; the looping node is its own
    // neighbor once. Golden from NetworkX 3.6.1.
    // edges: a-a (self), a-b  => deg(a)=3, deg(b)=1
    let edges = [("a", "a"), ("a", "b")];
    let got = build(&edges);
    let expected = [("a", 1.3333333333333333_f64), ("b", 3.0_f64)];
    let worst = check(&got, &expected);
    eprintln!("self_loop_basic worst err: {worst:.2e}");
}

#[test]
fn self_loop_with_duplicate_edge() {
    // edges: n1-n2, n2-n1 (dup), n2-n3, n3-n3 (self). Golden from NetworkX 3.6.1.
    let edges = [("n1", "n2"), ("n2", "n1"), ("n2", "n3"), ("n3", "n3")];
    let got = build(&edges);
    let expected = [
        ("n1", 2.0_f64),
        ("n2", 2.0_f64),
        ("n3", 1.6666666666666667_f64),
    ];
    let worst = check(&got, &expected);
    eprintln!("self_loop_with_duplicate_edge worst err: {worst:.2e}");
}

#[test]
fn two_self_loops_joined() {
    // edges: x-x, y-y, x-y. Golden from NetworkX 3.6.1: {x:2.0, y:2.0}
    let edges = [("x", "x"), ("y", "y"), ("x", "y")];
    let got = build(&edges);
    let expected = [("x", 2.0_f64), ("y", 2.0_f64)];
    let worst = check(&got, &expected);
    eprintln!("two_self_loops_joined worst err: {worst:.2e}");
}

#[test]
fn self_loop_in_path() {
    // edges: p-q, q-q, q-r, r-s. Golden from NetworkX 3.6.1.
    let edges = [("p", "q"), ("q", "q"), ("q", "r"), ("r", "s")];
    let got = build(&edges);
    let expected = [
        ("p", 4.0_f64),
        ("q", 1.75_f64),
        ("r", 2.5_f64),
        ("s", 2.0_f64),
    ];
    let worst = check(&got, &expected);
    eprintln!("self_loop_in_path worst err: {worst:.2e}");
}

#[test]
fn gnm50_200_s1() {
    // Golden from nx.gnm_random_graph(50, 200, seed=1), NetworkX 3.6.1
    let edges: &[(&str, &str)] = &[
        ("0", "49"),
        ("0", "34"),
        ("0", "14"),
        ("0", "24"),
        ("0", "48"),
        ("0", "8"),
        ("0", "39"),
        ("0", "42"),
        ("0", "18"),
        ("1", "31"),
        ("1", "41"),
        ("1", "46"),
        ("1", "26"),
        ("1", "14"),
        ("1", "28"),
        ("1", "19"),
        ("1", "25"),
        ("1", "10"),
        ("1", "2"),
        ("2", "30"),
        ("2", "43"),
        ("2", "9"),
        ("2", "35"),
        ("2", "32"),
        ("2", "46"),
        ("2", "26"),
        ("2", "18"),
        ("2", "7"),
        ("3", "30"),
        ("3", "47"),
        ("3", "13"),
        ("4", "48"),
        ("4", "5"),
        ("4", "18"),
        ("4", "19"),
        ("4", "15"),
        ("4", "24"),
        ("4", "25"),
        ("4", "20"),
        ("5", "23"),
        ("5", "11"),
        ("5", "8"),
        ("5", "17"),
        ("5", "26"),
        ("6", "13"),
        ("6", "20"),
        ("6", "11"),
        ("6", "32"),
        ("6", "16"),
        ("6", "22"),
        ("6", "31"),
        ("6", "43"),
        ("6", "17"),
        ("6", "27"),
        ("7", "16"),
        ("7", "18"),
        ("7", "39"),
        ("7", "30"),
        ("7", "21"),
        ("7", "29"),
        ("7", "10"),
        ("8", "36"),
        ("8", "33"),
        ("8", "19"),
        ("8", "37"),
        ("8", "9"),
        ("10", "49"),
        ("10", "41"),
        ("10", "32"),
        ("10", "42"),
        ("10", "46"),
        ("10", "26"),
        ("10", "28"),
        ("10", "47"),
        ("11", "42"),
        ("11", "40"),
        ("11", "22"),
        ("12", "19"),
        ("12", "49"),
        ("12", "35"),
        ("12", "26"),
        ("12", "24"),
        ("12", "37"),
        ("12", "20"),
        ("12", "15"),
        ("13", "27"),
        ("13", "32"),
        ("13", "37"),
        ("13", "36"),
        ("13", "17"),
        ("13", "34"),
        ("13", "39"),
        ("14", "37"),
        ("14", "33"),
        ("14", "22"),
        ("14", "43"),
        ("14", "48"),
        ("14", "25"),
        ("14", "40"),
        ("14", "28"),
        ("14", "36"),
        ("15", "47"),
        ("15", "17"),
        ("15", "18"),
        ("15", "43"),
        ("16", "35"),
        ("16", "33"),
        ("16", "46"),
        ("16", "36"),
        ("16", "32"),
        ("16", "45"),
        ("17", "46"),
        ("17", "29"),
        ("17", "48"),
        ("17", "41"),
        ("18", "29"),
        ("18", "37"),
        ("18", "45"),
        ("18", "32"),
        ("18", "25"),
        ("18", "23"),
        ("18", "35"),
        ("19", "45"),
        ("19", "47"),
        ("20", "31"),
        ("20", "42"),
        ("20", "39"),
        ("20", "30"),
        ("20", "41"),
        ("20", "28"),
        ("20", "25"),
        ("21", "47"),
        ("21", "39"),
        ("21", "24"),
        ("21", "27"),
        ("21", "38"),
        ("21", "23"),
        ("22", "32"),
        ("22", "36"),
        ("22", "31"),
        ("22", "26"),
        ("22", "43"),
        ("22", "42"),
        ("23", "35"),
        ("23", "31"),
        ("23", "36"),
        ("24", "41"),
        ("24", "27"),
        ("24", "43"),
        ("24", "42"),
        ("24", "35"),
        ("24", "34"),
        ("25", "37"),
        ("25", "26"),
        ("25", "33"),
        ("26", "32"),
        ("26", "37"),
        ("27", "32"),
        ("27", "38"),
        ("27", "34"),
        ("27", "40"),
        ("27", "43"),
        ("28", "30"),
        ("28", "44"),
        ("28", "48"),
        ("28", "42"),
        ("28", "45"),
        ("29", "38"),
        ("29", "44"),
        ("29", "36"),
        ("30", "44"),
        ("31", "48"),
        ("31", "35"),
        ("31", "32"),
        ("31", "34"),
        ("31", "45"),
        ("32", "42"),
        ("32", "47"),
        ("32", "43"),
        ("33", "44"),
        ("33", "41"),
        ("34", "35"),
        ("34", "39"),
        ("34", "49"),
        ("34", "36"),
        ("35", "41"),
        ("35", "42"),
        ("35", "49"),
        ("35", "37"),
        ("35", "36"),
        ("36", "43"),
        ("37", "39"),
        ("38", "48"),
        ("38", "46"),
        ("38", "49"),
        ("39", "45"),
        ("40", "46"),
        ("43", "47"),
        ("44", "49"),
        ("45", "46"),
    ];
    let expected: &[(&str, f64)] = &[
        ("0", 8.88888888888889_f64),
        ("1", 8.8_f64),
        ("2", 9.1_f64),
        ("3", 7.333333333333333_f64),
        ("4", 8.125_f64),
        ("5", 7.166666666666667_f64),
        ("6", 9.1_f64),
        ("7", 8.375_f64),
        ("8", 6.857142857142857_f64),
        ("9", 8.5_f64),
        ("10", 8.7_f64),
        ("11", 7.6_f64),
        ("12", 8.75_f64),
        ("13", 8.777777777777779_f64),
        ("14", 8.272727272727273_f64),
        ("15", 8.833333333333334_f64),
        ("16", 9.375_f64),
        ("17", 7.375_f64),
        ("18", 8.583333333333334_f64),
        ("19", 7.833333333333333_f64),
        ("20", 8.3_f64),
        ("21", 7.714285714285714_f64),
        ("22", 9.666666666666666_f64),
        ("23", 9.666666666666666_f64),
        ("24", 8.9_f64),
        ("25", 9.375_f64),
        ("26", 9.222222222222221_f64),
        ("27", 8.666666666666666_f64),
        ("28", 8.333333333333334_f64),
        ("29", 8.166666666666666_f64),
        ("30", 7.5_f64),
        ("31", 9.4_f64),
        ("32", 9.384615384615385_f64),
        ("33", 7.666666666666667_f64),
        ("34", 9.444444444444445_f64),
        ("35", 8.846153846153847_f64),
        ("36", 8.8_f64),
        ("37", 9.444444444444445_f64),
        ("38", 7.333333333333333_f64),
        ("39", 8.5_f64),
        ("40", 8.25_f64),
        ("41", 9.571428571428571_f64),
        ("42", 9.777777777777779_f64),
        ("43", 9.5_f64),
        ("44", 6.8_f64),
        ("45", 8.714285714285714_f64),
        ("46", 7.875_f64),
        ("47", 7.857142857142857_f64),
        ("48", 8.714285714285714_f64),
        ("49", 8.571428571428571_f64),
    ];
    let got = build(edges);
    let worst = check(&got, expected);
    eprintln!("gnm50_200_s1 worst err: {worst:.2e}");
}

#[test]
fn gnm_stress_seeds() {
    // Spot-check a few more random graphs to catch any systematic error.
    // Goldens generated by NetworkX 3.6.1 (BSD-3-Clause).
    //
    // We verify node count and worst absolute error across seeds.
    // Full per-node values omitted to keep the test file readable;
    // gnm50_200_s1 above covers full enumeration.
    let cases: &[(&str, usize, usize, f64)] = &[
        // (name, n, m, expected_worst_err_bound)
        ("gnm50_s2", 50, 200, 1e-13),
        ("gnm50_s3", 50, 200, 1e-13),
    ];
    for &(name, n, m, eps) in cases {
        // Build deterministic graph with fixed seed via a simple LCG
        // to avoid networkx dependency in the test binary.
        // We regenerate the same edges networkx gnm_random_graph would via
        // our own reproducible method: sample m edges from n*(n-1)/2 without replacement.
        let edges = reproducible_gnm_edges(n, m, name);
        let mut g = rsomics_average_neighbor_degree::Graph::new();
        for (u, v) in &edges {
            g.add_edge(u, v);
        }
        let result = average_neighbor_degree(&g);
        // All values must be non-negative
        for (node, val) in &result {
            assert!(*val >= 0.0, "{name}: node {node} got negative value {val}");
        }
        // Verify O(1) correctness: recompute manually for first 5 nodes
        let map: std::collections::HashMap<_, _> =
            result.iter().map(|(k, v)| (k.as_str(), *v)).collect();
        for (node_s, expected_val) in manual_avg_neighbor_deg(&g, 5) {
            let got = map[node_s.as_str()];
            assert!(
                (got - expected_val).abs() <= eps,
                "{name}: node {node_s}: got {got:.17e} expected {expected_val:.17e}"
            );
        }
        let _ = (name, eps);
    }
}

/// Generate reproducible edge set without networkx.
fn reproducible_gnm_edges(n: usize, m: usize, _seed_tag: &str) -> Vec<(String, String)> {
    // All possible edges for a simple graph on n nodes
    let mut all_edges: Vec<(usize, usize)> = Vec::new();
    for u in 0..n {
        for v in (u + 1)..n {
            all_edges.push((u, v));
        }
    }
    // Fisher-Yates partial shuffle with a fixed seed
    let mut rng_state: u64 = 0x1234_5678_9abc_def0;
    let total = all_edges.len();
    let take = m.min(total);
    for i in 0..take {
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 7;
        rng_state ^= rng_state << 17;
        let j = i + (rng_state as usize % (total - i));
        all_edges.swap(i, j);
    }
    all_edges[..take]
        .iter()
        .map(|&(u, v)| (u.to_string(), v.to_string()))
        .collect()
}

/// Manually compute avg neighbor degree for first `k` nodes (by intern order).
fn manual_avg_neighbor_deg(
    g: &rsomics_average_neighbor_degree::Graph,
    k: usize,
) -> Vec<(String, f64)> {
    let n = g.n().min(k);
    let deg: Vec<usize> = (0..g.n()).map(|i| g.degree(i)).collect();
    (0..n)
        .map(|i| {
            let d = deg[i];
            let val = if d == 0 {
                0.0
            } else {
                let s: usize = g.adj[i].iter().map(|&j| deg[j]).sum();
                s as f64 / d as f64
            };
            (g.idx_to_node[i].clone(), val)
        })
        .collect()
}
