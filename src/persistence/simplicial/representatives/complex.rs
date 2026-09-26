//! Materialize only the requested skeleton for the opt-in representative path.
use crate::algebra::{
    PrimeField,
    column::Column,
    reduction::{self, BoundaryReduction},
};
use crate::complex::Simplex;
use crate::filtration::simplicial::{ZeroBornSimplicialAccess, next_dimension};
use crate::persistence::execution::WorkBudget;
use crate::{Error, Result};
use std::collections::HashMap;

pub(super) fn reduce(
    access: &impl ZeroBornSimplicialAccess,
    max_dimension: usize,
    field: PrimeField,
    budget: &mut WorkBudget<'_>,
) -> Result<(Vec<Simplex>, BoundaryReduction)> {
    let mut level = access.vertices()?;
    let mut simplices = Vec::new();
    let top = max_dimension
        .saturating_add(1)
        .min(access.vertex_count().saturating_sub(1));
    for dimension in 0..=top {
        budget.step()?;
        simplices
            .try_reserve(level.len())
            .map_err(|_| allocation())?;
        simplices.extend(level.iter().cloned());
        if dimension == top || level.is_empty() {
            break;
        }
        level = next_dimension(access, &level, &mut || budget.step())?;
    }
    simplices.sort_unstable();
    budget.check()?;
    let mut positions: HashMap<Vec<usize>, usize> = HashMap::new();
    positions
        .try_reserve(simplices.len())
        .map_err(|_| allocation())?;
    let mut boundaries = Vec::new();
    boundaries
        .try_reserve_exact(simplices.len())
        .map_err(|_| allocation())?;
    for (position, simplex) in simplices.iter().enumerate() {
        budget.step()?;
        let mut boundary = Column::new();
        if simplex.dimension() > 0 {
            for omitted in 0..simplex.vertices.len() {
                budget.step()?;
                let mut face = simplex.vertices.clone();
                face.remove(omitted);
                let &row = positions.get(&face).ok_or(Error::InternalInvariant {
                    reason: "representative skeleton is not face closed",
                })?;
                boundary.add_term(row, field.orientation(omitted), field);
            }
        }
        boundaries.push(boundary);
        positions.insert(simplex.vertices.clone(), position);
    }
    let reduction = reduction::reduce(boundaries, field, &mut || budget.step())?;
    Ok((simplices, reduction))
}
fn allocation() -> Error {
    Error::AllocationFailed {
        context: "representative skeleton",
    }
}
