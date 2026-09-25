//! Standalone distance worker; private controls never become public crate options.
#![forbid(unsafe_code)]
#![allow(dead_code)]

#[cfg(not(cocycle_distance_bench))]
compile_error!("the distance worker requires --cfg cocycle_distance_bench");

pub use cocycle::{Error, Result, diagram};
#[path = "../../src/diagram_distances/mod.rs"]
mod diagram_distances;

use diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use diagram_distances::{bottleneck, wasserstein};
use std::{env, fs, time::Instant};

fn memory(field: &str) -> Option<usize> {
    fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|line| {
            line.strip_prefix(field)?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
}

fn nullable(value: Option<usize>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn diagram(points: &[[f64; 2]]) -> Result<PersistenceDiagram> {
    let mut intervals = Vec::new();
    for &[birth, death] in points {
        if birth == death && birth.is_finite() {
            continue; // Explicit raw-to-typed diagonal projection, inside timing.
        }
        intervals.push(PersistenceInterval::new(
            0,
            birth,
            if death == f64::INFINITY {
                IntervalEnd::Essential
            } else {
                IntervalEnd::Finite(death)
            },
        )?);
    }
    PersistenceDiagram::new(0, Coverage::Complete, intervals)
}

fn finite(points: &[[f64; 2]]) -> Result<Vec<[f64; 2]>> {
    let mut output = Vec::new();
    for &[birth, death] in points {
        if !birth.is_finite() || !death.is_finite() || death < birth {
            return Err(Error::InvalidInterval {
                reason: "finite ablation worker requires finite ordered endpoints",
            });
        }
        if birth < death {
            output.push([birth, death]);
        }
    }
    Ok(output)
}

fn compute(
    first: &[[f64; 2]],
    second: &[[f64; 2]],
    metric: &str,
    variant: &str,
) -> Result<(f64, bottleneck::Diagnostics, wasserstein::Stats)> {
    if variant == "public" {
        let first = diagram(first)?;
        let second = diagram(second)?;
        let value = match metric {
            "bottleneck" => cocycle::diagram_distances::bottleneck_distance(&first, &second, 0)?,
            "w1" => cocycle::diagram_distances::wasserstein_1_infinity(&first, &second, 0)?,
            "w2" => cocycle::diagram_distances::wasserstein_2_euclidean(&first, &second, 0)?,
            _ => {
                return Err(Error::InvalidInterval {
                    reason: "unknown metric",
                });
            }
        };
        return Ok((
            value,
            bottleneck::Diagnostics::default(),
            wasserstein::Stats::default(),
        ));
    }
    // Every native Rust variant has the same raw validation/copying boundary.
    let first = finite(first)?;
    let second = finite(second)?;
    match metric {
        "bottleneck" => {
            let mut options = bottleneck::Options::default();
            match variant {
                "baseline" => {}
                "binary" => options.search = bottleneck::Search::Binary,
                "refinement" => options.search = bottleneck::Search::Refinement,
                "quickselect" => options.search = bottleneck::Search::Quickselect,
                "quickselect_no_clip" => {
                    options.search = bottleneck::Search::Quickselect;
                    options.clip_candidates = false;
                }
                "refinement_no_matching_reuse" => {
                    options.search = bottleneck::Search::Refinement;
                    options.reuse_matching = false;
                }
                "refinement_no_scratch_reuse" => {
                    options.search = bottleneck::Search::Refinement;
                    options.reuse_scratch = false;
                }
                "no_clip" => options.clip_candidates = false,
                "no_matching_reuse" => options.reuse_matching = false,
                "no_scratch_reuse" => options.reuse_scratch = false,
                _ => {
                    return Err(Error::InvalidInterval {
                        reason: "unknown bottleneck variant",
                    });
                }
            }
            let mut stats = bottleneck::Diagnostics::default();
            let value = bottleneck::distance_with_options(&first, &second, options, &mut stats)?;
            Ok((value, stats, wasserstein::Stats::default()))
        }
        "w1" | "w2" => {
            let mut options = wasserstein::Options::default();
            match variant {
                "baseline" => {}
                "vectors" => options.force_sparse = true,
                "adaptive_arena" => options.sparse = wasserstein::SparseLayout::Arena,
                "arena" => {
                    options.force_sparse = true;
                    options.sparse = wasserstein::SparseLayout::Arena;
                }
                _ => {
                    return Err(Error::InvalidInterval {
                        reason: "unknown Wasserstein variant",
                    });
                }
            }
            let mut stats = wasserstein::Stats::default();
            let value = wasserstein::distance_with_options(
                &first,
                &second,
                if metric == "w1" {
                    wasserstein::Metric::W1
                } else {
                    wasserstein::Metric::W2
                },
                options,
                &mut stats,
            )?;
            Ok((value, bottleneck::Diagnostics::default(), stats))
        }
        _ => Err(Error::InvalidInterval {
            reason: "unknown metric",
        }),
    }
}

fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        return Err("usage: cocycle fixture metric variant".into());
    }
    let bytes = fs::read(&args[1])?;
    if bytes.len() < 24 || &bytes[..8] != b"COCDST1\0" {
        return Err("expected COCDST1 distance fixture".into());
    }
    let n = usize::try_from(u64::from_le_bytes(bytes[8..16].try_into()?))?;
    let m = usize::try_from(u64::from_le_bytes(bytes[16..24].try_into()?))?;
    if n.checked_add(m)
        .and_then(|count| count.checked_mul(16))
        .and_then(|length| length.checked_add(24))
        != Some(bytes.len())
    {
        return Err("invalid fixture length".into());
    }
    let points: Vec<[f64; 2]> = bytes[24..]
        .chunks_exact(16)
        .map(|chunk| {
            [
                f64::from_le_bytes(chunk[..8].try_into().expect("eight bytes")),
                f64::from_le_bytes(chunk[8..].try_into().expect("eight bytes")),
            ]
        })
        .collect();
    // Drop parser bytes before the shared prepared-input memory observation.
    drop(bytes);
    let rss = memory("VmRSS:");
    let hwm = memory("VmHWM:");
    let start = Instant::now();
    let (value, bs, ws) = compute(&points[..n], &points[n..], &args[2], &args[3])?;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let peak = memory("VmHWM:");
    if value.is_nan() || value < 0.0 {
        return Err("invalid solver distance".into());
    }
    let scalar = if value.is_infinite() {
        "null".to_owned()
    } else {
        format!("{value:.17e}")
    };
    let stats = format!(
        concat!(
            "{{\"route\":\"{:?}\",\"threshold_decisions\":{},",
            "\"candidate_count\":{},\"adjacency_checks\":{},\"augment_searches\":{},",
            "\"kd_nodes_visited\":{},\"matching_reuses\":{},\"scratch_reuses\":{},",
            "\"capacity_edges\":{},\"peak_workspace_bytes\":{},\"candidate_pairs\":{},",
            "\"positive_edges\":{},\"dense_solves\":{},\"sparse_solves\":{},\"augmentations\":{},",
            "\"components\":{},\"tiny_components\":{},\"duplicate_groups\":{},\"greedy_certificates\":{},",
            "\"direct_cost_fallbacks\":{},\"peak_graph_storage_bytes\":{},",
            "\"peak_residual_storage_bytes\":{},\"peak_sparse_scratch_bytes\":{}}}"
        ),
        bs.route,
        bs.threshold_decisions,
        bs.candidate_count,
        bs.adjacency_checks,
        bs.augment_searches,
        bs.kd_nodes_visited,
        bs.matching_reuses,
        bs.scratch_reuses + ws.scratch_reuses,
        bs.capacity_edges,
        bs.peak_workspace_bytes,
        ws.candidate_pairs,
        ws.positive_edges,
        ws.dense_solves,
        ws.sparse_solves,
        ws.augmentations,
        ws.components,
        ws.tiny_components,
        ws.duplicate_groups,
        ws.greedy_certificates,
        ws.direct_cost_fallbacks,
        ws.peak_graph_storage_bytes,
        ws.peak_residual_storage_bytes,
        ws.peak_sparse_scratch_bytes
    );
    println!(
        concat!(
            "{{\"protocol_id\":\"cocycle-distance-v1\",\"status\":\"completed\",",
            "\"backend\":\"cocycle\",\"metric\":\"{}\",\"variant\":\"{}\",",
            "\"value_kind\":\"{}\",\"value\":{},\"elapsed_ms\":{:.17e},",
            "\"rss_before_kib\":{},\"hwm_before_kib\":{},\"peak_rss_kib\":{},\"stats\":{}}}"
        ),
        args[2],
        args[3],
        if value.is_infinite() {
            "infinite"
        } else {
            "finite"
        },
        scalar,
        elapsed,
        nullable(rss),
        nullable(hwm),
        nullable(peak),
        stats
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
