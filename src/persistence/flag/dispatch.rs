//! Default selection for exact flag inputs; algorithms do not call this module.
use super::{cohomology, h0};
use crate::Result;
use crate::algebra::PrimeField;
use crate::complex::WeightedGraph;
use crate::execution::WorkBudget;
use crate::filtration::flag::{CliqueAccess, DenseFlag, SparseFlag};
use crate::geometry::DissimilarityMatrixView;
use crate::persistence::{RawIntervals, simplicial};

pub(in crate::persistence) fn compute_dense(
    input: DissimilarityMatrixView<'_>,
    dimension: usize,
    cutoff: f64,
    field: PrimeField,
    budget: &mut WorkBudget<'_>,
) -> Result<RawIntervals> {
    budget.check()?;
    if dimension == 0 {
        h0::compute(
            input.len(),
            (0..input.len()).flat_map(|b| (0..b).map(move |a| (a, b, input.get(a, b).unwrap()))),
            cutoff,
            budget,
        )
    } else {
        let stop = cutoff.min(crate::filtration::rips::cone_radius(input, &mut || {
            budget.step()
        })?);
        if dimension > 1 || field.characteristic() != 2 {
            return simplicial::cohomology::compute(
                &CliqueAccess::Dense(input, stop),
                dimension,
                field,
                budget,
            );
        }
        let access = DenseFlag::new(input, stop)?;
        budget.check()?;
        cohomology::compute(&access, budget)
    }
}

pub(in crate::persistence) fn compute_graph(
    graph: &WeightedGraph,
    dimension: usize,
    cutoff: f64,
    field: PrimeField,
    budget: &mut WorkBudget<'_>,
) -> Result<RawIntervals> {
    budget.check()?;
    if dimension == 0 {
        h0::compute(
            graph.vertex_count(),
            graph
                .edges()
                .iter()
                .map(|e| (e.vertices[0], e.vertices[1], e.value)),
            cutoff,
            budget,
        )
    } else {
        if dimension > 1 || field.characteristic() != 2 {
            return simplicial::cohomology::compute(
                &CliqueAccess::Sparse(graph, cutoff),
                dimension,
                field,
                budget,
            );
        }
        let access = SparseFlag::new(graph, cutoff)?;
        budget.check()?;
        cohomology::compute(&access, budget)
    }
}
