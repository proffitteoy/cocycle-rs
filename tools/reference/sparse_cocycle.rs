//! Sparse Rips native worker: topology, implicit/explicit persistence and basis parity.
use cocycle::algebra::PrimeField;
use cocycle::diagram::{Coverage, IntervalEnd};
use cocycle::execution::Execution;
use cocycle::filtration::ApproximateRipsBuilder;
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout, MetricPolicy};
use cocycle::persistence::{
    PersistenceExt, PersistenceOptions, RepresentativeRequest, RepresentativeSelection,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 {
        return Err("usage: sparse_cocycle FIXTURE EPSILON MIN_RADIUS START DIMENSION".into());
    }
    let file = std::fs::read_to_string(&args[1])?;
    let mut words = file.split_whitespace();
    if words.next() != Some("dense") {
        return Err("dense fixture required".into());
    }
    let n: usize = words.next().ok_or("n")?.parse()?;
    let q: usize = words.next().ok_or("q")?.parse()?;
    let cutoff = words.next().ok_or("cutoff")?;
    let cutoff = if cutoff == "none" {
        None
    } else {
        Some(cutoff.parse()?)
    };
    let count: usize = words.next().ok_or("count")?.parse()?;
    let characteristic: u32 = words.next().ok_or("field")?.parse()?;
    let values: Vec<f64> = words.map(str::parse).collect::<Result<_, _>>()?;
    if values.len() != count {
        return Err("count".into());
    }
    let matrix = DissimilarityMatrixView::new(&values, n, MatrixLayout::LowerTriangle)?;
    let mut builder =
        ApproximateRipsBuilder::from_distance_matrix(matrix, args[2].parse()?, MetricPolicy::Check)
            .min_insertion_radius(args[3].parse()?);
    if let Some(t) = cutoff {
        builder = builder.max_filtration_value(t);
    }
    if n > 0 {
        builder = builder.start_vertex(args[4].parse()?);
    }
    let input = builder.prepare()?;
    let dimension: usize = args[5].parse()?;
    let expanded = input.build_complex(dimension)?;
    let compute = dimension > q || dimension >= n;
    let result = if compute {
        let opts = PersistenceOptions::new(q, None)?.with_field(PrimeField::new(characteristic)?);
        let limits = Execution::default();
        let implicit = analyze(&input, &opts, &[], &limits)?;
        if implicit.diagram() != analyze(&expanded, &opts, &[], &limits)?.diagram() {
            return Err("explicit disagreement".into());
        }
        let scale = match input.coverage() {
            Coverage::Through(t) => t.min(25.),
            Coverage::Complete => 25.,
        };
        let requests: Vec<_> = (0..=q)
            .map(|d| RepresentativeRequest::new(d, scale, RepresentativeSelection::Both))
            .collect::<Result<_, _>>()?;
        if implicit.diagram() != analyze(&input, &opts, &requests, &limits)?.diagram() {
            return Err("representative disagreement".into());
        }
        Some(implicit)
    } else {
        None
    };
    print!("{{\"status\":\"ok\",\"characteristic\":{characteristic},\"simplices\":[");
    for (i, simplex) in expanded.complex().simplices().iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        print!("[{:?},{:?}]", simplex.vertices(), simplex.value());
    }
    print!("],\"intervals\":[");
    if let Some(result) = &result {
        for (i, interval) in result.diagram().intervals().iter().enumerate() {
            if i > 0 {
                print!(",");
            }
            print!("[{},{:?},", interval.dimension(), interval.birth());
            match interval.end() {
                IntervalEnd::Finite(d) => print!("{d:?}"),
                _ => print!("null"),
            };
            print!("]");
        }
    }
    print!("],\"coverage\":");
    match input.coverage() {
        Coverage::Complete => print!("null"),
        Coverage::Through(t) => print!("{t:?}"),
    };
    println!("}}");
    print!(
        "{{\"permutation\":{:?},\"radii\":[",
        input.approximation().permutation()
    );
    for (i, radius) in input.approximation().insertion_radii().iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        match radius {
            Some(r) => print!("{r:?}"),
            None => print!("null"),
        };
    }
    println!("]}}");
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
