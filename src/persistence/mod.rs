//! Persistent homology computations and owned result assembly.
//!
//! The entry points compute ordinary Rips, flag and supplied filtered-complex persistence over prime fields. Algorithm
//! configuration and working state stay in private, operation-specific modules.

use crate::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use crate::{Error, Result};

mod execution;
mod flag;
mod options;
#[cfg(test)]
mod reference;
mod rips;
pub use execution::ExecutionLimits;
pub use flag::{compute_flag, compute_flag_with_representatives};
pub use options::PersistenceOptions;
pub use rips::{
    compute_expanded_rips, compute_expanded_rips_with_representatives,
    compute_expanded_sparse_rips, compute_expanded_sparse_rips_with_representatives,
    compute_rips_from_distances, compute_rips_from_distances_with_representatives,
    compute_rips_from_points, compute_rips_from_points_with_representatives, compute_sparse_rips,
    compute_sparse_rips_with_representatives, compute_threshold_rips,
    compute_threshold_rips_with_representatives,
};

pub use rips::{RipsOptions, rips_from_dissimilarities, rips_from_points};

/// Shared by algorithms; None means unpaired in the computed range, not necessarily essential.
fn assemble_diagram(
    max_dimension: usize,
    coverage: Coverage,
    raw: impl IntoIterator<Item = (usize, f64, Option<f64>)>,
) -> Result<PersistenceDiagram> {
    let mut intervals = Vec::new();
    for (dimension, birth, death) in raw {
        if dimension > max_dimension || death == Some(birth) {
            continue;
        }
        let end = match death {
            Some(value) => IntervalEnd::Finite(value),
            None => match coverage {
                Coverage::Complete => IntervalEnd::Essential,
                Coverage::Through(through) => IntervalEnd::RightCensored { through },
            },
        };
        let interval = PersistenceInterval::new(dimension, birth, end)?;
        intervals
            .try_reserve(1)
            .map_err(|_| Error::AllocationFailed {
                context: "persistence diagram",
            })?;
        intervals.push(interval);
    }
    PersistenceDiagram::new(max_dimension, coverage, intervals)
}

#[cfg(test)]
mod tests;

mod builder;
mod source;
pub use builder::PersistenceBuilder;
pub use source::PersistenceExt;

mod simplicial;
pub use simplicial::representatives::{RepresentativeRequest, RepresentativeSelection};
mod boundary;
mod filtered;

mod union_find;
type RawIntervals = Vec<(usize, f64, Option<f64>)>;
