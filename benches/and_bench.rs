use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rsomics_average_neighbor_degree::{Graph, average_neighbor_degree};

/// Build a deterministic graph of approximately `n` nodes and `m` edges.
fn make_graph(n: usize, m: usize) -> Graph {
    let mut g = Graph::new();
    let mut state: u64 = 0xdead_beef_cafe_babe;
    let mut added = 0usize;
    while added < m {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let u = (state >> 32) as usize % n;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let v = (state >> 32) as usize % n;
        if u != v {
            g.add_edge(&u.to_string(), &v.to_string());
            added += 1;
        }
    }
    g
}

fn bench_compute(c: &mut Criterion) {
    let g = make_graph(5000, 25000);
    c.bench_function("average_neighbor_degree_5k", |b| {
        b.iter(|| average_neighbor_degree(black_box(&g)))
    });
}

criterion_group!(benches, bench_compute);
criterion_main!(benches);
