//! Boundary columns from immutable, already validated simplicial storage.
//! This concrete adapter relies on face closure, filtration order and oriented
//! incidence established by construction. External filtered cells use their own
//! validating reader; no caller-controlled trust flag crosses that boundary.
use crate::algebra::column::Column;
use crate::complex::{SimplexId, SimplicialComplex};
use crate::execution::WorkBudget;
use crate::persistence::{PersistenceOptions, boundary::BoundaryInput};
use crate::{Error, Result};
use std::collections::HashMap;

pub(super) fn read(
    source: &SimplicialComplex,
    options: &PersistenceOptions,
    budget: &mut WorkBudget<'_>,
) -> Result<BoundaryInput<SimplexId>> {
    let field = options.field();
    let mut input = BoundaryInput::new();
    // Only selected IDs need a reduction position. In particular, a low-degree
    // query must not allocate an ID table the size of the entire stored complex.
    let mut selected = HashMap::new();
    for (index, simplex) in source.simplices().iter().enumerate() {
        budget.step()?;
        if options.max_edge().is_some_and(|t| simplex.value() > t) {
            break;
        }
        if simplex.dimension() > options.max_homology_dimension().saturating_add(1) {
            continue;
        }
        let id = SimplexId(index);
        let mut column = Column::new();
        for term in source.boundary(id).unwrap() {
            budget.step()?;
            // Face closure and monotonicity keep every face in this selection.
            let row = selected[&term.face];
            let coefficient = if term.coefficient > 0 {
                1
            } else {
                field.characteristic() - 1
            };
            column.add_term(row, coefficient, field);
        }
        let position = input.cells.len();
        selected.try_reserve(1).map_err(|_| allocation())?;
        input.push(id, simplex.dimension(), simplex.value(), column)?;
        selected.insert(id, position);
    }
    budget.check()?;
    Ok(input)
}

fn allocation() -> Error {
    Error::AllocationFailed {
        context: "simplicial boundary input",
    }
}
