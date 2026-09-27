//! Exact matching distances between complete persistence diagrams.
//!
//! Each operation compares one explicitly computed homology dimension and keeps
//! interval multiplicity. Finite points may match other finite points or the
//! diagonal. Essential points `(birth, +infinity)` match only essential points;
//! unequal essential counts give positive infinity. Truncated coverage is an
//! error even when no interval in the requested dimension is right-censored.
//!
//! "Exact" means no algorithmic approximation, not exact real arithmetic. The
//! solvers use binary64 arithmetic. Numerical failure is an error, never a NaN or
//! a substitute infinite distance. The existing diagram types require finite
//! births and positive finite lifetimes; diagonal points and other infinite
//! endpoint categories are outside this API's input domain.
//!
//! Raw diagrams do not establish field or source compatibility. The `_results`
//! functions additionally require equal fields and declared edge-length parameter
//! conventions. Both APIs leave units and normalization to the caller; matching
//! conventions do not establish those facts. Distances between sparse
//! approximations describe the two supplied diagrams, not the unknown original
//! diagrams and not an approximation error certificate.
//!
//! # Execution controls
//!
//! Each raw and context-aware function has a `_with` variant accepting
//! [`Execution`]. Existing functions use `Execution::default()`. One fresh private
//! budget spans context/coverage validation, interval extraction, preparation,
//! candidates, all matching routes and accumulation. Counted work includes
//! intervals, candidate or graph rows, search visits and batches; repeated scans
//! count again. Counts are algorithm-dependent, not timings, bytes or a stable
//! performance score. A zero budget rejects even an empty distance operation.
//!
//! A pre-set cancellation flag is checked before validation or extraction;
//! otherwise validation and resource errors occur in execution order. Long loops
//! poll at row, visit or batch boundaries. Sorting, selection, allocations and
//! user `AsRef` callbacks cannot be interrupted internally; cancellation is
//! cooperative, without a hard latency guarantee. A resource error returns no
//! partial distance and changes neither input nor the caller's flag. Each retry
//! starts fresh. Workspace quotas and internal parallelism controls are outside
//! this API; matching routes and scratch remain private.
//!
//! ```
//! use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
//! use cocycle::diagram_distances::{bottleneck_distance, wasserstein_2_euclidean};
//! let first = PersistenceDiagram::new(0, Coverage::Complete, vec![
//!     PersistenceInterval::new(0, 0.0, IntervalEnd::Finite(2.0))?,
//! ])?;
//! let empty = PersistenceDiagram::new(0, Coverage::Complete, vec![])?;
//! assert_eq!(bottleneck_distance(&first, &empty, 0)?, 1.0);
//! assert!((wasserstein_2_euclidean(&first, &empty, 0)? - 2.0_f64.sqrt()).abs() < 1e-14);
//! # Ok::<(), cocycle::Error>(())
//! ```

use crate::diagram::{Coverage, IntervalEnd, PersistenceData, PersistenceDiagram};
use crate::execution::{Execution, WorkBudget};
use crate::{Error, Result};

// The standalone worker opts in with --cfg cocycle_distance_bench. Neither
// counter updates nor their argument evaluation exist in ordinary crate builds.
macro_rules! record {
    ($($body:tt)*) => {
        #[cfg(any(test, cocycle_distance_bench))]
        { $($body)* }
    };
}

// Keep ablation controls out of the normal build; the second expression is the
// selected production policy. This is compile-time selection, including debug.
macro_rules! experiment {
    ($value:expr, $production:expr) => {{
        #[cfg(any(test, cocycle_distance_bench))]
        {
            $value
        }
        #[cfg(not(any(test, cocycle_distance_bench)))]
        {
            $production
        }
    }};
}

pub(crate) mod bottleneck;
pub(crate) mod wasserstein;

/// Exact bottleneck distance, with pointwise L-infinity cost.
///
/// A finite point `(b, d)` has diagonal cost `(d-b)/2`. Empty finite diagrams
/// have distance zero. Essential counts must agree; otherwise returns infinity.
///
/// # Errors
/// Returns an error for uncomputed dimensions, incomplete coverage, allocation
/// failure or unrepresentable numerical work. See the module's input contract.
pub fn bottleneck_distance(
    first: &PersistenceDiagram,
    second: &PersistenceDiagram,
    dimension: usize,
) -> Result<f64> {
    bottleneck_distance_with(first, second, dimension, &Execution::default())
}

/// Bottleneck distance with cooperative work and cancellation controls.
///
/// One budget covers validation, point preparation, matching and accumulation.
/// See the module's execution contract for counted work and cancellation limits.
/// # Errors
/// Includes [`bottleneck_distance`] errors, [`Error::Cancelled`] and
/// [`Error::WorkLimitExceeded`]. No partial distance is returned.
pub fn bottleneck_distance_with(
    first: &PersistenceDiagram,
    second: &PersistenceDiagram,
    dimension: usize,
    execution: &Execution<'_>,
) -> Result<f64> {
    distance_with(first, second, dimension, Kind::Bottleneck, execution)
}

/// Exact order-one Wasserstein distance, with pointwise L-infinity cost.
///
/// Sums costs, including diagonal cost `(d-b)/2`. Empty finite diagrams have
/// distance zero. This is distinct from Wasserstein with Euclidean ground cost.
///
/// # Errors
/// Returns an error for uncomputed dimensions, incomplete coverage, allocation
/// failure or numerical overflow/underflow. Essential count mismatch is infinity.
pub fn wasserstein_1_infinity(
    first: &PersistenceDiagram,
    second: &PersistenceDiagram,
    dimension: usize,
) -> Result<f64> {
    wasserstein_1_infinity_with(first, second, dimension, &Execution::default())
}

/// W1-L-infinity distance with the execution contract of [`bottleneck_distance_with`].
/// # Errors
/// Includes [`wasserstein_1_infinity`] errors, [`Error::Cancelled`] and
/// [`Error::WorkLimitExceeded`]. No partial distance is returned.
pub fn wasserstein_1_infinity_with(
    first: &PersistenceDiagram,
    second: &PersistenceDiagram,
    dimension: usize,
    execution: &Execution<'_>,
) -> Result<f64> {
    distance_with(first, second, dimension, Kind::W1, execution)
}

/// Exact order-two Wasserstein distance, with pointwise Euclidean cost.
///
/// Minimizes the sum of squared costs and returns its square root. The diagonal
/// cost is `(d-b)/sqrt(2)`. Empty finite diagrams have distance zero.
///
/// # Errors
/// Returns an error for uncomputed dimensions, incomplete coverage, allocation
/// failure or numerical overflow/underflow. Essential count mismatch is infinity.
pub fn wasserstein_2_euclidean(
    first: &PersistenceDiagram,
    second: &PersistenceDiagram,
    dimension: usize,
) -> Result<f64> {
    wasserstein_2_euclidean_with(first, second, dimension, &Execution::default())
}

/// W2-Euclidean distance with the execution contract of [`bottleneck_distance_with`].
/// # Errors
/// Includes [`wasserstein_2_euclidean`] errors, [`Error::Cancelled`] and
/// [`Error::WorkLimitExceeded`]. No partial distance is returned.
pub fn wasserstein_2_euclidean_with(
    first: &PersistenceDiagram,
    second: &PersistenceDiagram,
    dimension: usize,
    execution: &Execution<'_>,
) -> Result<f64> {
    distance_with(first, second, dimension, Kind::W2, execution)
}

/// Bottleneck distance with additional computation-context validation.
///
/// Accepts common data and compatible wrappers through a borrow of already stored
/// [`PersistenceData`]. This adapter performs no copying or context reconstruction.
///
/// Requires equal coefficient characteristics and declared edge-length scales.
/// Unspecified scales are rejected, including two unspecified scales: equality
/// does not establish comparable units. Call the raw-diagram function after
/// establishing a common scale yourself for supplied filtrations. Even declared
/// edge lengths do not certify physical units or normalization across datasets;
/// the caller must establish them before interpreting the returned distance.
/// Original and modified sparse edge values use the same parameter convention,
/// but describe different filtrations. Their distance compares the resulting
/// diagrams and is not a distance between the original pairwise metrics.
/// Vertex counts, construction kinds and cutoff requests
/// need not agree when both diagrams have complete coverage. Approximation
/// provenance remains in the borrowed results; no original-data guarantee is
/// inferred from this scalar distance.
///
/// # Errors
/// Returns [`Error::IncompatibleDiagramContext`] for unequal fields or unsupported
/// scale conventions, in addition to errors from [`bottleneck_distance`].
pub fn bottleneck_distance_results<L, R>(first: &L, second: &R, dimension: usize) -> Result<f64>
where
    L: AsRef<PersistenceData> + ?Sized,
    R: AsRef<PersistenceData> + ?Sized,
{
    bottleneck_distance_results_with(first, second, dimension, &Execution::default())
}

/// Context-checked bottleneck distance with one cooperative operation budget.
///
/// Borrows stored common data just like [`bottleneck_distance_results`].
/// # Errors
/// Includes [`bottleneck_distance_results`] errors, [`Error::Cancelled`] and
/// [`Error::WorkLimitExceeded`]. No partial distance is returned.
pub fn bottleneck_distance_results_with<L, R>(
    first: &L,
    second: &R,
    dimension: usize,
    execution: &Execution<'_>,
) -> Result<f64>
where
    L: AsRef<PersistenceData> + ?Sized,
    R: AsRef<PersistenceData> + ?Sized,
{
    distance_results_with(first, second, dimension, Kind::Bottleneck, execution)
}

/// W1-L-infinity distance with the context checks of [`bottleneck_distance_results`].
///
/// # Errors
/// Returns an error for incompatible fields/scales or any [`wasserstein_1_infinity`]
/// input, allocation or numerical failure.
pub fn wasserstein_1_infinity_results<L, R>(first: &L, second: &R, dimension: usize) -> Result<f64>
where
    L: AsRef<PersistenceData> + ?Sized,
    R: AsRef<PersistenceData> + ?Sized,
{
    wasserstein_1_infinity_results_with(first, second, dimension, &Execution::default())
}

/// Context-checked W1 with the controls of [`bottleneck_distance_results_with`].
/// # Errors
/// Includes [`wasserstein_1_infinity_results`] errors, [`Error::Cancelled`] and
/// [`Error::WorkLimitExceeded`]. No partial distance is returned.
pub fn wasserstein_1_infinity_results_with<L, R>(
    first: &L,
    second: &R,
    dimension: usize,
    execution: &Execution<'_>,
) -> Result<f64>
where
    L: AsRef<PersistenceData> + ?Sized,
    R: AsRef<PersistenceData> + ?Sized,
{
    distance_results_with(first, second, dimension, Kind::W1, execution)
}

/// W2-Euclidean distance with the context checks of [`bottleneck_distance_results`].
///
/// # Errors
/// Returns an error for incompatible fields/scales or any [`wasserstein_2_euclidean`]
/// input, allocation or numerical failure.
pub fn wasserstein_2_euclidean_results<L, R>(first: &L, second: &R, dimension: usize) -> Result<f64>
where
    L: AsRef<PersistenceData> + ?Sized,
    R: AsRef<PersistenceData> + ?Sized,
{
    wasserstein_2_euclidean_results_with(first, second, dimension, &Execution::default())
}

/// Context-checked W2 with the controls of [`bottleneck_distance_results_with`].
/// # Errors
/// Includes [`wasserstein_2_euclidean_results`] errors, [`Error::Cancelled`] and
/// [`Error::WorkLimitExceeded`]. No partial distance is returned.
pub fn wasserstein_2_euclidean_results_with<L, R>(
    first: &L,
    second: &R,
    dimension: usize,
    execution: &Execution<'_>,
) -> Result<f64>
where
    L: AsRef<PersistenceData> + ?Sized,
    R: AsRef<PersistenceData> + ?Sized,
{
    distance_results_with(first, second, dimension, Kind::W2, execution)
}

#[derive(Clone, Copy)]
enum Kind {
    Bottleneck,
    W1,
    W2,
}

fn distance_with(
    first: &PersistenceDiagram,
    second: &PersistenceDiagram,
    dimension: usize,
    kind: Kind,
    execution: &Execution<'_>,
) -> Result<f64> {
    if execution.is_unlimited() {
        distance(first, second, dimension, kind, &mut WorkBudget::unlimited())
    } else {
        distance(
            first,
            second,
            dimension,
            kind,
            &mut WorkBudget::new(execution)?,
        )
    }
}

fn distance_results_with<L, R>(
    first: &L,
    second: &R,
    dimension: usize,
    kind: Kind,
    execution: &Execution<'_>,
) -> Result<f64>
where
    L: AsRef<PersistenceData> + ?Sized,
    R: AsRef<PersistenceData> + ?Sized,
{
    if execution.is_unlimited() {
        distance_results(
            first.as_ref(),
            second.as_ref(),
            dimension,
            kind,
            &mut WorkBudget::unlimited(),
        )
    } else {
        // Check cancellation before invoking caller-provided conversion code.
        let mut budget = WorkBudget::new(execution)?;
        distance_results(
            first.as_ref(),
            second.as_ref(),
            dimension,
            kind,
            &mut budget,
        )
    }
}

struct Points {
    finite: Vec<[f64; 2]>,
    essential: Vec<f64>,
}

fn points<const CONTROLLED: bool>(
    diagram: &PersistenceDiagram,
    dimension: usize,
    budget: &mut WorkBudget<'_, CONTROLLED>,
) -> Result<Points> {
    budget.step()?;
    let view = diagram.dimension(dimension)?;
    if let Coverage::Through(through) = diagram.coverage() {
        return Err(Error::IncompleteDiagram { through });
    }
    let mut finite = Vec::new();
    let mut essential = Vec::new();
    for interval in view.iter() {
        budget.step()?;
        match interval.end() {
            IntervalEnd::Finite(death) => {
                finite.try_reserve(1).map_err(|_| Error::AllocationFailed {
                    context: "finite distance points",
                })?;
                finite.push([interval.birth(), death]);
            }
            IntervalEnd::Essential => {
                essential
                    .try_reserve(1)
                    .map_err(|_| Error::AllocationFailed {
                        context: "essential distance points",
                    })?;
                essential.push(interval.birth());
            }
            IntervalEnd::RightCensored { through } => {
                return Err(Error::IncompleteDiagram { through });
            }
        }
    }
    // The owning diagram sorts by dimension and then birth, so this subsequence
    // already has the monotone order required by one-dimensional matching.
    Ok(Points { finite, essential })
}

fn distance<const CONTROLLED: bool>(
    first: &PersistenceDiagram,
    second: &PersistenceDiagram,
    dimension: usize,
    kind: Kind,
    budget: &mut WorkBudget<'_, CONTROLLED>,
) -> Result<f64> {
    let first = points(first, dimension, budget)?;
    let second = points(second, dimension, budget)?;
    budget.check()?;
    if first.essential.len() != second.essential.len() {
        return Ok(f64::INFINITY);
    }
    let finite = match kind {
        Kind::Bottleneck => bottleneck::distance(&first.finite, &second.finite, budget)?,
        Kind::W1 => wasserstein::distance(
            &first.finite,
            &second.finite,
            wasserstein::Metric::W1,
            budget,
        )?,
        Kind::W2 => wasserstein::distance(
            &first.finite,
            &second.finite,
            wasserstein::Metric::W2,
            budget,
        )?,
    };
    let mut value = finite;
    let mut compensation = 0.0;
    for (left, right) in first
        .essential
        .chunks(256)
        .zip(second.essential.chunks(256))
    {
        budget.step_by(left.len())?;
        for (&left, &right) in left.iter().zip(right) {
            let cost = (left - right).abs();
            if !cost.is_finite() {
                return Err(Error::NumericalFailure {
                    context: "essential point distance",
                });
            }
            value = match kind {
                Kind::Bottleneck => value.max(cost),
                Kind::W1 => {
                    let corrected = cost - compensation;
                    let sum = value + corrected;
                    compensation = (sum - value) - corrected;
                    sum
                }
                Kind::W2 => value.hypot(cost),
            };
        }
    }
    budget.check()?;
    if !value.is_finite() {
        return Err(Error::NumericalFailure {
            context: "diagram distance accumulation",
        });
    }
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn distance_results<const CONTROLLED: bool>(
    first: &PersistenceData,
    second: &PersistenceData,
    dimension: usize,
    kind: Kind,
    budget: &mut WorkBudget<'_, CONTROLLED>,
) -> Result<f64> {
    budget.step()?;
    check_context(first, second)?;
    distance(first.diagram(), second.diagram(), dimension, kind, budget)
}

fn check_context(first: &PersistenceData, second: &PersistenceData) -> Result<()> {
    if first.context().characteristic() != second.context().characteristic() {
        return Err(Error::IncompatibleDiagramContext {
            reason: "coefficient field characteristics differ",
        });
    }
    use crate::filtration::FiltrationScale;
    // An unspecified scale is not a shared unit, even for the same source kind.
    // Match the supported convention explicitly so future conventions require
    // a deliberate compatibility policy here.
    if !matches!(
        (
            first.context().filtration().scale(),
            second.context().filtration().scale(),
        ),
        (FiltrationScale::EdgeLength, FiltrationScale::EdgeLength)
    ) {
        return Err(Error::IncompatibleDiagramContext {
            reason: "diagram distance requires declared edge-length parameter conventions",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Exercise an actual kernel/stage rather than stopping at the public facade.
    // Discover work dynamically; no algorithm's incidental count is frozen.
    pub(super) fn check_control<T: PartialEq + Debug>(
        mut run: impl FnMut(&mut WorkBudget<'_>) -> Result<T>,
    ) {
        let mut counted = WorkBudget::new(&Execution::default().max_work(u64::MAX)).unwrap();
        let expected = run(&mut counted).unwrap();
        let work = counted.used();
        assert!(work > 1);
        for limit in [0, work / 4, work / 2, work - 1, work, work + 1] {
            let mut budget = WorkBudget::new(&Execution::default().max_work(limit)).unwrap();
            let result = run(&mut budget);
            if limit < work {
                assert_eq!(result, Err(Error::WorkLimitExceeded { limit }));
            } else {
                assert_eq!(result.unwrap(), expected);
            }
            assert_eq!(
                run(&mut WorkBudget::new(&Execution::default()).unwrap()).unwrap(),
                expected
            );
        }
        for at in [work / 4, work / 2] {
            let flag = AtomicBool::new(false);
            let execution = Execution::new(Some(u64::MAX), Some(&flag));
            let mut budget = WorkBudget::new(&execution).unwrap();
            budget.cancel_at_work(at);
            assert_eq!(run(&mut budget), Err(Error::Cancelled));
            assert!(budget.used() >= at);
            assert!(flag.load(Ordering::Relaxed));
            assert_eq!(
                run(&mut WorkBudget::new(&Execution::default()).unwrap()).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn extraction_and_matching_share_the_operation_budget() {
        let make = |offset| {
            PersistenceDiagram::new(
                0,
                Coverage::Complete,
                vec![
                    crate::diagram::PersistenceInterval::new(
                        0,
                        offset,
                        IntervalEnd::Finite(offset + 3.),
                    )
                    .unwrap(),
                    crate::diagram::PersistenceInterval::new(
                        0,
                        offset + 1.,
                        IntervalEnd::Finite(offset + 4.),
                    )
                    .unwrap(),
                ],
            )
            .unwrap()
        };
        let a = make(0.);
        let b = make(0.25);
        for kind in [Kind::Bottleneck, Kind::W1, Kind::W2] {
            let mut preparation =
                WorkBudget::new(&Execution::default().max_work(u64::MAX)).unwrap();
            let left = points(&a, 0, &mut preparation).unwrap();
            let right = points(&b, 0, &mut preparation).unwrap();
            let mut kernel = WorkBudget::new(&Execution::default().max_work(u64::MAX)).unwrap();
            match kind {
                Kind::Bottleneck => bottleneck::distance(&left.finite, &right.finite, &mut kernel),
                Kind::W1 => wasserstein::distance(
                    &left.finite,
                    &right.finite,
                    wasserstein::Metric::W1,
                    &mut kernel,
                ),
                Kind::W2 => wasserstein::distance(
                    &left.finite,
                    &right.finite,
                    wasserstein::Metric::W2,
                    &mut kernel,
                ),
            }
            .unwrap();
            let mut whole = WorkBudget::new(&Execution::default().max_work(u64::MAX)).unwrap();
            distance(&a, &b, 0, kind, &mut whole).unwrap();
            assert_eq!(whole.used(), preparation.used() + kernel.used());
            let limit = preparation.used().max(kernel.used());
            let mut limited = WorkBudget::new(&Execution::default().max_work(limit)).unwrap();
            assert_eq!(
                distance(&a, &b, 0, kind, &mut limited),
                Err(Error::WorkLimitExceeded { limit })
            );
        }
    }
}
