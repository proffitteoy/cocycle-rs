//! Sparse constraint elimination for a basis dual to persistent homology cycles.
use crate::algebra::{PrimeField, column::Column};
use crate::persistence::execution::WorkBudget;
use crate::{Error, Result};
use std::collections::BTreeMap;

// Each equation is phi(chain) = rhs. A unit right-hand side selects one active
// interval; boundaries have zero rhs. Free chain coordinates are set to zero.
struct Equation {
    chain: Column<usize>,
    rhs: Column<usize>,
}
pub(super) fn cocycles<'a>(
    boundaries: impl Iterator<Item = &'a Column<usize>>,
    cycles: &[&Column<usize>],
    field: PrimeField,
    budget: &mut WorkBudget<'_>,
) -> Result<Vec<Column<usize>>> {
    let mut equations: BTreeMap<usize, Equation> = BTreeMap::new();
    for (chain, rhs) in boundaries.map(|b| (b.clone(), Column::new())).chain(
        cycles
            .iter()
            .enumerate()
            .map(|(i, c)| ((*c).clone(), Column::unit(i))),
    ) {
        budget.step()?;
        let mut equation = Equation { chain, rhs };
        loop {
            budget.step()?;
            let Some((&pivot, &coefficient)) = equation.chain.first() else {
                if !equation.rhs.is_empty() {
                    return Err(Error::InternalInvariant {
                        reason: "representative cycles are dependent modulo boundaries",
                    });
                }
                break;
            };
            if let Some(owner) = equations.get(&pivot) {
                let factor = field.negate(coefficient);
                equation
                    .chain
                    .add_scaled(&owner.chain, factor, field, &mut || budget.step())?;
                equation
                    .rhs
                    .add_scaled(&owner.rhs, factor, field, &mut || budget.step())?;
            } else {
                let inverse = field.inverse(coefficient)?;
                equation
                    .chain
                    .scale(inverse, field, &mut || budget.step())?;
                equation.rhs.scale(inverse, field, &mut || budget.step())?;
                equations.insert(pivot, equation);
                break;
            }
        }
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(cycles.len())
        .map_err(|_| Error::AllocationFailed {
            context: "dual representative basis",
        })?;
    for selected in 0..cycles.len() {
        let mut cocycle = Column::new();
        for (&pivot, equation) in equations.iter().rev() {
            budget.step()?;
            let mut value = equation.rhs.get(&selected);
            for (&variable, &coefficient) in equation.chain.entries().skip(1) {
                budget.step()?;
                value = field.subtract(value, field.multiply(coefficient, cocycle.get(&variable)));
            }
            cocycle.add_term(pivot, value, field);
        }
        result.push(cocycle);
    }
    Ok(result)
}
