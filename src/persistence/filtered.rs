//! Boundary reduction driven by the public filtered-cell contract.
//! Only the requested q+1 skeleton is retained; metadata is read for all cells
//! to establish the range. Boundary validity is checked on the retained skeleton.
use super::{
    PersistenceOptions, RepresentativeRequest,
    boundary::{self, BoundaryInput},
};
use crate::algebra::column::Column;
use crate::complex::FilteredComplex;
use crate::diagram::{ComputationContext, PersistenceResult};
use crate::execution::WorkBudget;
use crate::filtration::{Coverage, FiltrationKind};
use crate::{Error, Result};
use std::collections::HashMap;

fn read<C: FilteredComplex>(
    source: &C,
    options: &PersistenceOptions,
    budget: &mut WorkBudget<'_>,
) -> Result<(BoundaryInput<C::CellId>, Coverage, usize)> {
    let field = options.field();
    let mut input = BoundaryInput::new();
    let mut coverage = Coverage::Complete;
    let mut vertex_count = 0_usize;
    let mut seen = HashMap::new();
    let mut selected = HashMap::new();
    let mut previous: Option<f64> = None;
    for (index, cell) in source.cells().enumerate() {
        budget.step()?;
        let value = source.filtration_value(cell);
        let dimension = source.dimension(cell);
        if !value.is_finite() || previous.is_some_and(|p| value < p) {
            return Err(invalid(index, "values must be finite and nondecreasing"));
        }
        previous = Some(value);
        seen.try_reserve(1).map_err(|_| allocation())?;
        if seen.insert(cell, ()).is_some() {
            return Err(invalid(index, "duplicate cell ID"));
        }
        if dimension == 0 {
            vertex_count = vertex_count.checked_add(1).ok_or(Error::SizeOverflow {
                operation: "cell vertices",
            })?;
        }
        if options.max_edge().is_some_and(|t| value > t) {
            coverage = Coverage::Through(options.max_edge().unwrap());
            continue;
        }
        if dimension > options.max_homology_dimension().saturating_add(1) {
            continue;
        }
        let mut column = Column::new();
        for (face, coefficient) in source.boundary(cell) {
            budget.step()?;
            if coefficient == 0 {
                continue;
            }
            let &row = selected.get(&face).ok_or(invalid(
                index,
                "boundary cell missing or not earlier in filtration",
            ))?;
            if dimension.checked_sub(1) != Some(input.dimensions[row]) {
                return Err(invalid(index, "boundary dimension must decrease by one"));
            }
            let coefficient =
                i64::from(coefficient).rem_euclid(i64::from(field.characteristic())) as u32;
            column.add_term(row, coefficient, field);
        }
        // Detect invalid chain inputs in the field actually used for this run.
        let mut twice = Column::new();
        for (&row, &coefficient) in column.entries() {
            twice.add_scaled(&input.columns[row], coefficient, field, &mut || {
                budget.step()
            })?;
        }
        if !twice.is_empty() {
            return Err(invalid(
                index,
                "boundary of boundary is nonzero in the selected field",
            ));
        }
        let position = input.cells.len();
        selected.try_reserve(1).map_err(|_| allocation())?;
        input.push(cell, dimension, crate::canonical_zero(value), column)?;
        selected.insert(cell, position);
    }
    budget.check()?;
    Ok((input, coverage, vertex_count))
}

pub(super) fn compute<C: FilteredComplex>(
    source: &C,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<PersistenceResult> {
    if !requests.is_empty() {
        return Err(Error::InvalidParameter {
            parameter: "representatives",
            reason: "generic cell analysis has no simplex vertex labels; use a simplicial source",
        });
    }
    let (input, coverage, vertex_count) = read(source, options, budget)?;
    let diagram = boundary::diagram(
        input,
        options.max_homology_dimension(),
        options.field(),
        coverage,
        budget,
    )?;
    Ok(PersistenceResult::new(
        diagram,
        ComputationContext::new(
            options.field(),
            FiltrationKind::SuppliedCells,
            vertex_count,
            options.max_edge(),
            None,
            None,
        ),
        None,
    ))
}
fn invalid(index: usize, reason: &'static str) -> Error {
    Error::InvalidComplex {
        cell: Some(index),
        reason,
    }
}
fn allocation() -> Error {
    Error::AllocationFailed {
        context: "filtered boundary input",
    }
}
