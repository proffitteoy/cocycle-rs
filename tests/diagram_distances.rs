//! Public contracts and an independent exhaustive partial-matching oracle.

use cocycle::Error;
use cocycle::algebra::PrimeField;
use cocycle::complex::WeightedGraph;
use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use cocycle::diagram_distances::{
    bottleneck_distance, bottleneck_distance_results, bottleneck_distance_results_with,
    bottleneck_distance_with, wasserstein_1_infinity, wasserstein_1_infinity_results,
    wasserstein_1_infinity_results_with, wasserstein_1_infinity_with, wasserstein_2_euclidean,
    wasserstein_2_euclidean_results, wasserstein_2_euclidean_results_with,
    wasserstein_2_euclidean_with,
};
use cocycle::execution::Execution;
use cocycle::filtration::FlagFiltration;
use cocycle::persistence::{ExecutionLimits, PersistenceOptions, compute_flag};
use std::sync::atomic::{AtomicBool, Ordering};

type Distance = fn(&PersistenceDiagram, &PersistenceDiagram, usize) -> cocycle::Result<f64>;
type ControlledDistance =
    fn(&PersistenceDiagram, &PersistenceDiagram, usize, &Execution<'_>) -> cocycle::Result<f64>;
const CONTROLLED: [ControlledDistance; 3] = [
    bottleneck_distance_with,
    wasserstein_1_infinity_with,
    wasserstein_2_euclidean_with,
];

#[test]
fn default_controls_preserve_values_and_errors() {
    let cases = [
        diagram(&[]),
        diagram(&[[-2., 0.]]),
        diagram(&[[0., 2.], [0., 2.], [3., 4.]]),
        with_essential(&[[0., 1.]], &[0., 2.]),
        with_essential(&[], &[1.]),
        PersistenceDiagram::new(1, Coverage::Complete, vec![]).unwrap(),
        PersistenceDiagram::new(0, Coverage::Through(2.), vec![]).unwrap(),
        diagram(&[[0., f64::from_bits(1)]]),
    ];
    for (legacy, controlled) in DISTANCES.into_iter().zip(CONTROLLED) {
        for a in &cases {
            for b in &cases {
                for dimension in [0, 1, 3] {
                    assert_eq!(
                        legacy(a, b, dimension),
                        controlled(a, b, dimension, &Execution::default())
                    );
                    assert_eq!(
                        legacy(a, b, dimension),
                        controlled(a, b, dimension, &Execution::default().max_work(u64::MAX))
                    );
                }
            }
        }
    }
}

#[test]
fn public_budgets_exhaust_recover_and_restart_for_each_call() {
    let a = with_essential(&[[0., 2.], [1., 4.]], &[0., 3.]);
    let b = with_essential(&[[0.5, 2.5]], &[0.25, 3.5]);
    let original_a = a.clone();
    let original_b = b.clone();
    for (legacy, controlled) in DISTANCES.into_iter().zip(CONTROLLED) {
        let expected = legacy(&a, &b, 0).unwrap();
        let mut success = false;
        for limit in 0..20_000 {
            let execution = Execution::default().max_work(limit);
            match controlled(&a, &b, 0, &execution) {
                Err(error) => {
                    assert_eq!(error, Error::WorkLimitExceeded { limit });
                    assert_eq!(
                        controlled(&a, &b, 0, &Execution::default()).unwrap(),
                        expected
                    );
                }
                Ok(value) => {
                    assert!(limit > 0);
                    assert_eq!(value, expected);
                    assert_eq!(controlled(&a, &b, 0, &execution).unwrap(), expected);
                    success = true;
                    break;
                }
            }
        }
        assert!(success);
        let flag = AtomicBool::new(true);
        let execution = Execution::new(Some(0), Some(&flag));
        assert_eq!(controlled(&a, &b, 0, &execution), Err(Error::Cancelled));
        assert!(flag.load(Ordering::Relaxed));
        assert_eq!(
            controlled(&a, &b, 0, &Execution::default()).unwrap(),
            expected
        );
        flag.store(false, Ordering::Relaxed);
        assert_eq!(
            controlled(&a, &b, 0, &execution),
            Err(Error::WorkLimitExceeded { limit: 0 })
        );
        let cancellation = Execution::default().cancellation(&flag);
        assert_eq!(controlled(&a, &b, 0, &cancellation).unwrap(), expected);
        flag.store(true, Ordering::Relaxed);
        for dimension in [0, 5] {
            assert_eq!(
                controlled(&diagram(&[]), &diagram(&[]), dimension, &cancellation),
                Err(Error::Cancelled)
            );
        }
        assert_eq!(
            controlled(
                &diagram(&[]),
                &diagram(&[]),
                0,
                &Execution::default().max_work(0)
            ),
            Err(Error::WorkLimitExceeded { limit: 0 })
        );
    }
    assert_eq!(a, original_a);
    assert_eq!(b, original_b);
}

#[test]
fn result_controls_preserve_context_checks_and_cover_the_operation() -> cocycle::Result<()> {
    use cocycle::diagram::PersistenceData;
    use cocycle::filtration::RipsBuilder;
    use cocycle::geometry::PointCloudView;
    use cocycle::persistence::PersistenceExt;
    type ResultDistance = fn(&PersistenceData, &PersistenceData, usize) -> cocycle::Result<f64>;
    type ControlledResultDistance =
        fn(&PersistenceData, &PersistenceData, usize, &Execution<'_>) -> cocycle::Result<f64>;
    let source = RipsBuilder::from_points(PointCloudView::new(&[0., 1., 3.], 3, 1)?);
    let a = source.persistence().compute()?.into_data();
    let b = source
        .persistence()
        .field(PrimeField::new(3)?)
        .compute()?
        .into_data();
    let functions: [(ResultDistance, ControlledResultDistance); 3] = [
        (
            bottleneck_distance_results,
            bottleneck_distance_results_with,
        ),
        (
            wasserstein_1_infinity_results,
            wasserstein_1_infinity_results_with,
        ),
        (
            wasserstein_2_euclidean_results,
            wasserstein_2_euclidean_results_with,
        ),
    ];
    // Cancellation requested after the operation starts, during a caller's
    // AsRef conversion, must be observed even without a work limit.
    struct CancelOnBorrow<'a>(&'a PersistenceData, &'a AtomicBool);
    impl AsRef<PersistenceData> for CancelOnBorrow<'_> {
        fn as_ref(&self) -> &PersistenceData {
            self.1.store(true, Ordering::Relaxed);
            self.0
        }
    }
    let flag = AtomicBool::new(false);
    for distance in [
        bottleneck_distance_results_with::<PersistenceData, CancelOnBorrow<'_>>,
        wasserstein_1_infinity_results_with,
        wasserstein_2_euclidean_results_with,
    ] {
        flag.store(false, Ordering::Relaxed);
        assert_eq!(
            distance(
                &a,
                &CancelOnBorrow(&a, &flag),
                0,
                &Execution::default().cancellation(&flag)
            ),
            Err(Error::Cancelled)
        );
    }
    for (legacy, controlled) in functions {
        for right in [&a, &b] {
            for dim in [0, 1, 7] {
                assert_eq!(
                    legacy(&a, right, dim),
                    controlled(&a, right, dim, &Execution::default())
                );
            }
        }
        let expected = legacy(&a, &a, 0)?;
        let mut success = false;
        for limit in 0..20_000 {
            match controlled(&a, &a, 0, &Execution::default().max_work(limit)) {
                Err(error) => assert_eq!(error, Error::WorkLimitExceeded { limit }),
                Ok(value) => {
                    assert_eq!(value, expected);
                    success = true;
                    break;
                }
            }
            assert_eq!(controlled(&a, &a, 0, &Execution::default())?, expected);
        }
        assert!(success);
        let flag = AtomicBool::new(true);
        assert_eq!(
            controlled(&a, &a, 0, &Execution::default().cancellation(&flag)),
            Err(Error::Cancelled)
        );
        assert_eq!(controlled(&a, &a, 0, &Execution::default())?, expected);
    }
    Ok(())
}

#[test]
fn distances_respect_gaps_and_selected_dimensions() -> cocycle::Result<()> {
    use cocycle::diagram::ComputedDimensions;
    let only_h1 = PersistenceDiagram::with_dimensions(
        ComputedDimensions::new(vec![1])?,
        Coverage::Complete,
        vec![],
    )?;
    let separated = PersistenceDiagram::with_dimensions(
        ComputedDimensions::new(vec![1, 3])?,
        Coverage::Complete,
        vec![],
    )?;
    for distance in [
        bottleneck_distance,
        wasserstein_1_infinity,
        wasserstein_2_euclidean,
    ] {
        // A selected common dimension does not require identical full domains.
        assert_eq!(distance(&only_h1, &separated, 1)?, 0.);
        for missing in [0, 2, 3] {
            assert!(matches!(
                distance(&only_h1, &separated, missing),
                Err(cocycle::Error::DimensionNotComputed { .. })
            ));
            assert!(matches!(
                distance(&separated, &only_h1, missing),
                Err(cocycle::Error::DimensionNotComputed { .. })
            ));
        }
    }
    Ok(())
}

#[test]
fn compatible_results_borrow_common_data_and_keep_witness_indices() -> cocycle::Result<()> {
    use cocycle::{
        algebra::PrimeField,
        diagram::{PersistenceData, PersistenceResult},
        filtration::RipsBuilder,
        geometry::PointCloudView,
        persistence::{PersistenceExt, RepresentativeRequest, RepresentativeSelection},
    };
    struct ResearchResult {
        data: PersistenceData,
        iterations: usize,
    }
    impl AsRef<PersistenceData> for ResearchResult {
        fn as_ref(&self) -> &PersistenceData {
            &self.data
        }
    }
    let points = PointCloudView::new(&[0., 1., 2.], 3, 1)?;
    let source = RipsBuilder::from_points(points);
    let requests = [RepresentativeRequest::new(
        0,
        0.5,
        RepresentativeSelection::Both,
    )?];
    let result = source.persistence().representatives(&requests).compute()?;
    let common: &PersistenceData = result.as_ref();
    assert_eq!(common.diagram(), result.diagram());
    assert!(std::ptr::eq(common.context(), result.context()));
    let intervals: Vec<_> = result.diagram().intervals().collect();
    let witnesses = result.representatives().unwrap().to_vec();
    let (data, representatives) = result.into_parts();
    assert_eq!(data.diagram().intervals().collect::<Vec<_>>(), intervals);
    assert_eq!(representatives.as_deref(), Some(witnesses.as_slice()));
    for witness in representatives.unwrap() {
        let interval = &data.diagram().interval(witness.interval_index()).unwrap();
        assert_eq!(interval.dimension(), witness.dimension());
        assert!(interval.birth() <= witness.scale());
    }
    let research = ResearchResult {
        data,
        iterations: 7,
    };
    let plain = source.persistence().compute()?;
    assert_eq!(research.iterations, 7);
    assert_eq!(bottleneck_distance_results(&research, &plain, 0)?, 0.);
    assert_eq!(
        wasserstein_1_infinity_results(&plain, &research.data, 0)?,
        0.
    );
    assert_eq!(
        wasserstein_2_euclidean_results(&research, &research, 0)?,
        0.
    );
    let distance: fn(&PersistenceResult, &ResearchResult, usize) -> cocycle::Result<f64> =
        bottleneck_distance_results;
    assert_eq!(distance(&plain, &research, 0)?, 0.);
    let execution = Execution::default();
    assert_eq!(
        bottleneck_distance_results_with(&plain, &research, 0, &execution)?,
        0.
    );
    assert_eq!(
        wasserstein_1_infinity_results_with(&research, &research.data, 0, &execution)?,
        0.
    );
    let erased: &dyn AsRef<PersistenceData> = &research;
    assert_eq!(
        wasserstein_2_euclidean_results_with(erased, &plain, 0, &execution)?,
        0.
    );
    let mod3 = source
        .persistence()
        .field(PrimeField::new(3)?)
        .compute()?
        .into_data();
    assert!(matches!(
        bottleneck_distance_results(&mod3, &research, 0),
        Err(cocycle::Error::IncompatibleDiagramContext { .. })
    ));
    let moved = plain.into_data();
    assert!(std::ptr::eq(moved.as_ref(), &moved));
    assert_eq!(moved.into_diagram(), research.data.into_diagram());
    Ok(())
}
const DISTANCES: [Distance; 3] = [
    bottleneck_distance,
    wasserstein_1_infinity,
    wasserstein_2_euclidean,
];

fn diagram(points: &[[f64; 2]]) -> PersistenceDiagram {
    PersistenceDiagram::new(
        0,
        Coverage::Complete,
        points
            .iter()
            .map(|&[b, d]| PersistenceInterval::new(0, b, IntervalEnd::Finite(d)).unwrap())
            .collect(),
    )
    .unwrap()
}

fn with_essential(points: &[[f64; 2]], births: &[f64]) -> PersistenceDiagram {
    let mut intervals = diagram(points).intervals().collect::<Vec<_>>();
    intervals.extend(
        births
            .iter()
            .map(|&b| PersistenceInterval::new(0, b, IntervalEnd::Essential).unwrap()),
    );
    PersistenceDiagram::new(0, Coverage::Complete, intervals).unwrap()
}

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 128.0 * f64::EPSILON * expected.abs().max(1.0),
        "actual={actual:.17e}, expected={expected:.17e}"
    );
}

#[test]
fn hand_derived_diagonal_costs_and_repeated_intervals() {
    let a = diagram(&[[0., 2.], [0., 2.]]);
    let empty = diagram(&[]);
    assert_eq!(bottleneck_distance(&a, &empty, 0).unwrap(), 1.);
    assert_eq!(wasserstein_1_infinity(&a, &empty, 0).unwrap(), 2.);
    close(wasserstein_2_euclidean(&a, &empty, 0).unwrap(), 2.);
    let single = diagram(&[[0., 2.]]);
    assert_eq!(bottleneck_distance(&a, &single, 0).unwrap(), 1.);
    assert_eq!(wasserstein_1_infinity(&a, &single, 0).unwrap(), 1.);
    close(
        wasserstein_2_euclidean(&a, &single, 0).unwrap(),
        2.0_f64.sqrt(),
    );
}

#[test]
fn ground_norm_and_wasserstein_order_are_independent() {
    let a = diagram(&[[0., 10.], [20., 30.]]);
    let b = diagram(&[[1., 11.], [21., 31.]]);
    assert_eq!(bottleneck_distance(&a, &b, 0).unwrap(), 1.);
    assert_eq!(wasserstein_1_infinity(&a, &b, 0).unwrap(), 2.);
    close(wasserstein_2_euclidean(&a, &b, 0).unwrap(), 2.);
}

#[test]
fn empty_self_distance_and_computed_dimension_contracts() {
    let empty = diagram(&[]);
    let larger_range = PersistenceDiagram::new(3, Coverage::Complete, vec![]).unwrap();
    let a = diagram(&[[-5., -2.], [-5., -2.], [2., 8.]]);
    for distance in DISTANCES {
        assert_eq!(
            distance(&empty, &empty, 0).unwrap().to_bits(),
            0.0_f64.to_bits()
        );
        assert_eq!(distance(&a, &a, 0).unwrap(), 0.);
        assert_eq!(distance(&empty, &larger_range, 0).unwrap(), 0.);
        assert!(matches!(
            distance(&a, &empty, 1),
            Err(Error::DimensionNotComputed { .. })
        ));
    }
}

#[test]
fn incomplete_coverage_is_rejected_even_without_censored_points() {
    let partial_empty = PersistenceDiagram::new(1, Coverage::Through(3.), vec![]).unwrap();
    let partial_finite = PersistenceDiagram::new(
        1,
        Coverage::Through(3.),
        vec![PersistenceInterval::new(0, 1., IntervalEnd::Finite(2.)).unwrap()],
    )
    .unwrap();
    let partial_censored = PersistenceDiagram::new(
        1,
        Coverage::Through(3.),
        vec![PersistenceInterval::new(0, 1., IntervalEnd::RightCensored { through: 3. }).unwrap()],
    )
    .unwrap();
    let complete = diagram(&[]);
    for distance in DISTANCES {
        for partial in [&partial_empty, &partial_finite, &partial_censored] {
            assert!(matches!(
                distance(partial, &complete, 0),
                Err(Error::IncompleteDiagram { .. })
            ));
            assert!(matches!(
                distance(&complete, partial, 0),
                Err(Error::IncompleteDiagram { .. })
            ));
        }
    }
}

#[test]
fn essential_matching_preserves_counts_and_aggregates_finite_costs() {
    let a = with_essential(&[[0., 2.]], &[10., 0.]);
    let b = with_essential(&[], &[12., 1.]);
    assert_eq!(bottleneck_distance(&a, &b, 0).unwrap(), 2.);
    assert_eq!(wasserstein_1_infinity(&a, &b, 0).unwrap(), 4.);
    close(wasserstein_2_euclidean(&a, &b, 0).unwrap(), 7.0_f64.sqrt());
    let wrong_count = with_essential(&[], &[1.]);
    for distance in DISTANCES {
        assert_eq!(distance(&a, &wrong_count, 0).unwrap(), f64::INFINITY);
        assert_eq!(distance(&wrong_count, &a, 0).unwrap(), f64::INFINITY);
    }
}

#[test]
fn essential_overflow_is_an_error_not_count_mismatch_infinity() {
    let a = with_essential(&[], &[-f64::MAX]);
    let b = with_essential(&[], &[f64::MAX]);
    for distance in DISTANCES {
        assert!(matches!(
            distance(&a, &b, 0),
            Err(Error::NumericalFailure { .. })
        ));
    }
}

#[test]
fn extreme_representable_diagonal_distance_avoids_intermediate_overflow() {
    let a = diagram(&[[-f64::MAX, f64::MAX]]);
    let empty = diagram(&[]);
    assert_eq!(bottleneck_distance(&a, &empty, 0).unwrap(), f64::MAX);
    assert_eq!(wasserstein_1_infinity(&a, &empty, 0).unwrap(), f64::MAX);
    assert!(matches!(
        wasserstein_2_euclidean(&a, &empty, 0),
        Err(Error::NumericalFailure { .. })
    ));
    let a = diagram(&[[0., 1e200]]);
    close(
        wasserstein_2_euclidean(&a, &empty, 0).unwrap(),
        1e200 / 2.0_f64.sqrt(),
    );
}

#[test]
fn tiny_distances_and_near_identical_w2_do_not_cancel() {
    let empty = diagram(&[]);
    let a = diagram(&[[0., 1e-200]]);
    let actual = wasserstein_2_euclidean(&a, &empty, 0).unwrap();
    let expected = 1e-200 / 2.0_f64.sqrt();
    assert!((actual / expected - 1.).abs() < 1e-14);
    let a = diagram(&[[0., 1.], [2., 4.]]);
    let delta = 2.0_f64.powi(-40);
    let b = diagram(&[[delta, 1. + delta], [2. + delta, 4. + delta]]);
    let actual = wasserstein_2_euclidean(&a, &b, 0).unwrap();
    assert!((actual / (2. * delta) - 1.).abs() < 1e-12, "{actual}");
}

#[test]
fn adjacent_float_diagonal_costs_do_not_require_representable_midpoints() {
    let birth = 1.0_f64;
    let death = birth.next_up();
    let expected = (death - birth) * 0.5;
    let a = diagram(&[[birth, death]]);
    let empty = diagram(&[]);
    assert_eq!(bottleneck_distance(&a, &empty, 0).unwrap(), expected);
    assert_eq!(wasserstein_1_infinity(&a, &empty, 0).unwrap(), expected);
    let w2 = wasserstein_2_euclidean(&a, &empty, 0).unwrap();
    assert!((w2 / (expected * 2.0_f64.sqrt()) - 1.0).abs() < 1e-14);
}

#[test]
fn near_identical_w2_selects_matching_before_reconstructing_cost() {
    // The optimal cross assignment swaps the two columns. Both savings round to
    // the same large diagonal baseline, so reconstructing an arbitrary saving
    // maximizer afterwards is insufficient to recover the correct distance.
    let e = 2.0_f64.powi(-40);
    let first = diagram(&[[0., 2.], [e, 2. + 2. * e]]);
    let second = diagram(&[[0., 2. + 2. * e], [e, 2.]]);
    let expected = 2.0_f64.sqrt() * e;
    let actual = wasserstein_2_euclidean(&first, &second, 0).unwrap();
    assert!(
        (actual / expected - 1.).abs() < 1e-12,
        "{actual:e} != {expected:e}"
    );
}

#[test]
fn narrow_lifetimes_at_large_offsets_keep_profitable_sweep_edges() {
    // Each widely separated block has one profitable cross edge. The first
    // point's real midpoint lies halfway between adjacent representable values.
    // Computing savings from rounded rotated coordinates can erase that edge.
    let mut a = Vec::new();
    let mut b = Vec::new();
    let mut expected_w1 = 0.;
    let mut expected_w2 = 0.0_f64;
    let mut expected_bn = 0.0_f64;
    for i in 0..40 {
        let birth = i as f64 * 32.0 + 1.0;
        let one = birth.next_up();
        let two = one.next_up();
        a.push([birth, one]);
        b.push([birth, two]);
        let cost = one - birth;
        expected_w1 += cost;
        expected_w2 = expected_w2.hypot(cost);
        expected_bn = expected_bn.max(cost);
    }
    let first = diagram(&a);
    let second = diagram(&b);
    assert_eq!(
        bottleneck_distance(&first, &second, 0).unwrap(),
        expected_bn
    );
    let actual_w1 = wasserstein_1_infinity(&first, &second, 0).unwrap();
    let actual_w2 = wasserstein_2_euclidean(&first, &second, 0).unwrap();
    assert!(
        (actual_w1 / expected_w1 - 1.0).abs() < 1e-12,
        "{actual_w1:e} != {expected_w1:e}"
    );
    assert!(
        (actual_w2 / expected_w2 - 1.0).abs() < 1e-12,
        "{actual_w2:e} != {expected_w2:e}"
    );
}

#[test]
fn input_types_reject_diagonal_invalid_and_infinite_endpoints() {
    for (b, d) in [(1., 1.), (2., 1.), (f64::NAN, 3.), (0., f64::INFINITY)] {
        assert!(PersistenceInterval::new(0, b, IntervalEnd::Finite(d)).is_err());
    }
    assert!(PersistenceInterval::new(0, f64::NEG_INFINITY, IntervalEnd::Essential).is_err());
}

// Enumerate every partial injection A -> B. Unassigned points in either set go
// to the diagonal. This uses no candidate graph, dual, pruning or production code.
fn exhaustive(a: &[[f64; 2]], b: &[[f64; 2]], metric: usize) -> f64 {
    fn diagonal(p: [f64; 2], metric: usize) -> f64 {
        let length = p[1] - p[0];
        if metric == 2 {
            length * length / 2.
        } else {
            length / 2.
        }
    }
    fn combine(a: f64, b: f64, metric: usize) -> f64 {
        if metric == 0 { a.max(b) } else { a + b }
    }
    fn visit(
        a: &[[f64; 2]],
        b: &[[f64; 2]],
        row: usize,
        used: u32,
        metric: usize,
        cost: f64,
    ) -> f64 {
        if row == a.len() {
            return b
                .iter()
                .enumerate()
                .filter(|(j, _)| used & (1 << j) == 0)
                .fold(cost, |v, (_, &p)| combine(v, diagonal(p, metric), metric));
        }
        let mut best = visit(
            a,
            b,
            row + 1,
            used,
            metric,
            combine(cost, diagonal(a[row], metric), metric),
        );
        for (j, &point) in b.iter().enumerate() {
            if used & (1 << j) != 0 {
                continue;
            }
            let db = a[row][0] - point[0];
            let dd = a[row][1] - point[1];
            let edge = if metric == 2 {
                db * db + dd * dd
            } else {
                db.abs().max(dd.abs())
            };
            best = best.min(visit(
                a,
                b,
                row + 1,
                used | (1 << j),
                metric,
                combine(cost, edge, metric),
            ));
        }
        best
    }
    let value = visit(a, b, 0, 0, metric, 0.);
    if metric == 2 { value.sqrt() } else { value }
}

#[test]
fn seeded_small_diagrams_match_independent_exhaustive_oracle() {
    let mut seed = 20260922_u64;
    let mut next = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        seed >> 32
    };
    for case in 0..180 {
        let n = case % 6;
        let m = (case / 6) % 6;
        let mut make = |count| -> Vec<[f64; 2]> {
            (0..count)
                .map(|_| {
                    let birth = (next() % 16) as f64 / 4. - 2.;
                    [birth, birth + (next() % 8 + 1) as f64 / 4.]
                })
                .collect()
        };
        let a = make(n);
        let b = make(m);
        let da = diagram(&a);
        let db = diagram(&b);
        for (metric, distance) in DISTANCES.into_iter().enumerate() {
            let expected = exhaustive(&a, &b, metric);
            let actual = distance(&da, &db, 0).unwrap();
            if metric < 2 {
                assert_eq!(actual, expected, "case {case}, metric {metric}");
            } else {
                close(actual, expected);
            }
            close(distance(&db, &da, 0).unwrap(), expected);
        }
    }
}

#[test]
fn result_wrappers_validate_fields_without_requiring_identical_contexts() {
    let graph = FlagFiltration::new(WeightedGraph::new(1, vec![]).unwrap());
    let limits = ExecutionLimits::default();
    let options = PersistenceOptions::new(0, None).unwrap();
    let a = compute_flag(&graph, &options, &limits).unwrap();
    let b = compute_flag(
        &graph,
        &options.with_field(PrimeField::new(3).unwrap()),
        &limits,
    )
    .unwrap();
    assert!(matches!(
        bottleneck_distance_results(&a, &b, 0),
        Err(Error::IncompatibleDiagramContext { .. })
    ));
    assert!(matches!(
        wasserstein_1_infinity_results(&a, &b, 0),
        Err(Error::IncompatibleDiagramContext { .. })
    ));
    assert!(matches!(
        wasserstein_2_euclidean_results(&a, &b, 0),
        Err(Error::IncompatibleDiagramContext { .. })
    ));
    assert_eq!(
        bottleneck_distance(a.diagram(), b.diagram(), 0).unwrap(),
        0.
    );
    let other = FlagFiltration::new(WeightedGraph::new(2, vec![]).unwrap());
    let c = compute_flag(&other, &options, &limits).unwrap();
    assert_eq!(
        bottleneck_distance_results(&a, &c, 0).unwrap(),
        f64::INFINITY
    );
    assert_eq!(wasserstein_1_infinity_results(&a, &a, 0).unwrap(), 0.);
    assert_eq!(wasserstein_2_euclidean_results(&a, &a, 0).unwrap(), 0.);
}

#[test]
fn result_wrappers_reject_unspecified_scales_but_raw_signed_diagrams_remain_usable() {
    use cocycle::complex::{Simplex, SimplicialComplex};
    use cocycle::persistence::{PersistenceBuilder, PersistenceExt};

    let source = SimplicialComplex::new(vec![Simplex::new(vec![0], -2.).unwrap()]).unwrap();
    let supplied = source
        .persistence()
        .max_homology_dimension(0)
        .compute()
        .unwrap();
    let cells = PersistenceBuilder::from_complex(&source)
        .max_homology_dimension(0)
        .compute()
        .unwrap();
    let graph = FlagFiltration::new(WeightedGraph::new(1, vec![]).unwrap());
    let edge_lengths = graph
        .persistence()
        .max_homology_dimension(0)
        .compute()
        .unwrap();
    for (raw, contextual) in DISTANCES.into_iter().zip([
        bottleneck_distance_results,
        wasserstein_1_infinity_results,
        wasserstein_2_euclidean_results,
    ]) {
        // Even self-comparison cannot certify units for an unspecified source.
        for (left, right) in [
            (&supplied, &supplied),
            (&supplied, &cells),
            (&cells, &cells),
            (&supplied, &edge_lengths),
            (&edge_lengths, &supplied),
        ] {
            assert!(matches!(
                contextual(left, right, 0),
                Err(Error::IncompatibleDiagramContext { .. })
            ));
        }
        assert_eq!(raw(supplied.diagram(), cells.diagram(), 0).unwrap(), 0.);
        // The caller explicitly interprets these births in a common scalar unit.
        assert_eq!(
            raw(supplied.diagram(), edge_lengths.diagram(), 0).unwrap(),
            2.
        );
    }
}

#[test]
fn certified_expansion_keeps_scale_convention_but_bare_storage_does_not() {
    use cocycle::filtration::RipsBuilder;
    use cocycle::geometry::PointCloudView;
    use cocycle::persistence::PersistenceExt;

    let points = PointCloudView::new(&[0., 1.], 2, 1).unwrap();
    let builder = RipsBuilder::from_points(points);
    let implicit = builder
        .persistence()
        .max_homology_dimension(0)
        .compute()
        .unwrap();
    let expanded = builder.build_complex(1).unwrap();
    let certified = expanded
        .persistence()
        .max_homology_dimension(0)
        .compute()
        .unwrap();
    let bare = expanded
        .complex()
        .persistence()
        .max_homology_dimension(0)
        .compute()
        .unwrap();
    for (raw, contextual) in DISTANCES.into_iter().zip([
        bottleneck_distance_results,
        wasserstein_1_infinity_results,
        wasserstein_2_euclidean_results,
    ]) {
        assert_eq!(contextual(&implicit, &certified, 0).unwrap(), 0.);
        assert_eq!(contextual(&certified, &implicit, 0).unwrap(), 0.);
        assert!(matches!(
            contextual(&certified, &bare, 0),
            Err(Error::IncompatibleDiagramContext { .. })
        ));
        assert_eq!(raw(certified.diagram(), bare.diagram(), 0).unwrap(), 0.);
    }
}

#[test]
fn modified_sparse_edge_values_measure_diagrams_in_the_declared_parameter() {
    use cocycle::filtration::{ApproximateRipsBuilder, RipsBuilder};
    use cocycle::geometry::{MetricPolicy, PointCloudView};
    use cocycle::persistence::PersistenceExt;

    for unit in [1., 2.] {
        let coordinates = [0., 3. * unit];
        let points = PointCloudView::new(&coordinates, 2, 1).unwrap();
        let exact = RipsBuilder::from_points(points)
            .persistence()
            .max_homology_dimension(0)
            .compute()
            .unwrap();
        // With epsilon=3 and later insertion radius 3*unit, the retained edge
        // is 2*(3*unit - 3*unit/3) = 4*unit. No interleaving bound is asserted
        // for epsilon >= 1. Both diagrams also have one essential birth at 0.
        let sparse = ApproximateRipsBuilder::from_points(points, 3., MetricPolicy::Check)
            .persistence()
            .max_homology_dimension(0)
            .compute()
            .unwrap();
        assert!(
            exact
                .diagram()
                .intervals()
                .any(|i| i.end() == IntervalEnd::Finite(3. * unit))
        );
        assert!(
            sparse
                .diagram()
                .intervals()
                .any(|i| i.end() == IntervalEnd::Finite(4. * unit))
        );
        for (raw, contextual) in DISTANCES.into_iter().zip([
            bottleneck_distance_results,
            wasserstein_1_infinity_results,
            wasserstein_2_euclidean_results,
        ]) {
            // Matching the finite points costs unit; sending both to the
            // diagonal is more expensive under each of the three metrics.
            assert_eq!(contextual(&exact, &sparse, 0).unwrap(), unit);
            assert_eq!(raw(exact.diagram(), sparse.diagram(), 0).unwrap(), unit);
        }
    }
}

#[test]
fn approximate_result_wrappers_measure_actual_diagrams_and_retain_provenance() {
    use cocycle::filtration::{SparseRipsOptions, sparse_rips_from_points};
    use cocycle::geometry::{MetricPolicy, PointCloudView};
    use cocycle::persistence::compute_sparse_rips;

    let coordinates = [0., 1., 2.];
    let points = PointCloudView::new(&coordinates, 3, 1).unwrap();
    let first = sparse_rips_from_points(
        points,
        &SparseRipsOptions::new(0.25, MetricPolicy::Check).unwrap(),
    )
    .unwrap();
    let second = sparse_rips_from_points(
        points,
        &SparseRipsOptions::new(0.5, MetricPolicy::Check).unwrap(),
    )
    .unwrap();
    let options = PersistenceOptions::new(0, None).unwrap();
    let limits = ExecutionLimits::default();
    let first = compute_sparse_rips(&first, &options, &limits).unwrap();
    let second = compute_sparse_rips(&second, &options, &limits).unwrap();
    let before = first.context().clone();
    assert!(first.context().approximation().is_some());
    assert_ne!(first.context(), second.context());
    assert_eq!(
        bottleneck_distance_results(&first, &second, 0).unwrap(),
        bottleneck_distance(first.diagram(), second.diagram(), 0).unwrap()
    );
    assert_eq!(
        wasserstein_1_infinity_results(&first, &second, 0).unwrap(),
        wasserstein_1_infinity(first.diagram(), second.diagram(), 0).unwrap()
    );
    assert_eq!(
        wasserstein_2_euclidean_results(&first, &second, 0).unwrap(),
        wasserstein_2_euclidean(first.diagram(), second.diagram(), 0).unwrap()
    );
    assert_eq!(first.context(), &before);
}
