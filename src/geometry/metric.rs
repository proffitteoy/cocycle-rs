//! Explicit metric hypotheses for approximation algorithms.
use super::DissimilarityMatrixView;
use crate::{Error, Result};

/// How a caller establishes the triangle inequality on stored distances.
/// Zero distance between distinct vertices is allowed (a pseudometric).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricPolicy {
    /// Exhaustively check every triangle in O(n³) time, without a tolerance.
    Check,
    /// Caller asserts the triangle inequality; no check is performed.
    Assume,
    /// Construct without a metric hypothesis; no approximation bound is reported.
    Unchecked,
}

/// Evidence for the metric hypothesis, independent of an approximation theorem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricValidation {
    /// All triangle inequalities of the stored binary64 numbers were checked.
    Checked,
    /// The caller supplied the hypothesis.
    Assumed,
    /// No metric hypothesis was established.
    Unchecked,
}

/// Check or explicitly assume metric structure on validated dissimilarities.
///
/// `Check` treats each finite binary64 input as an exact real number. It checks
/// all three inequalities per triple, including equality, without tolerances.
/// It does not certify the original coordinates or later floating arithmetic.
///
/// # Errors
/// Returns the first violating triple when checking is requested.
pub fn validate_metric(
    input: DissimilarityMatrixView<'_>,
    policy: MetricPolicy,
) -> Result<MetricValidation> {
    validate_metric_with(input.len(), policy, |a, b| Ok(input.get(a, b).unwrap()))
}

pub(crate) fn validate_metric_with(
    n: usize,
    policy: MetricPolicy,
    mut distance: impl FnMut(usize, usize) -> Result<f64>,
) -> Result<MetricValidation> {
    validate_metric_with_checkpoints(n, policy, &mut distance, &mut || Ok(()))
}

pub(crate) fn validate_metric_with_checkpoints(
    n: usize,
    policy: MetricPolicy,
    mut distance: impl FnMut(usize, usize) -> Result<f64>,
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<MetricValidation> {
    match policy {
        MetricPolicy::Assume => return Ok(MetricValidation::Assumed),
        MetricPolicy::Unchecked => return Ok(MetricValidation::Unchecked),
        MetricPolicy::Check => {}
    }
    for c in 0..n {
        for b in 0..c {
            for a in 0..b {
                let x = distance(a, b)?;
                let y = distance(a, c)?;
                let z = distance(b, c)?;
                for (long, first, second) in [(x, y, z), (y, x, z), (z, x, y)] {
                    checkpoint()?;
                    if exceeds_sum(long, first, second) {
                        return Err(Error::InvalidMetric {
                            vertices: [a, b, c],
                        });
                    }
                }
            }
        }
    }
    Ok(MetricValidation::Checked)
}

fn exceeds_sum(c: f64, a: f64, b: f64) -> bool {
    // FastTwoSum recovers the rounding error for nonnegative finite inputs.
    // If the sum overflows, every finite c is below the exact sum.
    let hi = a.max(b);
    let lo = a.min(b);
    let sum = hi + lo;
    if !sum.is_finite() {
        return false;
    }
    let error = lo - (sum - hi);
    c > sum || (c == sum && error < 0.0)
}
