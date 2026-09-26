//! Forward reduction with normalized pivots and retained column transformations.
use crate::algebra::{PrimeField, column::Column};
use crate::{Error, Result};

pub(crate) struct BoundaryReduction {
    pub(crate) reduced: Vec<Column<usize>>,
    pub(crate) transforms: Vec<Column<usize>>,
    pub(crate) deaths: Vec<Option<usize>>,
}
pub(crate) fn reduce(
    columns: Vec<Column<usize>>,
    field: PrimeField,
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<BoundaryReduction> {
    reduce_impl::<true>(columns, field, checkpoint)
}
/// Reduce without retaining basis transformations for diagram-only computation.
pub(crate) fn reduce_pairs(
    columns: Vec<Column<usize>>,
    field: PrimeField,
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<BoundaryReduction> {
    reduce_impl::<false>(columns, field, checkpoint)
}
fn reduce_impl<const BASES: bool>(
    mut columns: Vec<Column<usize>>,
    field: PrimeField,
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<BoundaryReduction> {
    let mut transforms: Vec<Column<usize>> = Vec::new();
    transforms
        .try_reserve_exact(if BASES { columns.len() } else { 0 })
        .map_err(|_| allocation())?;
    let mut deaths = Vec::new();
    deaths
        .try_reserve_exact(columns.len())
        .map_err(|_| allocation())?;
    deaths.resize(columns.len(), None);
    for j in 0..columns.len() {
        checkpoint()?;
        let mut transform = if BASES {
            Column::unit(j)
        } else {
            Column::new()
        };
        let (earlier, current) = columns.split_at_mut(j);
        let current = &mut current[0];
        while let Some((&pivot, &coefficient)) = current.last() {
            checkpoint()?;
            if pivot >= j {
                return Err(Error::InternalInvariant {
                    reason: "boundary face does not precede coface",
                });
            }
            if let Some(owner) = deaths[pivot] {
                let factor = field.negate(coefficient); // Stored pivots are normalized to one.
                current.add_scaled(&earlier[owner], factor, field, checkpoint)?;
                if BASES {
                    transform.add_scaled(&transforms[owner], factor, field, checkpoint)?;
                }
            } else {
                let inverse = field.inverse(coefficient)?;
                current.scale(inverse, field, checkpoint)?;
                if BASES {
                    transform.scale(inverse, field, checkpoint)?;
                }
                deaths[pivot] = Some(j);
                break;
            }
        }
        if BASES {
            transforms.push(transform);
        }
    }
    Ok(BoundaryReduction {
        reduced: columns,
        transforms,
        deaths,
    })
}
fn allocation() -> Error {
    Error::AllocationFailed {
        context: "representative boundary reduction",
    }
}
