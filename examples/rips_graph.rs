//! Construct and inspect an exact Rips graph, then compute its persistence.
use cocycle::descriptors::betti_curve;
use cocycle::filtration::RipsBuilder;
use cocycle::geometry::PointCloudView;
use cocycle::persistence::PersistenceExt;
fn main() -> cocycle::Result<()> {
    let coordinates = [0., 0., 1., 0., 1., 1., 0., 1.];
    let graph = RipsBuilder::from_points(PointCloudView::new(&coordinates, 4, 2)?)
        .max_edge_length(1.)
        .prepare()?;
    println!(
        "{} vertices, {} edges; {:?}",
        graph.graph().vertex_count(),
        graph.graph().edge_count(),
        graph.coverage()
    );
    let result = graph.persistence().compute()?;
    println!(
        "H1 Betti curve: {:?}",
        betti_curve(result.diagram(), 1, &[0., 1.])?
    );
    assert_eq!(betti_curve(result.diagram(), 1, &[0., 1.])?, [0, 1]);
    Ok(())
}
