//! Descriptor formulas, interval endpoint semantics and numerical boundaries.

use cocycle::Error;
use cocycle::descriptors::{betti_curve, finite_lifetime_summary};
use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};

fn interval(dimension: usize, birth: f64, end: IntervalEnd) -> PersistenceInterval {
    PersistenceInterval::new(dimension, birth, end).unwrap()
}

#[test]
fn equal_lifetimes_have_natural_log_entropy_and_dimension_specific_exclusions() {
    let diagram = PersistenceDiagram::new(
        1,
        Coverage::Complete,
        vec![
            interval(0, 0., IntervalEnd::Finite(1.)),
            interval(0, 0., IntervalEnd::Finite(1.)),
            interval(0, 0., IntervalEnd::Essential),
            interval(1, 0., IntervalEnd::Essential),
        ],
    )
    .unwrap();
    let summary = finite_lifetime_summary(&diagram, 0).unwrap();
    assert_eq!(summary.finite_count(), 2);
    assert_eq!(summary.total_persistence(), 2.);
    assert_eq!(summary.max_persistence(), Some(1.));
    assert_eq!(summary.entropy(), Some(2.0_f64.ln()));
    assert_eq!(summary.excluded_essential_count(), 1);
    assert_eq!(summary.excluded_censored_count(), 0);
    let empty = finite_lifetime_summary(&diagram, 1).unwrap();
    assert_eq!(empty.finite_count(), 0);
    assert_eq!(empty.total_persistence(), 0.);
    assert_eq!(empty.max_persistence(), None);
    assert_eq!(empty.entropy(), None);
    assert_eq!(empty.excluded_essential_count(), 1);
}

#[test]
fn a_single_finite_interval_has_zero_entropy_and_censored_intervals_are_excluded() {
    let diagram = PersistenceDiagram::new(
        1,
        Coverage::Through(3.),
        vec![
            interval(0, 0., IntervalEnd::Finite(2.)),
            interval(0, 0., IntervalEnd::RightCensored { through: 3. }),
            interval(1, 1., IntervalEnd::RightCensored { through: 3. }),
        ],
    )
    .unwrap();
    let summary = finite_lifetime_summary(&diagram, 0).unwrap();
    assert_eq!(summary.entropy().unwrap().to_bits(), 0.0_f64.to_bits());
    assert_eq!(summary.total_persistence(), 2.);
    assert_eq!(summary.excluded_censored_count(), 1);
    assert_eq!(summary.excluded_essential_count(), 0);
}

#[test]
fn total_persistence_uses_compensation_for_disparate_lifetimes() {
    let mut intervals = vec![interval(0, 0., IntervalEnd::Finite(1e16))];
    // Later births keep the large term first in the diagram's canonical order.
    intervals.extend((0..100).map(|_| interval(0, 1., IntervalEnd::Finite(2.))));
    let diagram = PersistenceDiagram::new(0, Coverage::Complete, intervals).unwrap();
    assert_eq!(
        finite_lifetime_summary(&diagram, 0)
            .unwrap()
            .total_persistence(),
        1e16 + 100.
    );
}

#[test]
fn overflow_is_reported_and_extreme_probability_ratios_remain_finite() {
    for intervals in [
        vec![interval(0, -f64::MAX, IntervalEnd::Finite(f64::MAX))],
        vec![interval(0, 0., IntervalEnd::Finite(f64::MAX)); 2],
    ] {
        let diagram = PersistenceDiagram::new(0, Coverage::Complete, intervals).unwrap();
        assert!(matches!(
            finite_lifetime_summary(&diagram, 0),
            Err(Error::NumericalFailure { .. })
        ));
    }
    let diagram = PersistenceDiagram::new(
        0,
        Coverage::Complete,
        vec![
            interval(0, 0., IntervalEnd::Finite(f64::from_bits(1))),
            interval(0, 0., IntervalEnd::Finite(f64::MAX)),
        ],
    )
    .unwrap();
    assert!(
        finite_lifetime_summary(&diagram, 0)
            .unwrap()
            .entropy()
            .unwrap()
            .is_finite()
    );
}

#[test]
fn betti_counts_births_deaths_and_the_censoring_cutoff_correctly() {
    let diagram = PersistenceDiagram::new(
        0,
        Coverage::Through(2.),
        vec![
            interval(0, 0., IntervalEnd::Finite(1.)),
            interval(0, 1., IntervalEnd::Finite(2.)),
            interval(0, 1., IntervalEnd::RightCensored { through: 2. }),
        ],
    )
    .unwrap();
    assert_eq!(
        betti_curve(&diagram, 0, &[0., 0.5, 1., 1.5, 2.]).unwrap(),
        [1, 1, 2, 2, 1]
    );
    assert!(matches!(
        betti_curve(&diagram, 0, &[0., 3.]),
        Err(Error::QueryOutsideCoverage { index: 1, .. })
    ));
}

#[test]
fn grid_validation_rejects_invalid_order_values_and_uncomputed_dimensions() {
    let diagram = PersistenceDiagram::new(1, Coverage::Complete, vec![]).unwrap();
    for grid in [
        vec![f64::NAN],
        vec![f64::INFINITY],
        vec![1., 1.],
        vec![2., 1.],
        vec![-0., 0.],
    ] {
        assert!(matches!(
            betti_curve(&diagram, 0, &grid),
            Err(Error::InvalidGrid { .. })
        ));
    }
    assert_eq!(betti_curve(&diagram, 1, &[0., 1.]).unwrap(), [0, 0]);
    assert!(betti_curve(&diagram, 1, &[]).unwrap().is_empty());
    assert!(matches!(
        betti_curve(&diagram, 2, &[]),
        Err(Error::DimensionNotComputed { .. })
    ));
    assert!(matches!(
        finite_lifetime_summary(&diagram, 2),
        Err(Error::DimensionNotComputed { .. })
    ));
}

#[test]
fn translation_and_positive_rescaling_preserve_counts_and_normalized_entropy() {
    // Finite lifetimes are 2, 2, 3. Duplicate intervals remain separate terms.
    let expected_entropy =
        -2.0 * (2.0_f64 / 7.0) * (2.0_f64 / 7.0).ln() - (3.0_f64 / 7.0) * (3.0_f64 / 7.0).ln();
    for censored in [false, true] {
        for (factor, shift) in [(1.0, 0.0), (1.0, 8.0), (2.0, -4.0)] {
            let scale = |value: f64| factor * value + shift;
            let coverage = if censored {
                Coverage::Through(scale(4.0))
            } else {
                Coverage::Complete
            };
            let open_end = if censored {
                IntervalEnd::RightCensored {
                    through: scale(4.0),
                }
            } else {
                IntervalEnd::Essential
            };
            let repeated = interval(0, scale(-3.0), IntervalEnd::Finite(scale(-1.0)));
            let diagram = PersistenceDiagram::new(
                2,
                coverage,
                vec![
                    interval(1, scale(-4.0), IntervalEnd::Finite(scale(4.0))),
                    interval(0, scale(0.0), open_end),
                    interval(0, scale(-1.0), IntervalEnd::Finite(scale(2.0))),
                    repeated,
                    repeated,
                ],
            )
            .unwrap();
            let grid = [-4.0, -3.0, -1.0, 0.0, 2.0, 4.0].map(scale);
            assert_eq!(betti_curve(&diagram, 0, &grid).unwrap(), [0, 2, 1, 2, 1, 1]);
            assert_eq!(betti_curve(&diagram, 2, &grid).unwrap(), [0; 6]);
            let summary = finite_lifetime_summary(&diagram, 0).unwrap();
            assert_eq!(summary.finite_count(), 3);
            assert_eq!(summary.total_persistence(), 7.0 * factor);
            assert_eq!(summary.max_persistence(), Some(3.0 * factor));
            // The hand formula groups equal terms; allow rounding in the three
            // logarithmic contributions, while the integer lifetimes are exact.
            assert!((summary.entropy().unwrap() - expected_entropy).abs() < 8.0 * f64::EPSILON);
            assert_eq!(summary.excluded_essential_count(), usize::from(!censored));
            assert_eq!(summary.excluded_censored_count(), usize::from(censored));
            if censored {
                assert!(matches!(
                    betti_curve(&diagram, 0, &[scale(5.0)]),
                    Err(Error::QueryOutsideCoverage { .. })
                ));
            }
        }
    }
}

#[test]
fn descriptors_distinguish_gaps_from_computed_empty_dimensions() -> cocycle::Result<()> {
    use cocycle::diagram::ComputedDimensions;
    let separated = PersistenceDiagram::with_dimensions(
        ComputedDimensions::new(vec![1, 3])?,
        Coverage::Complete,
        vec![],
    )?;
    assert_eq!(betti_curve(&separated, 3, &[0., 1.])?, [0, 0]);
    assert_eq!(finite_lifetime_summary(&separated, 3)?.finite_count(), 0);
    for missing in [0, 2] {
        assert!(matches!(
            betti_curve(&separated, missing, &[]),
            Err(cocycle::Error::DimensionNotComputed { .. })
        ));
        assert!(matches!(
            finite_lifetime_summary(&separated, missing),
            Err(cocycle::Error::DimensionNotComputed { .. })
        ));
    }
    Ok(())
}
