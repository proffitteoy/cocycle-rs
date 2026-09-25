//! One public Rips workflow per fresh process; timings follow public API boundaries.
use cocycle::algebra::PrimeField;
use cocycle::complex::{WeightedEdge, WeightedGraph};
use cocycle::diagram::{Coverage, IntervalEnd, PersistenceResult};
use cocycle::execution::Execution;
use cocycle::filtration::{ApproximateRipsBuilder, FlagFiltration, RipsBuilder};
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout, MetricPolicy, PointCloudView};
use cocycle::persistence::{
    PersistenceExt, PersistenceOptions, RepresentativeRequest, RepresentativeSelection,
};
use std::fmt::Write;
use std::time::Instant;

fn memory(field: &str) -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap()
        .lines()
        .find_map(|line| {
            line.strip_prefix(field)?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
        .unwrap()
}
fn timed<T>(out: &mut f64, action: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = action();
    *out = start.elapsed().as_secs_f64() * 1000.;
    result
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 {
        return Err("usage: pipeline FIXTURE PATH LAYOUT EPSILON REPRESENTATIVES".into());
    }
    let source = std::fs::read_to_string(&args[1])?;
    let mut words = source.split_whitespace();
    let mode = words.next().ok_or("mode")?.to_owned();
    let n: usize = words.next().ok_or("n")?.parse()?;
    let q: usize = words.next().ok_or("q")?.parse()?;
    let cutoff = words.next().ok_or("cutoff")?;
    let cutoff = if cutoff == "none" {
        None
    } else {
        Some(cutoff.parse()?)
    };
    let count: usize = words.next().ok_or("count")?.parse()?;
    let p: u32 = words.next().ok_or("field")?.parse()?;
    let mut values = Vec::new();
    let mut edges = Vec::new();
    if mode == "dense" {
        values = words.map(str::parse).collect::<Result<_, _>>()?;
        if values.len() != count {
            return Err("distance count".into());
        }
    } else if mode == "flag" {
        for _ in 0..count {
            edges.push(WeightedEdge {
                vertices: [
                    words.next().ok_or("a")?.parse()?,
                    words.next().ok_or("b")?.parse()?,
                ],
                value: words.next().ok_or("value")?.parse()?,
            });
        }
        if words.next().is_some() {
            return Err("trailing data".into());
        }
    } else {
        return Err("invalid mode".into());
    }
    let mut coordinates = Vec::new();
    if args[2] == "points" {
        coordinates = std::fs::read_to_string(format!("{}.points", args[1]))?
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
    }
    if args[2] == "points" {
        values = Vec::new();
    }
    if !["yes", "no"].contains(&args[5].as_str()) {
        return Err("invalid representative selection".into());
    }
    if args[5] == "yes" && ["expanded", "approximate_expanded"].contains(&args[2].as_str()) {
        return Err("this timed adapter does not combine explicit expansion and bases".into());
    }
    // Fixture I/O, parsing and raw input buffers precede the measured workflow.
    drop(source);
    let baseline = memory("VmHWM:");
    let rss = memory("VmRSS:");
    let start = Instant::now();
    let mut phases = [0.; 5];
    let opts = PersistenceOptions::new(q, cutoff)?.with_field(PrimeField::new(p)?);
    let limits = Execution::default();
    let requests = if args[5] == "yes" {
        vec![RepresentativeRequest::new(
            q,
            cutoff.unwrap_or(1.),
            RepresentativeSelection::Both,
        )?]
    } else {
        Vec::new()
    };
    let mut edge_count = None;
    let mut simplex_count = None;
    let mut order = Vec::new();
    let mut retained = n;
    let result: PersistenceResult = if mode == "flag" {
        let graph = timed(&mut phases[0], || WeightedGraph::new(n, edges))?;
        edge_count = Some(graph.edge_count());
        let input = FlagFiltration::new(graph);
        timed(&mut phases[3], || {
            analyze(&input, &opts, &requests, &limits)
        })?
    } else if args[2] == "points" {
        let view = timed(&mut phases[0], || PointCloudView::new(&coordinates, n, 2))?;
        let mut builder = RipsBuilder::from_points(view);
        if let Some(t) = cutoff {
            builder = builder.max_edge_length(t);
        }
        let graph = timed(&mut phases[1], || builder.prepare())?;
        edge_count = Some(graph.graph().edge_count());
        timed(&mut phases[3], || {
            analyze(&graph, &opts, &requests, &limits)
        })?
    } else {
        let view = timed(&mut phases[0], || -> cocycle::Result<_> {
            if args[3] == "lower" {
                return Ok((MatrixLayout::LowerTriangle, Vec::new()));
            }
            let lower = DissimilarityMatrixView::new(&values, n, MatrixLayout::LowerTriangle)?;
            let layout = match args[3].as_str() {
                "lower" => MatrixLayout::LowerTriangle,
                "upper" => MatrixLayout::UpperTriangle,
                "square" => MatrixLayout::Square,
                _ => {
                    return Err(cocycle::Error::InvalidParameter {
                        parameter: "layout",
                        reason: "unknown",
                    });
                }
            };
            let converted = match layout {
                MatrixLayout::LowerTriangle => Vec::new(),
                MatrixLayout::UpperTriangle => (0..n)
                    .flat_map(|a| ((a + 1)..n).map(move |b| lower.get(a, b).unwrap()))
                    .collect(),
                MatrixLayout::Square => (0..n)
                    .flat_map(|a| (0..n).map(move |b| lower.get(a, b).unwrap()))
                    .collect(),
            };
            Ok((layout, converted))
        })?;
        let matrix_start = Instant::now();
        let matrix = DissimilarityMatrixView::new(
            if args[3] == "lower" { &values } else { &view.1 },
            n,
            view.0,
        )?;
        phases[0] += matrix_start.elapsed().as_secs_f64() * 1000.;
        match args[2].as_str() {
            "dense" => timed(&mut phases[3], || {
                analyze(
                    &RipsBuilder::from_distance_matrix(matrix),
                    &opts,
                    &requests,
                    &limits,
                )
            })?,
            "threshold" | "expanded" => {
                let graph = timed(&mut phases[1], || {
                    let mut builder = RipsBuilder::from_distance_matrix(matrix);
                    if let Some(t) = cutoff {
                        builder = builder.max_edge_length(t);
                    }
                    builder.prepare()
                })?;
                edge_count = Some(graph.graph().edge_count());
                if args[2] == "expanded" {
                    let expanded = timed(&mut phases[2], || graph.build_complex(q + 1))?;
                    simplex_count = Some(expanded.complex().len());
                    timed(&mut phases[3], || analyze(&expanded, &opts, &[], &limits))?
                } else {
                    timed(&mut phases[3], || {
                        analyze(&graph, &opts, &requests, &limits)
                    })?
                }
            }
            "approximate" | "approximate_checked" | "approximate_expanded" => {
                let policy = if args[2] == "approximate_checked" {
                    MetricPolicy::Check
                } else {
                    MetricPolicy::Assume
                };
                let mut builder =
                    ApproximateRipsBuilder::from_distance_matrix(matrix, args[4].parse()?, policy);
                if let Some(t) = cutoff {
                    builder = builder.max_filtration_value(t);
                }
                let graph = timed(&mut phases[1], || builder.prepare())?;
                edge_count = Some(graph.graph().edge_count());
                retained = graph.graph().vertex_count();
                order = graph.approximation().permutation().to_vec();
                if args[2] == "approximate_expanded" {
                    let expanded = timed(&mut phases[2], || graph.build_complex(q + 1))?;
                    simplex_count = Some(expanded.complex().len());
                    timed(&mut phases[3], || analyze(&expanded, &opts, &[], &limits))?
                } else {
                    timed(&mut phases[3], || {
                        analyze(&graph, &opts, &requests, &limits)
                    })?
                }
            }
            _ => return Err("unknown workflow".into()),
        }
    };
    let mut payload = String::new();
    timed(&mut phases[4], || {
        payload.push('[');
        for (i, interval) in result.diagram().intervals().iter().enumerate() {
            if i > 0 {
                payload.push(',');
            }
            write!(
                payload,
                "[{}, {:?},",
                interval.dimension(),
                interval.birth()
            )
            .unwrap();
            match interval.end() {
                IntervalEnd::Finite(d) => write!(payload, "{d:?}").unwrap(),
                _ => payload.push_str("null"),
            };
            payload.push(']');
        }
        payload.push(']');
    });
    let elapsed = start.elapsed().as_secs_f64() * 1000.;
    let peak = memory("VmHWM:");
    let coverage = match result.diagram().coverage() {
        Coverage::Complete => "null".into(),
        Coverage::Through(t) => format!("{t:?}"),
    };
    let optional = |v: Option<usize>| v.map_or_else(|| "null".into(), |v| v.to_string());
    let terms: usize = result
        .representatives()
        .unwrap_or_default()
        .iter()
        .map(|r| r.terms().len())
        .sum();
    println!(
        "{{\"status\":\"completed\",\"elapsed_ms\":{elapsed},\"phases_ms\":{:?},\"rss_before_kib\":{rss},\"hwm_before_kib\":{baseline},\"peak_rss_kib\":{peak},\"intervals\":{payload},\"coverage\":{coverage},\"vertices\":{retained},\"edges\":{},\"simplices\":{},\"representative_terms\":{terms},\"permutation\":{order:?}}}",
        phases,
        optional(edge_count),
        optional(simplex_count)
    );
    std::hint::black_box(result);
    Ok(())
}

// Preserve this worker's analysis range and field while exercising the public builder.
fn analyze(
    source: &impl PersistenceExt,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    limits: &Execution,
) -> cocycle::Result<cocycle::diagram::PersistenceResult> {
    let mut request = source
        .persistence()
        .max_homology_dimension(options.max_homology_dimension())
        .field(options.field())
        .representatives(requests);
    if let Some(cutoff) = options.max_edge() {
        request = request.max_filtration_value(cutoff);
    }
    request.compute_with(limits)
}
