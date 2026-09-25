//! Native correctness worker for the new graph and matrix APIs.
use cocycle::algebra::PrimeField;
use cocycle::complex::{WeightedEdge, WeightedGraph};
use cocycle::diagram::{Coverage, IntervalEnd};
use cocycle::execution::Execution;
use cocycle::filtration::{FlagFiltration, RipsBuilder};
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout};
use cocycle::persistence::{
    PersistenceExt, PersistenceOptions, RepresentativeRequest, RepresentativeSelection,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string(std::env::args().nth(1).ok_or("fixture missing")?)?;
    let (header, data) = file.split_once('\n').ok_or("header newline")?;
    let mut words = header.split_whitespace();
    let mode = words.next().ok_or("mode")?;
    let n: usize = words.next().ok_or("n")?.parse()?;
    let q: usize = words.next().ok_or("q")?.parse()?;
    let cutoff = words.next().ok_or("cutoff")?;
    let cutoff: Option<f64> = if cutoff == "none" {
        None
    } else {
        Some(cutoff.parse()?)
    };
    let count: usize = words.next().ok_or("count")?.parse()?;
    let characteristic: u32 = words.next().unwrap_or("2").parse()?;
    if words.next().is_some() {
        return Err("trailing header fields".into());
    }
    let mut words = data.split_whitespace();
    let options = PersistenceOptions::new(q, cutoff)?.with_field(PrimeField::new(characteristic)?);
    let requests = [RepresentativeRequest::new(
        q,
        cutoff.unwrap_or(1.).min(1.),
        RepresentativeSelection::Both,
    )?];
    let limits = Execution::default();
    let (edges, result, complex) = if mode == "dense" {
        let values: Vec<f64> = words.map(str::parse).collect::<Result<_, _>>()?;
        if values.len() != count {
            return Err("value count".into());
        }
        let matrix = DissimilarityMatrixView::new(&values, n, MatrixLayout::LowerTriangle)?;
        let mut rips = RipsBuilder::from_distance_matrix(matrix);
        if let Some(t) = cutoff {
            rips = rips.max_edge_length(t);
        }
        let construction = rips.prepare()?;
        let result = analyze(&construction, &options, &[], &limits)?;
        let represented = analyze(&construction, &options, &requests, &limits)?;
        if result.diagram() != represented.diagram() {
            return Err("representative diagram disagreement".into());
        }
        // Exercise the new dense layout engine on every dense fixture too.
        if result.diagram()
            != analyze(
                &RipsBuilder::from_distance_matrix(matrix),
                &options,
                &[],
                &limits,
            )?
            .diagram()
        {
            return Err("dense/sparse disagreement".into());
        }
        let expanded = construction.build_complex(q + 1)?;
        if result.diagram() != analyze(&expanded, &options, &[], &limits)?.diagram() {
            return Err("implicit/explicit disagreement".into());
        }
        (
            construction.graph().edges().to_vec(),
            result,
            expanded.complex().clone(),
        )
    } else if mode == "flag" {
        let mut edges = Vec::new();
        for _ in 0..count {
            edges.push(WeightedEdge {
                vertices: [
                    words.next().ok_or("a")?.parse()?,
                    words.next().ok_or("b")?.parse()?,
                ],
                value: words.next().ok_or("value")?.parse()?,
            });
        }
        let filtration = FlagFiltration::new(WeightedGraph::new(n, edges)?);
        let result = analyze(&filtration, &options, &[], &limits)?;
        let represented = analyze(&filtration, &options, &requests, &limits)?;
        if result.diagram() != represented.diagram() {
            return Err("representative diagram disagreement".into());
        }
        let edges = filtration
            .graph()
            .edges()
            .iter()
            .filter(|e| cutoff.is_none_or(|t| e.value <= t))
            .copied()
            .collect::<Vec<_>>();
        let filtered = FlagFiltration::new(WeightedGraph::new(n, edges.clone())?);
        (
            edges,
            result,
            filtered.build_complex(q + 1)?.complex().clone(),
        )
    } else {
        return Err("unknown mode".into());
    };
    print!("{{\"status\":\"ok\",\"coverage\":");
    match result.diagram().coverage() {
        Coverage::Complete => print!("null"),
        Coverage::Through(t) => print!("{t:?}"),
    }
    print!(
        ",\"characteristic\":{},\"representative_diagram_parity\":true,\"edges\":[",
        result.context().characteristic()
    );
    for (i, edge) in edges.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        print!(
            "[{},{},{:?}]",
            edge.vertices[0], edge.vertices[1], edge.value
        );
    }
    print!("],\"intervals\":[");
    for (i, interval) in result.diagram().intervals().iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        print!("[{}, {:?},", interval.dimension(), interval.birth());
        match interval.end() {
            IntervalEnd::Finite(d) => print!("{d:?}"),
            _ => print!("null"),
        }
        print!("]");
    }
    print!("],\"simplices\":[");
    for (i, simplex) in complex.simplices().iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        print!("[{:?},{:?}]", simplex.vertices(), simplex.value());
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
