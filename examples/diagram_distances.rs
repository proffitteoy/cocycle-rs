//! Compare complete diagrams using three distinct matching metrics.
use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use cocycle::diagram_distances::{
    bottleneck_distance, wasserstein_1_infinity, wasserstein_2_euclidean,
};

fn main() -> cocycle::Result<()> {
    let first = PersistenceDiagram::new(
        1,
        Coverage::Complete,
        vec![PersistenceInterval::new(1, 0., IntervalEnd::Finite(2.))?],
    )?;
    let second = PersistenceDiagram::new(1, Coverage::Complete, vec![])?;
    let bottleneck = bottleneck_distance(&first, &second, 1)?;
    let w1 = wasserstein_1_infinity(&first, &second, 1)?;
    let w2 = wasserstein_2_euclidean(&first, &second, 1)?;
    assert_eq!(bottleneck, 1.);
    assert_eq!(w1, 1.);
    assert!((w2 - 2.0_f64.sqrt()).abs() < 1e-14);
    println!("bottleneck(L-inf)={bottleneck}, W1(L-inf)={w1}, W2(L2)={w2}");
    Ok(())
}
