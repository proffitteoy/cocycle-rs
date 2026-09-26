//! Analyze hand-built interval multisets without constructing a complex.

use cocycle::Result;
use cocycle::descriptors::{betti_curve, finite_lifetime_summary};
use cocycle::diagram::{
    ComputedDimensions, Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval,
};

fn main() -> Result<()> {
    // Two repeated H1 intervals of lifetime two, plus an essential class.
    // H0 is a computed empty dimension. Input order need not be sorted.
    let finite = PersistenceInterval::new(1, -1.0, IntervalEnd::Finite(1.0))?;
    let complete = PersistenceDiagram::new(
        1,
        Coverage::Complete,
        vec![
            PersistenceInterval::new(1, 0.0, IntervalEnd::Essential)?,
            finite,
            finite,
        ],
    )?;
    let grid = [-2.0, -1.0, 0.0, 1.0, 2.0];
    let counts = betti_curve(&complete, 1, &grid)?;
    let summary = finite_lifetime_summary(&complete, 1)?;
    assert_eq!(counts, [0, 2, 3, 1, 1]);
    assert_eq!(summary.total_persistence(), 4.0);
    assert_eq!(summary.excluded_essential_count(), 1);
    println!("Complete H1 Betti curve at {grid:?}: {counts:?}");
    println!("Observed finite H1 lifetimes: {summary:?}");

    // This is a separate supplied diagram, observed only through scale one.
    // The class born at zero is censored: its eventual death is unknown.
    let truncated = PersistenceDiagram::new(
        1,
        Coverage::Through(1.0),
        vec![
            finite,
            finite,
            PersistenceInterval::new(1, 0.0, IntervalEnd::RightCensored { through: 1.0 })?,
        ],
    )?;
    assert_eq!(betti_curve(&truncated, 1, &grid[..4])?, [0, 2, 3, 1]);
    let summary = finite_lifetime_summary(&truncated, 1)?;
    assert_eq!(summary.excluded_censored_count(), 1);
    println!("Truncated H1 finite lifetimes (censored class excluded): {summary:?}");
    println!(
        "Computed empty H0: {:?}",
        betti_curve(&truncated, 0, &[0.0])?
    );
    // An H1-only algorithm must not declare H0 computed merely by omitting bars.
    let h1_only = PersistenceDiagram::with_dimensions(
        ComputedDimensions::new(vec![1])?,
        Coverage::Complete,
        vec![finite],
    )?;
    assert_eq!(betti_curve(&h1_only, 1, &[0.])?, [1]);
    assert!(betti_curve(&h1_only, 0, &[0.]).is_err());
    println!(
        "H1-only computed dimensions: {:?}",
        h1_only.computed_dimensions().iter().collect::<Vec<_>>()
    );
    Ok(())
}
