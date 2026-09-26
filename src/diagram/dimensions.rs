//! Declared computation domains, independent of nonempty interval lists.
use crate::{Error, Result};

/// A nonempty set of homology dimensions recorded as computed.
///
/// Membership does not depend on whether a dimension contains intervals. Values
/// iterate in increasing order without duplicates. Storage is private; callers
/// should not infer membership from the maximum alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComputedDimensions {
    first: (usize, usize),
    rest: Vec<(usize, usize)>,
}

impl ComputedDimensions {
    /// Record every dimension from zero through `maximum`, without enumerating it.
    pub fn through(maximum: usize) -> Self {
        Self {
            first: (0, maximum),
            rest: Vec::new(),
        }
    }

    /// Own and normalize an explicit list of computed dimensions.
    ///
    /// Input order and duplicates have no effect. Sorting takes O(k log k) time
    /// for k supplied entries; consecutive values share internal storage.
    /// # Errors
    /// Rejects an empty list or failed memory reservation.
    pub fn new(mut dimensions: Vec<usize>) -> Result<Self> {
        dimensions.sort_unstable();
        let Some(&start) = dimensions.first() else {
            return Err(Error::InvalidParameter {
                parameter: "computed dimensions",
                reason: "at least one computed dimension is required",
            });
        };
        let mut result = Self {
            first: (start, start),
            rest: Vec::new(),
        };
        for dimension in dimensions.into_iter().skip(1) {
            let last = result.rest.last_mut().unwrap_or(&mut result.first);
            if dimension <= last.1 || last.1.checked_add(1) == Some(dimension) {
                last.1 = dimension;
            } else {
                result
                    .rest
                    .try_reserve(1)
                    .map_err(|_| Error::AllocationFailed {
                        context: "computed dimension ranges",
                    })?;
                result.rest.push((dimension, dimension));
            }
        }
        Ok(result)
    }

    /// Whether this dimension is recorded as computed, even with no intervals.
    pub fn contains(&self, dimension: usize) -> bool {
        if dimension <= self.first.1 {
            return dimension >= self.first.0;
        }
        let position = self.rest.partition_point(|&(_, end)| end < dimension);
        self.rest
            .get(position)
            .is_some_and(|&(start, _)| start <= dimension)
    }

    /// Iterate over exactly the computed dimensions, in increasing order.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = usize> + '_ {
        std::iter::once(self.first)
            .chain(self.rest.iter().copied())
            .flat_map(|(start, end)| start..=end)
    }

    /// Greatest computed dimension; lower dimensions can be absent.
    pub fn max(&self) -> usize {
        self.rest.last().unwrap_or(&self.first).1
    }
}
