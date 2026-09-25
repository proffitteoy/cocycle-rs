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
//! functions additionally check computation contexts. Distances between sparse
//! approximations describe the two supplied diagrams, not the unknown original
//! diagrams and not an approximation error certificate.
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

use crate::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceResult};
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
    distance(first, second, dimension, Kind::Bottleneck)
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
    distance(first, second, dimension, Kind::W1)
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
    distance(first, second, dimension, Kind::W2)
}

/// Bottleneck distance with additional computation-context validation.
///
/// Requires equal coefficient characteristics. All current computation contexts
/// use edge-length scales. Vertex counts, construction kinds and cutoff requests
/// need not agree when both diagrams have complete coverage. Approximation
/// provenance remains in the borrowed results; no original-data guarantee is
/// inferred from this scalar distance.
///
/// # Errors
/// Returns [`Error::IncompatibleDiagramContext`] for unequal fields, in addition
/// to errors from [`bottleneck_distance`].
pub fn bottleneck_distance_results(
    first: &PersistenceResult,
    second: &PersistenceResult,
    dimension: usize,
) -> Result<f64> {
    check_context(first, second)?;
    bottleneck_distance(first.diagram(), second.diagram(), dimension)
}

/// W1-L-infinity distance with the context checks of [`bottleneck_distance_results`].
///
/// # Errors
/// Returns an error for incompatible fields or any [`wasserstein_1_infinity`]
/// input, allocation or numerical failure.
pub fn wasserstein_1_infinity_results(
    first: &PersistenceResult,
    second: &PersistenceResult,
    dimension: usize,
) -> Result<f64> {
    check_context(first, second)?;
    wasserstein_1_infinity(first.diagram(), second.diagram(), dimension)
}

/// W2-Euclidean distance with the context checks of [`bottleneck_distance_results`].
///
/// # Errors
/// Returns an error for incompatible fields or any [`wasserstein_2_euclidean`]
/// input, allocation or numerical failure.
pub fn wasserstein_2_euclidean_results(
    first: &PersistenceResult,
    second: &PersistenceResult,
    dimension: usize,
) -> Result<f64> {
    check_context(first, second)?;
    wasserstein_2_euclidean(first.diagram(), second.diagram(), dimension)
}

#[derive(Clone, Copy)]
enum Kind {
    Bottleneck,
    W1,
    W2,
}

struct Points {
    finite: Vec<[f64; 2]>,
    essential: Vec<f64>,
}

fn points(diagram: &PersistenceDiagram, dimension: usize) -> Result<Points> {
    let intervals = diagram.intervals_in_dimension(dimension)?;
    if let Coverage::Through(through) = diagram.coverage() {
        return Err(Error::IncompleteDiagram { through });
    }
    let mut finite = Vec::new();
    let mut essential = Vec::new();
    for interval in intervals {
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

fn distance(
    first: &PersistenceDiagram,
    second: &PersistenceDiagram,
    dimension: usize,
    kind: Kind,
) -> Result<f64> {
    let first = points(first, dimension)?;
    let second = points(second, dimension)?;
    if first.essential.len() != second.essential.len() {
        return Ok(f64::INFINITY);
    }
    let finite = match kind {
        Kind::Bottleneck => bottleneck::distance(&first.finite, &second.finite)?,
        Kind::W1 => wasserstein::distance(&first.finite, &second.finite, wasserstein::Metric::W1)?,
        Kind::W2 => wasserstein::distance(&first.finite, &second.finite, wasserstein::Metric::W2)?,
    };
    let mut value = finite;
    let mut compensation = 0.0;
    for (&left, &right) in first.essential.iter().zip(&second.essential) {
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
    if !value.is_finite() {
        return Err(Error::NumericalFailure {
            context: "diagram distance accumulation",
        });
    }
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn check_context(first: &PersistenceResult, second: &PersistenceResult) -> Result<()> {
    if first.context().characteristic() != second.context().characteristic() {
        return Err(Error::IncompatibleDiagramContext {
            reason: "coefficient field characteristics differ",
        });
    }
    // ComputationContext currently guarantees edge-length units for every kind.
    // Do not compare whole contexts: differing datasets and approximation
    // parameters are valid inputs to a distance between their actual diagrams.
    Ok(())
}
