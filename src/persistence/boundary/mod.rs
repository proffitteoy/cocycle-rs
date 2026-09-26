//! Ordinary persistence from owned boundary columns in filtration order.
//! This computation does not select an input representation or call default
//! dispatch. Readers own source validation/selection; the algebra reducer owns
//! column elimination. Other algorithms can use different access and workspaces.
mod input;
pub(super) use input::BoundaryInput;

use crate::algebra::{PrimeField, reduction};
use crate::diagram::{Coverage, PersistenceDiagram};
use crate::execution::WorkBudget;
use crate::{Error, Result};

/// Consume validated columns without retaining basis transformations.
/// The caller supplies the requested skeleton and its established coverage.
/// Output declares the contiguous domain 0..=dimension used by current adapters.
pub(super) fn diagram<I>(
    input: BoundaryInput<I>,
    dimension: usize,
    field: PrimeField,
    coverage: Coverage,
    budget: &mut WorkBudget<'_>,
) -> Result<PersistenceDiagram> {
    let reduced = reduction::reduce_pairs(input.columns, field, &mut || budget.step())?;
    let mut intervals = Vec::new();
    for (birth, column) in reduced.reduced.iter().enumerate() {
        budget.step()?;
        if column.is_empty() && input.dimensions[birth] <= dimension {
            intervals
                .try_reserve(1)
                .map_err(|_| Error::AllocationFailed {
                    context: "filtered boundary input",
                })?;
            intervals.push((
                input.dimensions[birth],
                input.values[birth],
                reduced.deaths[birth].map(|d| input.values[d]),
            ));
        }
    }
    super::assemble_diagram(dimension, coverage, intervals)
}

#[cfg(test)]
mod tests;
