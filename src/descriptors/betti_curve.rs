//! Coverage-aware Betti curves on a validated query grid.

use crate::diagram::{Coverage, IntervalEnd, PersistenceDiagram};
use crate::{Error, Result};

/// Count live intervals at the given finite, strictly increasing scales.
///
/// Births are included and finite deaths excluded. A censored class remains alive
/// at the cutoff itself. Complete diagrams may be queried beyond the largest
/// observed endpoint. An empty grid yields an empty vector after dimension checks.
///
/// Sorts birth/death events, then scans the grid in O(m log m + g) time and O(m + g)
/// space for m selected intervals and g query values. Signed scales are supported.
///
/// # Errors
/// Returns an error for an uncomputed dimension, an invalid grid, a query past
/// censored coverage, or a failed memory reservation. Failure returns no partial curve.
pub fn betti_curve(
    diagram: &PersistenceDiagram,
    dimension: usize,
    grid: &[f64],
) -> Result<Vec<usize>> {
    let intervals = diagram.intervals_in_dimension(dimension)?;
    for (index, &value) in grid.iter().enumerate() {
        if !value.is_finite() {
            return Err(Error::InvalidGrid {
                index,
                reason: "scale must be finite",
            });
        }
        if index > 0 && value <= grid[index - 1] {
            return Err(Error::InvalidGrid {
                index,
                reason: "scales must be strictly increasing",
            });
        }
        if let Coverage::Through(through) = diagram.coverage()
            && value > through
        {
            return Err(Error::QueryOutsideCoverage {
                index,
                value,
                through,
            });
        }
    }
    if grid.is_empty() {
        return Ok(Vec::new());
    }
    let mut births = Vec::new();
    let mut deaths = Vec::new();
    for interval in intervals {
        births.try_reserve(1).map_err(|_| Error::AllocationFailed {
            context: "Betti births",
        })?;
        births.push(interval.birth());
        if let IntervalEnd::Finite(death) = interval.end() {
            deaths.try_reserve(1).map_err(|_| Error::AllocationFailed {
                context: "Betti deaths",
            })?;
            deaths.push(death);
        }
    }
    births.sort_unstable_by(f64::total_cmp);
    deaths.sort_unstable_by(f64::total_cmp);
    let mut result = Vec::new();
    result
        .try_reserve(grid.len())
        .map_err(|_| Error::AllocationFailed {
            context: "Betti curve",
        })?;
    let (mut born, mut dead) = (0, 0);
    for &value in grid {
        while born < births.len() && births[born] <= value {
            born += 1;
        }
        while dead < deaths.len() && deaths[dead] <= value {
            dead += 1;
        }
        result.push(born - dead); // Every counted death has an earlier counted birth.
    }
    Ok(result)
}
