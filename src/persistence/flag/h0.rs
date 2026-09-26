//! Independent H0 by sorted edges and component merges.
use crate::persistence::execution::WorkBudget;
use crate::persistence::union_find::UnionFind;
use crate::{Error, Result};

pub(super) fn compute(
    n: usize,
    edges: impl IntoIterator<Item = (usize, usize, f64)>,
    cutoff: f64,
    budget: &mut WorkBudget<'_>,
) -> Result<Vec<(usize, f64, Option<f64>)>> {
    let mut sorted = Vec::new();
    for (a, b, weight) in edges {
        budget.step()?;
        if weight <= cutoff {
            sorted.try_reserve(1).map_err(|_| Error::AllocationFailed {
                context: "H0 edges",
            })?;
            sorted.push((weight, a, b));
        }
    }
    sorted.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    budget.check()?;
    let mut forest = UnionFind::new(n)?;
    let mut raw = Vec::new();
    raw.try_reserve(n).map_err(|_| Error::AllocationFailed {
        context: "H0 intervals",
    })?;
    for (weight, a, b) in sorted {
        budget.step()?;
        if !forest.merge(a, b) {
            continue;
        }
        raw.push((0, 0.0, Some(weight)));
        if forest.components() == 1 {
            break;
        }
    }
    raw.extend((0..forest.components()).map(|_| (0, 0.0, None)));
    Ok(raw)
}
