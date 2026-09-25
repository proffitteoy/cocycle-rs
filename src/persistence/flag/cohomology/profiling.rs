//! Explicit developer profiling; excluded from normal test runs.

use super::*;
use crate::diagram::{Coverage, IntervalEnd};
use crate::persistence::rips::resolve_rips_range;
use crate::persistence::{RipsOptions, assemble_diagram, reference};

/// One fresh test process per stage/fixture; tools/profile_rips.py orchestrates.
#[test]
#[ignore = "explicit development profiling, no timing assertions"]
fn profile_stage() {
    use std::hint::black_box;
    use std::time::Instant;
    let bytes = std::fs::read(std::env::var("COCYCLE_ABLATION_FIXTURE").unwrap()).unwrap();
    assert!(bytes.len() >= 48 && &bytes[..8] == b"COCYCLE1");
    let integer = |i| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap()) as usize;
    assert_eq!(integer(8), 0, "distance fixtures only");
    assert_eq!(integer(32), 1, "H1 fixtures only");
    let cutoff = f64::from_le_bytes(bytes[40..48].try_into().unwrap());
    let values: Vec<_> = bytes[48..]
        .as_chunks::<8>()
        .0
        .iter()
        .map(|x| f64::from_le_bytes(*x))
        .collect();
    let input = DissimilarityView::new(&values, integer(16)).unwrap();
    let options = RipsOptions::new(1, (!cutoff.is_nan()).then_some(cutoff)).unwrap();
    let (cutoff, coverage) = resolve_rips_range(input, &options);
    let stage = std::env::var("COCYCLE_ABLATION_STAGE").unwrap();
    let execute = |stats: &mut Stats| {
        stats.two_pass_initialization = matches!(stage.as_str(), "two-pass" | "virtual-two-pass");
        match stage.as_str() {
            "explicit" => run::<false, false, false, NO_SHORTCUTS>(input, cutoff, stats),
            "clearing" => run::<false, true, false, NO_SHORTCUTS>(input, cutoff, stats),
            "implicit" => run::<true, true, false, NO_SHORTCUTS>(input, cutoff, stats),
            "cone" => run::<true, true, true, NO_SHORTCUTS>(input, cutoff, stats),
            "apparent" => run::<true, true, true, APPARENT>(input, cutoff, stats),
            "emergent" | "two-pass" => {
                run::<true, true, true, APPARENT_EMERGENT>(input, cutoff, stats)
            }
            "virtual" | "virtual-two-pass" => {
                run::<true, true, true, ALL_SHORTCUTS>(input, cutoff, stats)
            }
            _ => panic!("unknown stage"),
        }
    };
    let rss = |field: &str| -> Option<usize> {
        std::fs::read_to_string("/proc/self/status")
            .ok()?
            .lines()
            .find_map(|line| {
                line.strip_prefix(field)?
                    .split_whitespace()
                    .next()?
                    .parse()
                    .ok()
            })
    };
    let before = rss("VmHWM:");
    let mut stats = Stats::default();
    let warmup = execute(&mut stats).unwrap();
    let mut samples = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        drop(black_box(execute(&mut Stats::default()).unwrap()));
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let peak = rss("VmHWM:");
    // Reference execution is after measurement so it cannot pollute stage RSS.
    assert_eq!(
        assemble_diagram(1, coverage, warmup).unwrap(),
        reference::compute(input, &options).unwrap()
    );
    let json_number = |x: Option<usize>| x.map_or_else(|| "null".into(), |x| x.to_string());
    println!(
        "ABLATION {{\"stage\":\"{stage}\",\"samples_ms\":{samples:?},\"hwm_before_kib\":{},\"peak_rss_kib\":{},\"cofacets\":{},\"initial_candidates\":{},\"reconstruction_candidates\":{},\"apparent_candidates\":{},\"skipped_apparent\":{},\"virtual_additions\":{},\"stored_columns\":{},\"column_additions\":{},\"shortcuts\":{},\"peak_heap_entries\":{},\"stored_entries\":{}}}",
        json_number(before),
        json_number(peak),
        stats.cofacets,
        stats.initial_candidates,
        stats.reconstruction_candidates,
        stats.apparent_candidates,
        stats.skipped_apparent,
        stats.virtual_additions,
        stats.stored_columns,
        stats.column_additions,
        stats.shortcuts,
        stats.peak_heap,
        stats.stored_entries
    );
}

/// Large-input counters without running the cubic reference algorithm. The
/// Python controller MUST check the emitted diagram against a validated public
/// benchmark result before accepting these diagnostic counters.
#[test]
#[ignore = "development workload counters; validated by tools/profile_scaling.py"]
fn profile_workload() {
    let bytes = std::fs::read(std::env::var("COCYCLE_ABLATION_FIXTURE").unwrap()).unwrap();
    assert!(bytes.len() >= 48 && &bytes[..8] == b"COCYCLE1");
    let integer = |i| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap()) as usize;
    assert_eq!(integer(8), 0);
    assert_eq!(integer(32), 1);
    let cutoff = f64::from_le_bytes(bytes[40..48].try_into().unwrap());
    let values: Vec<_> = bytes[48..]
        .as_chunks::<8>()
        .0
        .iter()
        .map(|x| f64::from_le_bytes(*x))
        .collect();
    let input = DissimilarityView::new(&values, integer(16)).unwrap();
    let options = RipsOptions::new(1, (!cutoff.is_nan()).then_some(cutoff)).unwrap();
    let (cutoff, coverage) = resolve_rips_range(input, &options);
    let mut stats = Stats::default();
    let raw = run::<true, true, true, PRODUCTION_SHORTCUTS>(input, cutoff, &mut stats).unwrap();
    let diagram = assemble_diagram(1, coverage, raw).unwrap();
    print!(
        "WORKLOAD {{\"statistics\":{{\"edges\":{},\"cofacets\":{},\"initial_candidates\":{},\"reconstruction_candidates\":{},\"apparent_candidates\":{},\"skipped_apparent\":{},\"virtual_additions\":{},\"stored_columns\":{},\"column_additions\":{},\"shortcuts\":{},\"peak_coboundary_heap\":{},\"stored_transform_entries\":{},\"largest_transform\":{},\"peak_transform_heap\":{}}},",
        stats.edges,
        stats.cofacets,
        stats.initial_candidates,
        stats.reconstruction_candidates,
        stats.apparent_candidates,
        stats.skipped_apparent,
        stats.virtual_additions,
        stats.stored_columns,
        stats.column_additions,
        stats.shortcuts,
        stats.peak_heap,
        stats.stored_entries,
        stats.largest_transform,
        stats.peak_transform_heap
    );
    match coverage {
        Coverage::Complete => print!("\"coverage\":[\"complete\",null],"),
        Coverage::Through(t) => print!("\"coverage\":[\"through\",{t}],"),
    }
    print!("\"intervals\":[");
    for (i, bar) in diagram.intervals().iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        let (kind, endpoint) = match bar.end() {
            IntervalEnd::Finite(d) => ("F", d),
            IntervalEnd::Essential => ("E", 0.0),
            IntervalEnd::RightCensored { through } => ("C", through),
        };
        print!(
            "[{},{},\"{kind}\",{endpoint}]",
            bar.dimension(),
            bar.birth()
        );
    }
    println!("]}}");
}
