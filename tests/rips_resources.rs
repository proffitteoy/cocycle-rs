//! Resource failures are local to a call and do not corrupt reusable inputs.
use cocycle::algebra::PrimeField;
use cocycle::diagram::{
    Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval, PersistenceResult,
};
use cocycle::filtration::{
    FlagFiltration, SparseRipsOptions, sparse_rips_from_distances, threshold_rips_from_distances,
};
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout, MetricPolicy};
use cocycle::persistence::*;
use cocycle::{Error, Result};
use std::sync::atomic::{AtomicBool, Ordering};

type Computation<'a> = Box<dyn Fn(&ExecutionLimits<'_>) -> Result<PersistenceResult> + 'a>;

fn square_expectation(coverage: Coverage, h1_end: IntervalEnd) -> PersistenceDiagram {
    // Four unit edges connect the vertices at 1 and create exactly one cycle.
    // Diagonals at 2 kill it; absent diagonals leave it essential.
    let mut intervals = vec![PersistenceInterval::new(0, 0., IntervalEnd::Finite(1.)).unwrap(); 3];
    let h0_end = match coverage {
        Coverage::Complete => IntervalEnd::Essential,
        Coverage::Through(through) => IntervalEnd::RightCensored { through },
    };
    intervals.push(PersistenceInterval::new(0, 0., h0_end).unwrap());
    intervals.push(PersistenceInterval::new(1, 1., h1_end).unwrap());
    PersistenceDiagram::new(1, coverage, intervals).unwrap()
}

fn check_every_work_budget(
    compute: impl Fn(&ExecutionLimits<'_>) -> Result<PersistenceResult>,
    expected: &PersistenceDiagram,
) {
    // Do not hard-code the successful budget: optimizations change counted work.
    // The cap only keeps a broken tiny-fixture test from looping indefinitely.
    for limit in 0..=4096 {
        match compute(&ExecutionLimits::new(Some(limit), None)) {
            Err(error) => {
                assert_eq!(error, Error::WorkLimitExceeded { limit });
                assert_eq!(
                    compute(&ExecutionLimits::default()).unwrap().diagram(),
                    expected,
                    "recovery after work limit {limit}"
                );
            }
            Ok(result) => {
                assert!(limit > 0);
                assert_eq!(result.diagram(), expected);
                assert_eq!(
                    compute(&ExecutionLimits::new(Some(limit), None)).unwrap(),
                    result
                );
                return;
            }
        }
    }
    panic!("tiny H1 fixture did not finish within the test's work-budget cap");
}

#[test]
fn f2_h1_paths_recover_at_every_work_budget() {
    let values = [1., 2., 1., 1., 2., 1.];
    let view = DissimilarityMatrixView::new(&values, 4, MatrixLayout::LowerTriangle).unwrap();
    let exact = threshold_rips_from_distances(view, None).unwrap();
    let flag = FlagFiltration::new(exact.graph().clone());
    for cutoff in [None, Some(1.)] {
        let options = PersistenceOptions::new(1, cutoff).unwrap();
        let (coverage, h1_end) =
            cutoff.map_or((Coverage::Complete, IntervalEnd::Finite(2.)), |through| {
                (
                    Coverage::Through(through),
                    IntervalEnd::RightCensored { through },
                )
            });
        let expected = square_expectation(coverage, h1_end);
        let paths: Vec<Computation<'_>> = vec![
            Box::new(|limits| compute_rips_from_distances(view, &options, limits)),
            Box::new(|limits| compute_threshold_rips(&exact, &options, limits)),
            Box::new(|limits| compute_flag(&flag, &options, limits)),
        ];
        for compute in paths {
            check_every_work_budget(compute, &expected);
        }
    }
    // The supplied four-cycle has no later diagonals. Its surviving H1 class
    // is essential, unlike the right-censored class of the truncated matrix.
    let cycle = threshold_rips_from_distances(view, Some(1.)).unwrap();
    let flag = FlagFiltration::new(cycle.graph().clone());
    check_every_work_budget(
        |limits| compute_flag(&flag, &PersistenceOptions::default(), limits),
        &square_expectation(Coverage::Complete, IntervalEnd::Essential),
    );
}

#[test]
fn all_rips_paths_recover_after_budget_and_cancellation_failures() {
    let values: Vec<_> = (0..8)
        .flat_map(|b| (0..b).map(move |a| if a / 2 == b / 2 { 2. } else { 1. }))
        .collect();
    let view = DissimilarityMatrixView::new(&values, 8, MatrixLayout::LowerTriangle).unwrap();
    let exact = threshold_rips_from_distances(view, None).unwrap();
    let expanded = exact.expand(4).unwrap();
    let flag = FlagFiltration::new(exact.graph().clone());
    let sparse = sparse_rips_from_distances(
        view,
        &SparseRipsOptions::new(0.5, MetricPolicy::Check).unwrap(),
    )
    .unwrap();
    let sparse_expanded = sparse.expand(4).unwrap();
    let cancelled = AtomicBool::new(false);
    for p in [2, 3] {
        let options = PersistenceOptions::new(3, None)
            .unwrap()
            .with_field(PrimeField::new(p).unwrap());
        let requests = [RepresentativeRequest::new(3, 1., RepresentativeSelection::Both).unwrap()];
        let paths: Vec<Computation<'_>> = vec![
            Box::new(|limits| compute_rips_from_distances(view, &options, limits)),
            Box::new(|limits| compute_threshold_rips(&exact, &options, limits)),
            Box::new(|limits| compute_flag(&flag, &options, limits)),
            Box::new(|limits| compute_expanded_rips(&expanded, &options, limits)),
            Box::new(|limits| compute_sparse_rips(&sparse, &options, limits)),
            Box::new(|limits| compute_expanded_sparse_rips(&sparse_expanded, &options, limits)),
            Box::new(|limits| {
                compute_rips_from_distances_with_representatives(view, &options, &requests, limits)
            }),
            Box::new(|limits| {
                compute_threshold_rips_with_representatives(&exact, &options, &requests, limits)
            }),
            Box::new(|limits| {
                compute_flag_with_representatives(&flag, &options, &requests, limits)
            }),
            Box::new(|limits| {
                compute_expanded_rips_with_representatives(&expanded, &options, &requests, limits)
            }),
            Box::new(|limits| {
                compute_sparse_rips_with_representatives(&sparse, &options, &requests, limits)
            }),
            Box::new(|limits| {
                compute_expanded_sparse_rips_with_representatives(
                    &sparse_expanded,
                    &options,
                    &requests,
                    limits,
                )
            }),
        ];
        for compute in paths {
            let expected = compute(&ExecutionLimits::default()).unwrap();
            for limit in [0, 7, 31] {
                assert_eq!(
                    compute(&ExecutionLimits::new(Some(limit), None)).unwrap_err(),
                    Error::WorkLimitExceeded { limit }
                );
            }
            cancelled.store(true, Ordering::Relaxed);
            assert_eq!(
                compute(&ExecutionLimits::new(None, Some(&cancelled))).unwrap_err(),
                Error::Cancelled
            );
            assert!(cancelled.load(Ordering::Relaxed));
            cancelled.store(false, Ordering::Relaxed);
            assert_eq!(
                compute(&ExecutionLimits::new(None, Some(&cancelled))).unwrap(),
                expected
            );
        }
    }
}

#[test]
fn simultaneous_prime_field_calls_share_only_immutable_input() {
    let values = [1., 2., 1., 1., 2., 1.];
    let view = DissimilarityMatrixView::new(&values, 4, MatrixLayout::LowerTriangle).unwrap();
    let input = threshold_rips_from_distances(view, None).unwrap();
    let expected = square_expectation(Coverage::Complete, IntervalEnd::Finite(2.));
    std::thread::scope(|scope| {
        let handles: Vec<_> = [2, 2, 3, 5, 251]
            .into_iter()
            .map(|p| {
                let input = &input;
                let expected = &expected;
                scope.spawn(move || {
                    let options = PersistenceOptions::new(1, None)
                        .unwrap()
                        .with_field(PrimeField::new(p).unwrap());
                    check_every_work_budget(
                        |limits| compute_threshold_rips(input, &options, limits),
                        expected,
                    );
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    });
}
