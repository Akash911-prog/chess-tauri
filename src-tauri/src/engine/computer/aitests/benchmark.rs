use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::engine::computer::aitests::setup_board;
use crate::engine::computer::negamax::Search;

#[derive(Serialize, Deserialize, Debug, Default)]
struct BenchmarkResult {
    nodes_visited: u64,
    elapsed_ms: f64,
    nodes_per_sec: f64,
}

#[test]
fn timed_negamax() {
    let mut board =
        setup_board("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let mut search = Search::new(&mut board);

    // 1. Load previous benchmark if file exists
    let file_path = "negamax_benchmark.json";
    let prev_result: Option<BenchmarkResult> = if Path::new(file_path).exists() {
        fs::read_to_string(file_path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
    } else {
        None
    };

    let time_limit = Duration::from_millis(400000);

    // 2. Run current benchmark
    let start = Instant::now();
    let s1 = search.negamax(7, 0, i32::MIN + 1, i32::MAX, &start, &time_limit);
    let elapsed = start.elapsed();

    // 3. Calculate current metrics
    let elapsed_secs = elapsed.as_secs_f64();
    let elapsed_ms = elapsed_secs * 1000.0;
    let nodes = search.nodes_visited as u64;
    let nps = if elapsed_secs > 0.0 {
        nodes as f64 / elapsed_secs
    } else {
        0.0
    };

    let current_result = BenchmarkResult {
        nodes_visited: nodes,
        elapsed_ms,
        nodes_per_sec: nps,
    };

    // 4. Save current run to file
    if let Ok(json) = serde_json::to_string_pretty(&current_result) {
        let _ = fs::write(file_path, json);
    }

    // 5. Print results and comparison
    println!("\n--- Search Results ---");
    println!("Negamax score: {}", s1);
    println!("Nodes visited: {}", nodes);
    println!("Elapsed time : {:.3} ms", elapsed_ms);
    println!("NPS          : {:.0} nodes/s", nps);

    println!("\n--- Comparison with Previous Run ---");
    if let Some(prev) = prev_result {
        let node_diff = nodes as i64 - prev.nodes_visited as i64;
        let time_diff = elapsed_ms - prev.elapsed_ms;
        let nps_diff = nps - prev.nodes_per_sec;

        println!("Nodes diff   : {:+} nodes", node_diff);
        println!("Time diff    : {:+.3} ms", time_diff);
        println!(
            "NPS diff     : {:+.0} nodes/s ({:+.2}%)",
            nps_diff,
            (nps_diff / prev.nodes_per_sec) * 100.0
        );
    } else {
        println!("No previous benchmark file found. Saved current run as baseline.");
    }
}
