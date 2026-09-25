//! A supplied cycle graph and an isolated vertex have their own complete filtration.
use cocycle::complex::{WeightedEdge, WeightedGraph};
use cocycle::diagram::IntervalEnd;
use cocycle::filtration::FlagFiltration;
use cocycle::persistence::PersistenceExt;
fn main() -> cocycle::Result<()> {
    let edges = [[0, 1], [1, 2], [2, 3], [0, 3]]
        .into_iter()
        .map(|vertices| WeightedEdge {
            vertices,
            value: 1.,
        })
        .collect();
    let input = FlagFiltration::new(WeightedGraph::new(5, edges)?);
    let result = input.persistence().compute()?;
    println!(
        "{:?}: {:?}",
        result.context().filtration_kind(),
        result.diagram().intervals()
    );
    assert_eq!(
        result
            .diagram()
            .intervals_in_dimension(1)?
            .next()
            .unwrap()
            .end(),
        IntervalEnd::Essential
    );
    Ok(())
}
