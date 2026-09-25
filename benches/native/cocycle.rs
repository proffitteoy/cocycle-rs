//! Single-call native worker; fixture parsing and JSON serialization are untimed.
use std::hint::black_box;
use std::io;
use std::time::Instant;

use cocycle::diagram::{Coverage, IntervalEnd};
use cocycle::filtration::RipsBuilder;
use cocycle::geometry::DissimilarityView;
use cocycle::persistence::PersistenceExt;

fn memory(field: &str) -> String {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix(field)?
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()
            })
        })
        .map_or_else(|| "null".into(), |value| value.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        return Err(io::Error::other("usage: cocycle FIXTURE").into());
    }
    let bytes = std::fs::read(&args[0])?;
    if bytes.len() < 48 || &bytes[..8] != b"COCYCLE1" || !(bytes.len() - 48).is_multiple_of(8) {
        return Err(io::Error::other("invalid benchmark fixture").into());
    }
    let integer = |i| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    if integer(8) != 0 {
        return Err(io::Error::other("expected precomputed distances").into());
    }
    let n = usize::try_from(integer(16))?;
    let q = usize::try_from(integer(32))?;
    let cutoff = f64::from_le_bytes(bytes[40..48].try_into()?);
    let values: Vec<_> = bytes[48..]
        .chunks_exact(8)
        .map(|chunk| f64::from_le_bytes(chunk.try_into().unwrap()))
        .collect();
    drop(bytes);
    if values.iter().any(|&value| f64::from(value as f32) != value)
        || (!cutoff.is_nan() && f64::from(cutoff as f32) != cutoff)
    {
        return Err(io::Error::other("expected float32-exact filtration").into());
    }
    let input = DissimilarityView::new(&values, n)?;
    let rips = RipsBuilder::from_distance_matrix(input.into());
    let mut request = rips.persistence().max_homology_dimension(q);
    if !cutoff.is_nan() {
        request = request.max_filtration_value(cutoff);
    }
    let rss = memory("VmRSS:");
    let hwm = memory("VmHWM:");
    let start = Instant::now();
    let result = black_box(request.compute()?);
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let peak = memory("VmHWM:");
    let diagram = result.diagram();
    print!(
        "{{\"status\":\"completed\",\"elapsed_ms\":{elapsed},\"execution_path\":\"public_api\",\"rss_before_kib\":{rss},\"hwm_before_kib\":{hwm},\"peak_rss_kib\":{peak},"
    );
    match diagram.coverage() {
        Coverage::Complete => print!("\"coverage\":[\"complete\",null],"),
        Coverage::Through(t) => print!("\"coverage\":[\"through\",{t}],"),
    }
    print!("\"intervals\":[");
    for (i, bar) in diagram.intervals().iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        let (kind, end) = match bar.end() {
            IntervalEnd::Finite(d) => ("F", d),
            IntervalEnd::Essential => ("E", 0.0),
            IntervalEnd::RightCensored { through } => ("C", through),
        };
        print!("[{},{},\"{kind}\",{end}]", bar.dimension(), bar.birth());
    }
    println!("]}}");
    Ok(())
}
