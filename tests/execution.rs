//! Whole-operation budgets, cancellation boundaries and reusable source recovery.
use cocycle::{
    Error, Result,
    execution::Execution,
    filtration::{ApproximateRipsBuilder, RipsBuilder},
    geometry::{MetricPolicy, PointCloudView},
    persistence::{PersistenceExt, RepresentativeRequest, RepresentativeSelection},
};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

fn minimum<T>(mut run: impl FnMut(u64) -> Result<T>) -> u64 {
    for limit in 0..20_000 {
        match run(limit) {
            Ok(_) => return limit,
            Err(Error::WorkLimitExceeded { .. }) => {}
            Err(error) => panic!("unexpected error: {error}"),
        }
    }
    panic!("small fixture exceeded test search range")
}
#[test]
fn preparation_expansion_and_reduction_share_one_exact_budget() -> Result<()> {
    let values = [0., 1., 2., 3.];
    let rips = RipsBuilder::from_points(PointCloudView::new(&values, 4, 1)?).max_edge_length(2.);
    let prepared = rips.prepare()?;
    let prepare_work = minimum(|limit| rips.prepare_with(&Execution::default().max_work(limit)));
    let expansion_work =
        minimum(|limit| prepared.build_complex_with(2, &Execution::default().max_work(limit)));
    let combined_work =
        minimum(|limit| rips.build_complex_with(2, &Execution::default().max_work(limit)));
    assert!(prepare_work > 0 && expansion_work > 0);
    assert_eq!(combined_work, prepare_work + expansion_work);
    let compute_work = minimum(|limit| {
        prepared
            .persistence()
            .compute_with(&Execution::default().max_work(limit))
    });
    let direct_work = minimum(|limit| {
        rips.persistence()
            .compute_with(&Execution::default().max_work(limit))
    });
    assert_eq!(direct_work, prepare_work + compute_work);
    let execution = Execution::default().max_work(direct_work);
    assert_eq!(
        rips.persistence().compute_with(&execution)?,
        rips.persistence().compute_with(&execution)?
    );
    assert!(matches!(
        rips.persistence()
            .compute_with(&Execution::default().max_work(direct_work - 1)),
        Err(Error::WorkLimitExceeded { .. })
    ));
    assert_eq!(
        rips.persistence().compute()?,
        rips.persistence().compute_with(&Execution::default())?
    );
    Ok(())
}
#[test]
fn approximation_budget_includes_metric_sampling_and_expansion() -> Result<()> {
    let values = [0., 1., 2., 3.];
    let input = PointCloudView::new(&values, 4, 1)?;
    let approximate = ApproximateRipsBuilder::from_points(input, 0.5, MetricPolicy::Check);
    let prepared = approximate.prepare()?;
    let prepare_work = minimum(|n| approximate.prepare_with(&Execution::default().max_work(n)));
    let unchecked = ApproximateRipsBuilder::from_points(input, 0.5, MetricPolicy::Unchecked);
    let unchecked_work = minimum(|n| unchecked.prepare_with(&Execution::default().max_work(n)));
    assert!(prepare_work > unchecked_work);
    let expansion_work =
        minimum(|n| prepared.build_complex_with(2, &Execution::default().max_work(n)));
    let combined_work =
        minimum(|n| approximate.build_complex_with(2, &Execution::default().max_work(n)));
    assert_eq!(combined_work, prepare_work + expansion_work);
    let compute_work = minimum(|n| {
        prepared
            .persistence()
            .compute_with(&Execution::default().max_work(n))
    });
    let direct_work = minimum(|n| {
        approximate
            .persistence()
            .compute_with(&Execution::default().max_work(n))
    });
    assert_eq!(direct_work, prepare_work + compute_work);
    Ok(())
}
#[test]
fn cancellation_polls_callbacks_and_never_resets_the_callers_flag() -> Result<()> {
    let values = [0, 1, 2, 3];
    let cancelled = AtomicBool::new(false);
    let calls = Cell::new(0);
    let execution = Execution::default().cancellation(&cancelled);
    let callback = RipsBuilder::from_distance_fn(&values, |_, _| {
        calls.set(calls.get() + 1);
        cancelled.store(true, Ordering::Relaxed);
        Ok(1.)
    });
    assert!(matches!(
        callback.build_complex_with(2, &execution),
        Err(Error::Cancelled)
    ));
    assert_eq!(calls.get(), 1);
    assert!(cancelled.load(Ordering::Relaxed));
    let callback = ApproximateRipsBuilder::from_distance_fn(
        &values,
        |_, _| {
            calls.set(calls.get() + 1);
            Ok(1.)
        },
        0.5,
        MetricPolicy::Check,
    );
    assert!(matches!(
        callback.prepare_with(&execution),
        Err(Error::Cancelled)
    ));
    assert_eq!(calls.get(), 1);
    cancelled.store(false, Ordering::Relaxed);
    let callback = ApproximateRipsBuilder::from_distance_fn(
        &values,
        |_, _| {
            calls.set(calls.get() + 1);
            cancelled.store(true, Ordering::Relaxed);
            Ok(1.)
        },
        0.5,
        MetricPolicy::Check,
    );
    assert!(matches!(
        callback.prepare_with(&execution),
        Err(Error::Cancelled)
    ));
    assert_eq!(calls.get(), 2);
    cancelled.store(false, Ordering::Relaxed);
    let prepared =
        RipsBuilder::from_distance_fn(&values, |_, _| Ok(1.)).prepare_with(&execution)?;
    assert_eq!(prepared.graph().edge_count(), 6);
    Ok(())
}
#[test]
fn invalid_requests_precede_preparation_and_zero_budget_calls_no_callback() -> Result<()> {
    let values = [0., 1., 2.];
    let points = PointCloudView::new(&values, 3, 1)?;
    let requests = [RepresentativeRequest::new(
        2,
        1.,
        RepresentativeSelection::Both,
    )?];
    let execution = Execution::default().max_work(0);
    assert!(matches!(
        RipsBuilder::from_points(points)
            .persistence()
            .representatives(&requests)
            .compute_with(&execution),
        Err(Error::DimensionNotComputed { .. })
    ));
    let calls = Cell::new(0);
    assert!(matches!(
        RipsBuilder::from_distance_fn(&values, |_, _| {
            calls.set(calls.get() + 1);
            Ok(1.)
        })
        .prepare_with(&execution),
        Err(Error::WorkLimitExceeded { .. })
    ));
    assert_eq!(calls.get(), 0);
    assert!(RipsBuilder::from_points(points).build_complex(2).is_ok());
    Ok(())
}
