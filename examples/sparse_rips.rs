//! Approximate a point cloud with owned sampling provenance and requested bases.
use cocycle::algebra::PrimeField;
use cocycle::filtration::ApproximateRipsBuilder;
use cocycle::geometry::{MetricPolicy, PointCloudView};
use cocycle::persistence::{PersistenceExt, RepresentativeRequest, RepresentativeSelection};
fn main() -> cocycle::Result<()> {
    let points = [0., 0., 1., 0., 1., 1., 0., 1., 0., 0.];
    let input = PointCloudView::new(&points, 5, 2)?;
    let construction = ApproximateRipsBuilder::from_points(input, 0.5, MetricPolicy::Check);
    let requests = [RepresentativeRequest::new(
        1,
        1.,
        RepresentativeSelection::Both,
    )?];
    let result = construction
        .persistence()
        .field(PrimeField::new(3)?)
        .representatives(&requests)
        .compute()?;
    let metadata = result.context().approximation().unwrap();
    println!(
        "original vertices: {}; retained: {:?}",
        metadata.permutation().len(),
        metadata.retained_vertices()
    );
    println!("conditional ideal-arithmetic bound: {:?}", metadata.bound());
    println!("approximation coverage: {:?}", result.diagram().coverage());
    for interval in result.diagram().intervals() {
        println!("{interval:?}");
    }
    for representative in result.representatives().unwrap() {
        println!("{representative:?}");
    }
    Ok(())
}
