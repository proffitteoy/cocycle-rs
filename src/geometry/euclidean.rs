//! Euclidean distance construction in condensed storage order.

use super::PointCloudView;
use super::dissimilarity::pair_count;
use crate::{Error, Result, canonical_zero};

/// Stable Euclidean norms in the same condensed order used by DissimilarityView.
pub(crate) fn euclidean_distances(input: PointCloudView<'_>) -> Result<Vec<f64>> {
    euclidean_distances_with(input, &mut || Ok(()))
}

pub(crate) fn euclidean_distances_with(
    input: PointCloudView<'_>,
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<Vec<f64>> {
    let count = pair_count(input.len()).ok_or(Error::SizeOverflow {
        operation: "point count choose 2",
    })?;
    let mut values = Vec::new();
    values
        .try_reserve(count)
        .map_err(|_| Error::AllocationFailed {
            context: "Euclidean distances",
        })?;
    for i in 0..input.len() {
        for j in 0..i {
            checkpoint()?;
            values.push(euclidean_distance(input, i, j)?);
        }
    }
    Ok(values)
}

/// Pair evaluation shared by dense conversion and streamed threshold construction.
pub(crate) fn euclidean_distance(input: PointCloudView<'_>, i: usize, j: usize) -> Result<f64> {
    let a = input.point(i).ok_or(Error::InternalInvariant {
        reason: "missing point",
    })?;
    let b = input.point(j).ok_or(Error::InternalInvariant {
        reason: "missing point",
    })?;
    let mut norm: f64 = 0.0;
    for (&x, &y) in a.iter().zip(b) {
        norm = norm.hypot(x - y);
    }
    if !norm.is_finite() {
        return Err(Error::NumericalFailure {
            context: "Euclidean distance",
        });
    }
    Ok(canonical_zero(norm))
}
