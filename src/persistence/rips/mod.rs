//! Vietoris-Rips persistence and its computation options.
//!
//! Rips computation targets ordinary homology over prime fields, with an
//! edge-length filtration and a closed cutoff. H1 uses implicit persistent
//! cohomology; H0-only requests use an independent union-find path.

use super::assemble_diagram;
use crate::Result;
use crate::diagram::{Coverage, PersistenceDiagram};
use crate::geometry::{DissimilarityView, PointCloudView, euclidean_distances};

use super::execution::WorkBudget;
use super::{ExecutionLimits, PersistenceOptions, RepresentativeRequest};
use super::{flag, simplicial};
use crate::filtration::flag::CliqueAccess;
pub(super) mod approximation;
pub(super) mod expanded;
pub use approximation::{
    compute_expanded_sparse_rips, compute_expanded_sparse_rips_with_representatives,
    compute_sparse_rips, compute_sparse_rips_with_representatives,
};
mod options;
pub use expanded::{compute_expanded_rips, compute_expanded_rips_with_representatives};

pub use options::RipsOptions;

/// Compute ordinary Rips persistence over F2 from a symmetric dissimilarity matrix.
///
/// The cutoff is inclusive and measured in edge lengths. Zero-length intervals
/// are omitted. If the cutoff is below the input diameter, surviving classes are
/// right-censored; otherwise coverage is complete. No triangle inequality is
/// required. H1 uses the 2-skeleton, including triangles that kill cycles.
///
/// H1 stores edges and change-of-basis columns, generating triangle cofacets
/// on demand. It avoids materializing the full 2-skeleton, but reduction fill-in
/// and repeated cofacet enumeration can still be large. No automatic sampling,
/// approximation or process memory bound is applied. An internal cone bound may
/// stop computation early without changing the public coverage convention.
///
/// # Errors
/// Returns an error if simplex indexing exceeds the supported integer range,
/// a fallible allocation fails, or an internal invariant is violated. It never
/// returns a partial diagram on failure.
///
/// ```
/// use cocycle::geometry::DissimilarityView;
/// use cocycle::persistence::{RipsOptions, rips_from_dissimilarities};
/// let input = DissimilarityView::new(&[2.0], 2)?;
/// let diagram = rips_from_dissimilarities(input, &RipsOptions::default())?;
/// assert_eq!(diagram.intervals_in_dimension(0)?.count(), 2);
/// # Ok::<(), cocycle::Error>(())
/// ```
pub fn rips_from_dissimilarities(
    input: DissimilarityView<'_>,
    options: &RipsOptions,
) -> Result<PersistenceDiagram> {
    let (cutoff, coverage) = resolve_rips_range(input, options);
    let mut budget = WorkBudget::new(&ExecutionLimits::default())?;
    let raw = flag::compute_dense(
        input.into(),
        options.max_dimension(),
        cutoff,
        crate::algebra::PrimeField::default(),
        &mut budget,
    )?;
    assemble_diagram(options.max_dimension(), coverage, raw)
}

/// Compute ordinary Rips persistence over F2 from a Euclidean point cloud.
///
/// Computes one condensed distance buffer, then follows
/// [`rips_from_dissimilarities`]. Coordinates are not normalized or deduplicated.
/// Euclidean norms use scaled `hypot` operations to avoid unnecessary square-sum
/// overflow and underflow. The returned diagram does not borrow the point cloud.
///
/// # Errors
/// In addition to persistence errors, returns an error if distance storage cannot
/// be reserved or a Euclidean distance is not representable as a finite `f64`.
pub fn rips_from_points(
    input: PointCloudView<'_>,
    options: &RipsOptions,
) -> Result<PersistenceDiagram> {
    let distances = euclidean_distances(input)?;
    rips_from_dissimilarities(DissimilarityView::new(&distances, input.len())?, options)
}

pub(super) fn resolve_rips_range(
    input: DissimilarityView<'_>,
    options: &RipsOptions,
) -> (f64, Coverage) {
    match options.max_edge() {
        Some(cutoff) if cutoff < input.diameter() => (cutoff, Coverage::Through(cutoff)),
        _ => (input.diameter(), Coverage::Complete),
    }
}

/// Compute dimension-generic prime-field persistence from any validated matrix layout without converting buffers.
///
/// Produces an owned diagram and exact-Rips context. The cutoff is inclusive;
/// survivors below the input diameter are censored. Internal cone stopping does
/// not change this coverage. See [`ExecutionLimits`] for counted work and limits.
///
/// # Errors
/// Returns index/size/allocation errors, cancellation, work exhaustion, or an
/// internal invariant failure. No partial result is returned.
pub fn compute_rips_from_distances(
    input: crate::geometry::DissimilarityMatrixView<'_>,
    options: &PersistenceOptions,
    limits: &ExecutionLimits<'_>,
) -> Result<crate::diagram::PersistenceResult> {
    compute_rips_from_distances_with_representatives(input, options, &[], limits)
}

/// Compute persistence and requested cycle/cocycle bases for this input.
///
/// Uses the same input and coverage contracts as [`compute_rips_from_distances`].
/// Nonempty requests opt into explicit skeleton materialization and boundary
/// transformations; execution limits include this extra work. No cone stopping
/// discards simplices needed at a representative query scale.
///
/// # Errors
/// Includes [`compute_rips_from_distances`] errors, uncomputed dimensions and query
/// scales outside known coverage. Failure returns no partial result.
pub fn compute_rips_from_distances_with_representatives(
    input: crate::geometry::DissimilarityMatrixView<'_>,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    limits: &ExecutionLimits<'_>,
) -> Result<crate::diagram::PersistenceResult> {
    let mut budget = WorkBudget::new(limits)?;
    compute_rips_from_distances_budget(input, options, requests, &mut budget)
}

pub(in crate::persistence) fn compute_rips_from_distances_budget(
    input: crate::geometry::DissimilarityMatrixView<'_>,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<crate::diagram::PersistenceResult> {
    use crate::diagram::{ComputationContext, FiltrationKind, PersistenceResult};

    let (cutoff, coverage) = match options.max_edge() {
        Some(t) if t < input.diameter() => (t, Coverage::Through(t)),
        _ => (input.diameter(), Coverage::Complete),
    };
    let (diagram, representatives) = simplicial::finish_zero_born(
        &CliqueAccess::Dense(input, cutoff),
        options,
        requests,
        coverage,
        budget,
        |budget| {
            flag::compute_dense(
                input,
                options.max_homology_dimension(),
                cutoff,
                options.field(),
                budget,
            )
        },
    )?;
    Ok(PersistenceResult::new(
        diagram,
        ComputationContext::new(
            options.field(),
            FiltrationKind::RipsDissimilarities,
            input.len(),
            options.max_edge(),
            None,
            None,
        ),
        representatives,
    ))
}

/// Compute from an exact threshold graph, preserving its original-input coverage.
///
/// `None` uses the available construction range. A requested scale above an
/// incomplete construction is rejected. It never certifies completeness using
/// only the largest stored edge. Uses sparse adjacency without densification.
///
/// # Errors
/// Returns [`crate::Error::IncompleteFiltration`] when the requested range is
/// unavailable, in addition to the computation errors of [`compute_rips_from_distances`].
pub fn compute_threshold_rips(
    input: &crate::filtration::ThresholdRips,
    options: &PersistenceOptions,
    limits: &ExecutionLimits<'_>,
) -> Result<crate::diagram::PersistenceResult> {
    compute_threshold_rips_with_representatives(input, options, &[], limits)
}

/// Compute persistence and requested cycle/cocycle bases for this input.
///
/// Uses the same input and coverage contracts as [`compute_threshold_rips`].
/// Nonempty requests opt into explicit skeleton materialization and boundary
/// transformations; execution limits include this extra work. No cone stopping
/// discards simplices needed at a representative query scale.
///
/// # Errors
/// Includes [`compute_threshold_rips`] errors, uncomputed dimensions and query
/// scales outside known coverage. Failure returns no partial result.
pub fn compute_threshold_rips_with_representatives(
    input: &crate::filtration::ThresholdRips,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    limits: &ExecutionLimits<'_>,
) -> Result<crate::diagram::PersistenceResult> {
    let mut budget = WorkBudget::new(limits)?;
    compute_threshold_rips_budget(input, options, requests, &mut budget)
}

pub(in crate::persistence) fn compute_threshold_rips_budget(
    input: &crate::filtration::ThresholdRips,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<crate::diagram::PersistenceResult> {
    use crate::diagram::{ComputationContext, FiltrationKind, PersistenceResult};
    use crate::filtration::RipsInputKind;

    let (cutoff, coverage) = crate::persistence::options::source_range(
        input.coverage(),
        input.graph().max_edge(),
        options.max_edge(),
    )?;
    let (diagram, representatives) = simplicial::finish_zero_born(
        &CliqueAccess::Sparse(input.graph(), cutoff),
        options,
        requests,
        coverage,
        budget,
        |budget| {
            flag::compute_graph(
                input.graph(),
                options.max_homology_dimension(),
                cutoff,
                options.field(),
                budget,
            )
        },
    )?;
    let kind = match input.input_kind() {
        RipsInputKind::Dissimilarities => FiltrationKind::RipsDissimilarities,
        RipsInputKind::Euclidean => FiltrationKind::RipsEuclidean,
        RipsInputKind::Custom => FiltrationKind::RipsCustom,
    };
    Ok(PersistenceResult::new(
        diagram,
        ComputationContext::new(
            options.field(),
            kind,
            input.graph().vertex_count(),
            options.max_edge(),
            input.requested_cutoff(),
            None,
        ),
        representatives,
    ))
}

/// Compute Euclidean prime-field persistence with owned mathematical context.
///
/// A finite cutoff constructs a threshold graph directly, without a dense
/// distance buffer. Without a cutoff, materializes one condensed matrix.
/// Coordinate indices and duplicate points are retained. Execution controls
/// apply to persistence, not distance/graph construction; cancellation is also
/// checked before and after that preparation.
///
/// # Errors
/// Returns distance/construction errors and the errors of the selected exact
/// computation path, without a partial result.
pub fn compute_rips_from_points(
    input: PointCloudView<'_>,
    options: &PersistenceOptions,
    limits: &ExecutionLimits<'_>,
) -> Result<crate::diagram::PersistenceResult> {
    compute_rips_from_points_with_representatives(input, options, &[], limits)
}

/// Compute persistence and requested cycle/cocycle bases for this input.
///
/// Uses the same input and coverage contracts as [`compute_rips_from_points`].
/// Nonempty requests opt into explicit skeleton materialization and boundary
/// transformations; execution limits include this extra work. No cone stopping
/// discards simplices needed at a representative query scale.
///
/// # Errors
/// Includes [`compute_rips_from_points`] errors, uncomputed dimensions and query
/// scales outside known coverage. Failure returns no partial result.
pub fn compute_rips_from_points_with_representatives(
    input: PointCloudView<'_>,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    limits: &ExecutionLimits<'_>,
) -> Result<crate::diagram::PersistenceResult> {
    WorkBudget::new(limits)?.check()?;
    if options.max_edge().is_some() {
        let graph = crate::filtration::threshold_rips_from_points(input, options.max_edge())?;
        compute_threshold_rips_with_representatives(&graph, options, requests, limits)
    } else {
        let values = euclidean_distances(input)?;
        let result = compute_rips_from_distances_with_representatives(
            DissimilarityView::new(&values, input.len())?.into(),
            options,
            requests,
            limits,
        )?;
        let context = crate::diagram::ComputationContext::new(
            options.field(),
            crate::diagram::FiltrationKind::RipsEuclidean,
            input.len(),
            options.max_edge(),
            None,
            None,
        );
        Ok(result.with_context(context))
    }
}
