//! Shared exact flag engines and supplied-graph computation.
mod cohomology;
mod h0;
mod representatives;
pub use representatives::{RepresentativeRequest, RepresentativeSelection};
mod union_find;
pub(super) use cohomology::dimensions::compute as compute_simplicial;

use super::execution::WorkBudget;
use super::{ExecutionLimits, PersistenceOptions, assemble_diagram};
use crate::Result;
use crate::algebra::PrimeField;
use crate::complex::WeightedGraph;
use crate::diagram::{ComputationContext, Coverage, FiltrationKind, PersistenceResult};
use crate::filtration::{
    FlagFiltration,
    flag::{CliqueAccess, DenseFlag, SparseFlag},
};
use crate::geometry::DissimilarityMatrixView;

type RawIntervals = Vec<(usize, f64, Option<f64>)>;

pub(super) fn compute_dense(
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
            return compute_simplicial(&CliqueAccess::Dense(input, stop), dimension, field, budget);
        }
        let access = DenseFlag::new(input, stop)?;
        budget.check()?;
        cohomology::compute(&access, budget)
    }
}

pub(super) fn compute_graph(
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
            return compute_simplicial(
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

/// Compute ordinary dimension-generic prime-field persistence of a supplied graph's clique filtration.
///
/// Missing edges never enter this filtration. Without a cutoff (or at/above its
/// largest edge), unpaired classes are essential for this supplied graph. A lower
/// cutoff conservatively censors surviving classes. Input is borrowed and never
/// densified. Work/storage include O(n+m) input access plus reduction fill-in.
/// The specialized F2 H1 combinatorial IDs must fit `usize`, even for sparse
/// graphs. Odd-prime and higher-dimensional paths use ordered vertex tuples.
///
/// # Errors
/// Returns size/allocation errors, cancellation, work-limit exhaustion, or an
/// invariant error; never a successful partial result.
pub fn compute_flag(
    input: &FlagFiltration,
    options: &PersistenceOptions,
    limits: &ExecutionLimits<'_>,
) -> Result<PersistenceResult> {
    compute_flag_with_representatives(input, options, &[], limits)
}

/// Compute supplied-graph persistence and requested basis representatives.
///
/// Unlike diagram-only computation, nonempty requests materialize the skeleton
/// through q+1 and retain boundary transformations. Cost can be exponential.
/// The execution budget includes this preparation and representative extraction.
///
/// # Errors
/// Includes [`compute_flag`] errors, uncomputed dimensions and query scales
/// outside coverage. No partial diagram or representative payload is returned.
pub fn compute_flag_with_representatives(
    input: &FlagFiltration,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    limits: &ExecutionLimits<'_>,
) -> Result<PersistenceResult> {
    let mut budget = WorkBudget::new(limits)?;
    compute_flag_budget(input, options, requests, &mut budget)
}

pub(in crate::persistence) fn compute_flag_budget(
    input: &FlagFiltration,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<PersistenceResult> {
    let graph = input.graph();
    let (cutoff, coverage) = match options.max_edge() {
        Some(t) if t < graph.max_edge() => (t, Coverage::Through(t)),
        _ => (graph.max_edge(), Coverage::Complete),
    };
    let (diagram, representatives) = finish(
        &CliqueAccess::Sparse(graph, cutoff),
        options,
        requests,
        coverage,
        budget,
        |budget| {
            compute_graph(
                graph,
                options.max_homology_dimension(),
                cutoff,
                options.field(),
                budget,
            )
        },
    )?;
    Ok(PersistenceResult {
        diagram,
        representatives,
        context: ComputationContext {
            approximation: None,
            field: options.field(),
            kind: FiltrationKind::SuppliedFlag,
            vertex_count: graph.vertex_count(),
            requested_cutoff: options.max_edge(),
            construction_cutoff: None,
        },
    })
}

/// Shared dispatch for optional representatives; ordinary calls keep their implicit engine.
pub(super) fn finish(
    access: &impl crate::filtration::flag::SimplicialAccess,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    coverage: Coverage,
    budget: &mut WorkBudget<'_>,
    implicit: impl FnOnce(&mut WorkBudget<'_>) -> Result<RawIntervals>,
) -> Result<(
    crate::diagram::PersistenceDiagram,
    Option<Vec<crate::diagram::Representative>>,
)> {
    let result = if requests.is_empty() {
        (
            assemble_diagram(
                options.max_homology_dimension(),
                coverage,
                implicit(budget)?,
            )?,
            None,
        )
    } else {
        let (diagram, representatives) =
            representatives::compute(access, options, requests, coverage, budget)?;
        (diagram, Some(representatives))
    };
    budget.check()?;
    Ok(result)
}
