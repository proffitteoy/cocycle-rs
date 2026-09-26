//! Prime-field persistence on the blocker-aware sparse Rips filtration.
use crate::diagram::{
    ComputationContext, Coverage, FiltrationKind, PersistenceResult, RipsApproximation,
};
use crate::filtration::{
    RipsInputKind, SparseRips, SparseRipsExpansion,
    rips::approximation::SparseRipsAccess,
    simplicial::{ZeroBornExplicitAccess, ZeroBornSimplicialAccess},
};
use crate::persistence::{
    ExecutionLimits, PersistenceOptions, RepresentativeRequest, execution::WorkBudget, simplicial,
};
use crate::{Error, Result};

/// Compute sparse Rips persistence through any homology dimension over a prime field.
///
/// Uses the insertion-radius blocker for every coface; no ordinary-flag H1 or
/// dense-cone shortcut is applied. Coverage refers to the approximate filtration.
/// # Errors
/// Rejects unavailable scale ranges, allocation failures or exhausted limits.
pub fn compute_sparse_rips(
    input: &SparseRips,
    options: &PersistenceOptions,
    limits: &ExecutionLimits<'_>,
) -> Result<PersistenceResult> {
    compute_sparse_rips_with_representatives(input, options, &[], limits)
}

/// Compute sparse Rips persistence and requested cycle/cocycle bases.
///
/// Representative terms use original vertex IDs and belong to the approximate
/// filtration. They are not asserted to be representatives of exact Rips classes.
/// # Errors
/// Includes computation errors and representative dimension/coverage errors.
pub fn compute_sparse_rips_with_representatives(
    input: &SparseRips,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    limits: &ExecutionLimits<'_>,
) -> Result<PersistenceResult> {
    let mut budget = WorkBudget::new(limits)?;
    compute_sparse_rips_budget(input, options, requests, &mut budget)
}

pub(in crate::persistence) fn compute_sparse_rips_budget(
    input: &SparseRips,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<PersistenceResult> {
    let (cutoff, coverage) = crate::persistence::options::source_range(
        input.coverage,
        input.graph.max_edge(),
        options.max_edge(),
    )?;
    let access = SparseRipsAccess { input, cutoff };
    compute(
        &access,
        &input.metadata,
        input.kind,
        coverage,
        options,
        requests,
        budget,
    )
}

/// Compute persistence using a frozen sparse Rips skeleton's stored incidence.
/// # Errors
/// Rejects insufficient construction dimension or scale, and exhausted limits.
pub fn compute_expanded_sparse_rips(
    input: &SparseRipsExpansion,
    options: &PersistenceOptions,
    limits: &ExecutionLimits<'_>,
) -> Result<PersistenceResult> {
    compute_expanded_sparse_rips_with_representatives(input, options, &[], limits)
}

/// Compute persistence and representatives from frozen sparse Rips incidence.
/// # Errors
/// Includes insufficient skeleton/scale, execution and representative errors.
pub fn compute_expanded_sparse_rips_with_representatives(
    input: &SparseRipsExpansion,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    limits: &ExecutionLimits<'_>,
) -> Result<PersistenceResult> {
    let mut budget = WorkBudget::new(limits)?;
    compute_expanded_sparse_rips_budget(input, options, requests, &mut budget)
}

pub(in crate::persistence) fn compute_expanded_sparse_rips_budget(
    input: &SparseRipsExpansion,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<PersistenceResult> {
    if !input.complete && options.max_homology_dimension() >= input.max_simplex_dimension {
        return Err(Error::InsufficientSkeleton {
            requested_homology_dimension: options.max_homology_dimension(),
            constructed_simplex_dimension: input.max_simplex_dimension,
        });
    }
    let (cutoff, coverage) = crate::persistence::options::source_range(
        input.coverage,
        input.max_edge,
        options.max_edge(),
    )?;
    let access = ZeroBornExplicitAccess {
        complex: &input.complex,
        vertex_count: input.metadata.retained_vertices().len(),
        cutoff,
    };
    compute(
        &access,
        &input.metadata,
        input.kind,
        coverage,
        options,
        requests,
        budget,
    )
}

fn compute(
    access: &impl ZeroBornSimplicialAccess,
    metadata: &RipsApproximation,
    kind: RipsInputKind,
    coverage: Coverage,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<PersistenceResult> {
    let (diagram, representatives) =
        simplicial::finish_zero_born(access, options, requests, coverage, budget, |budget| {
            simplicial::cohomology::compute(
                access,
                options.max_homology_dimension(),
                options.field(),
                budget,
            )
        })?;
    Ok(PersistenceResult::new(
        diagram,
        ComputationContext::new(
            options.field(),
            match kind {
                RipsInputKind::Dissimilarities => FiltrationKind::SparseRipsDissimilarities,
                RipsInputKind::Euclidean => FiltrationKind::SparseRipsEuclidean,
                RipsInputKind::Custom => FiltrationKind::SparseRipsCustom,
            },
            access.vertex_count(),
            options.max_edge(),
            metadata.max_scale(),
            Some(metadata.clone()),
        ),
        representatives,
    ))
}
