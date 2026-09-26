//! Owned interval multisets and their computation coverage.

use std::cmp::Ordering;

use super::interval::{IntervalEnd, PersistenceInterval, finite_scale};
use crate::{Error, Result};

use super::ComputedDimensions;
use crate::filtration::Coverage;

/// An owned persistence diagram with explicitly recorded dimensions and coverage.
///
/// Computed dimensions are explicitly declared, even if their interval lists are
/// empty. [`Self::new`] declares `0..=max_dimension`; [`Self::with_dimensions`]
/// supports noncontiguous sets. The multiset is sorted deterministically; multiplicities are
/// preserved. No original input, simplex IDs or matrix indices are retained.
///
/// ```
/// use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
///
/// let intervals = vec![PersistenceInterval::new(1, 1.0, IntervalEnd::Finite(2.0))?];
/// let diagram = PersistenceDiagram::new(1, Coverage::Complete, intervals)?;
/// assert_eq!(diagram.intervals_in_dimension(0)?.count(), 0);
/// assert_eq!(diagram.intervals_in_dimension(1)?.count(), 1);
/// assert!(diagram.intervals_in_dimension(2).is_err());
/// # Ok::<(), cocycle::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceDiagram {
    dimensions: ComputedDimensions,
    coverage: Coverage,
    intervals: Vec<PersistenceInterval>,
}

impl PersistenceDiagram {
    /// Take ownership of intervals and validate them against the declared range.
    ///
    /// Complete diagrams may contain finite and essential intervals. For a
    /// truncated diagram, all births and finite deaths must be at or before its
    /// cutoff, all censoring cutoffs must equal it, and essential intervals are
    /// rejected. This conservative convention does not infer eventual survival.
    ///
    /// Sorting is by dimension, birth, endpoint kind (finite, essential, censored),
    /// and then endpoint value. Equal intervals are retained. Validation and
    /// sorting take O(m log m) time for m intervals and do not copy the vector.
    ///
    /// # Errors
    /// Returns an error for a non-finite cutoff, an interval outside the computed
    /// dimensions, or an interval inconsistent with coverage. Errors referring to
    /// an interval index refer to its position before sorting.
    pub fn new(
        max_dimension: usize,
        coverage: Coverage,
        intervals: Vec<PersistenceInterval>,
    ) -> Result<Self> {
        Self::with_dimensions(
            ComputedDimensions::through(max_dimension),
            coverage,
            intervals,
        )
    }

    /// Own intervals with an explicit, possibly noncontiguous computation domain.
    ///
    /// Applies the coverage and sorting rules of [`Self::new`]. The declared
    /// dimensions describe computation, not just dimensions with nonempty bars.
    ///
    /// ```
    /// use cocycle::diagram::{ComputedDimensions, Coverage, PersistenceDiagram};
    /// let diagram = PersistenceDiagram::with_dimensions(
    ///     ComputedDimensions::new(vec![1, 3])?, Coverage::Complete, vec![],
    /// )?;
    /// assert_eq!(diagram.intervals_in_dimension(3)?.count(), 0);
    /// assert!(diagram.intervals_in_dimension(2).is_err());
    /// # Ok::<(), cocycle::Error>(())
    /// ```
    ///
    /// # Errors
    /// Rejects intervals outside `dimensions` and inconsistent or invalid coverage.
    pub fn with_dimensions(
        dimensions: ComputedDimensions,
        coverage: Coverage,
        mut intervals: Vec<PersistenceInterval>,
    ) -> Result<Self> {
        let coverage = match coverage {
            Coverage::Complete => Coverage::Complete,
            Coverage::Through(value) => Coverage::Through(finite_scale(value, "coverage")?),
        };
        for (index, interval) in intervals.iter().enumerate() {
            if !dimensions.contains(interval.dimension()) {
                return Err(Error::DimensionNotComputed {
                    requested: interval.dimension(),
                    computed_max: dimensions.max(),
                });
            }
            validate_coverage(interval, coverage, index)?;
        }
        intervals.sort_unstable_by(compare_intervals);
        Ok(Self {
            dimensions,
            coverage,
            intervals,
        })
    }

    /// Largest computed dimension. This does not imply lower dimensions were computed.
    pub fn max_dimension(&self) -> usize {
        self.dimensions.max()
    }

    /// Exact computation domain, including computed dimensions with no intervals.
    pub fn computed_dimensions(&self) -> &ComputedDimensions {
        &self.dimensions
    }

    /// Complete or inclusive finite computation range.
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }

    /// All intervals, in deterministic order and with multiplicities preserved.
    pub fn intervals(&self) -> &[PersistenceInterval] {
        &self.intervals
    }

    /// Iterate over intervals in a computed dimension, which may have no intervals.
    ///
    /// # Errors
    /// Returns [`Error::DimensionNotComputed`] for a dimension absent from the
    /// declared set, including gaps below the maximum. Empty does not mean uncomputed.
    pub fn intervals_in_dimension(
        &self,
        dimension: usize,
    ) -> Result<impl Iterator<Item = &PersistenceInterval>> {
        if !self.dimensions.contains(dimension) {
            return Err(Error::DimensionNotComputed {
                requested: dimension,
                computed_max: self.dimensions.max(),
            });
        }
        let start = self
            .intervals
            .partition_point(|interval| interval.dimension() < dimension);
        let end = self
            .intervals
            .partition_point(|interval| interval.dimension() <= dimension);
        Ok(self.intervals[start..end].iter())
    }
}

fn validate_coverage(
    interval: &PersistenceInterval,
    coverage: Coverage,
    index: usize,
) -> Result<()> {
    let reason = match coverage {
        Coverage::Complete => match interval.end() {
            IntervalEnd::RightCensored { .. } => {
                Some("complete coverage cannot contain censored intervals")
            }
            _ => None,
        },
        Coverage::Through(cutoff) => {
            if interval.birth() > cutoff {
                Some("birth exceeds the computation cutoff")
            } else {
                match interval.end() {
                    IntervalEnd::Essential => {
                        Some("truncated coverage cannot certify an essential interval")
                    }
                    IntervalEnd::Finite(death) if death > cutoff => {
                        Some("death exceeds the computation cutoff")
                    }
                    IntervalEnd::RightCensored { through } if through != cutoff => {
                        Some("interval and diagram censoring cutoffs differ")
                    }
                    _ => None,
                }
            }
        }
    };
    match reason {
        Some(reason) => Err(Error::InconsistentDiagram {
            interval: index,
            reason,
        }),
        None => Ok(()),
    }
}

fn compare_intervals(a: &PersistenceInterval, b: &PersistenceInterval) -> Ordering {
    a.dimension()
        .cmp(&b.dimension())
        .then_with(|| a.birth().total_cmp(&b.birth()))
        .then_with(|| compare_ends(a.end(), b.end()))
}

fn compare_ends(a: IntervalEnd, b: IntervalEnd) -> Ordering {
    match (a, b) {
        (IntervalEnd::Finite(a), IntervalEnd::Finite(b)) => a.total_cmp(&b),
        (IntervalEnd::Essential, IntervalEnd::Essential) => Ordering::Equal,
        (IntervalEnd::RightCensored { through: a }, IntervalEnd::RightCensored { through: b }) => {
            a.total_cmp(&b)
        }
        (IntervalEnd::Finite(_), _)
        | (IntervalEnd::Essential, IntervalEnd::RightCensored { .. }) => Ordering::Less,
        _ => Ordering::Greater,
    }
}
