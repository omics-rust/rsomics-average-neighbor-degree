use anyhow::Result;
use clap::Parser;
use rsomics_average_neighbor_degree::{average_neighbor_degree, parse_edgelist};
use std::io::{self, Read};

/// Compute average neighbor degree for every node in an undirected graph.
///
/// Reads an edge list from stdin (`u v` per line; `#` comments and blank lines
/// are skipped; string node names; parallel edges deduplicated as in
/// `networkx.read_edgelist` → `nx.Graph`).
///
/// Output: `node<TAB>value` in `{:.17e}` format, sorted lexicographically by
/// node name. Isolated nodes emit 0.0.
#[derive(Parser)]
#[command(name = "rsomics-average-neighbor-degree", version)]
struct Cli {
    /// Emit JSON object mapping node name to value instead of TSV.
    #[arg(long)]
    json: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let graph = parse_edgelist(&input);
    let result = average_neighbor_degree(&graph);

    if cli.json {
        let obj: serde_json::Map<String, serde_json::Value> = result
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::from(v)))
            .collect();
        println!("{}", serde_json::to_string_pretty(&obj)?);
    } else {
        for (node, val) in &result {
            println!("{}\t{:.17e}", node, val);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
