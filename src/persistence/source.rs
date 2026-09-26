//! Consumer-owned source support; engine dispatch stays out of public traits.
use super::{PersistenceBuilder, PersistenceOptions, RepresentativeRequest, flag, rips};
use crate::complex::SimplicialComplex;
use crate::diagram::{ComputationContext, PersistenceResult};
use crate::execution::WorkBudget;
use crate::filtration::rips::builder::Input;
use crate::filtration::{
    ApproximateRipsBuilder, FlagFiltration, RipsBuilder, RipsExpansion, SimplicialFiltration,
    SparseRips, SparseRipsExpansion, ThresholdRips,
};
use crate::geometry::{DissimilarityMatrixView, MatrixLayout};
use crate::{Error, Result};

pub trait Sealed {}

/// Add a configurable persistence operation to supported library sources.
///
/// Import this trait to use `.persistence()`. The method neither computes nor
/// mutates a cache. Implementations are sealed; implicit simplex access is private.
///
/// Unexpanded sources deliberately provide no simplex queries:
/// ```compile_fail
/// use cocycle::{filtration::RipsBuilder, geometry::PointCloudView};
/// let data = [0., 1.];
/// let points = PointCloudView::new(&data, 2, 1).unwrap();
/// RipsBuilder::from_points(points).find(&[0, 1]);
/// ```
/// Stateful callbacks must be consumed into a prepared source or explicit complex:
/// ```compile_fail
/// use cocycle::{filtration::RipsBuilder, persistence::PersistenceExt};
/// let items = [0., 1.];
/// let callback = RipsBuilder::from_distance_fn(&items, |_, _| Ok(1.));
/// callback.persistence();
/// ```
/// Analysis requests cannot outlive their source:
/// ```compile_fail
/// use cocycle::{filtration::RipsBuilder, geometry::PointCloudView, persistence::PersistenceExt};
/// let values = [0., 1.];
/// let points = PointCloudView::new(&values, 2, 1).unwrap();
/// let request;
/// { let rips = RipsBuilder::from_points(points); request = rips.persistence(); }
/// request.compute();
/// ```
pub trait PersistenceExt: Sealed + Sized {
    /// Borrow this source for analysis; expensive work begins only at `compute`.
    fn persistence(&self) -> PersistenceBuilder<'_, 'static, Self>;
}
macro_rules! source {
    ($ty:ty, $compute:path) => {
        impl Sealed for $ty {}
        impl PersistenceExt for $ty {
            fn persistence(&self) -> PersistenceBuilder<'_, 'static, Self> {
                PersistenceBuilder::new(self, $compute)
            }
        }
    };
}
source!(RipsBuilder<'_>, exact);
source!(ApproximateRipsBuilder<'_>, approximate);
source!(ThresholdRips, rips::compute_threshold_rips_budget);
source!(FlagFiltration, flag::compute_flag_budget);
source!(SparseRips, rips::approximation::compute_sparse_rips_budget);
source!(RipsExpansion, rips::expanded::compute_expanded_rips_budget);
source!(
    SparseRipsExpansion,
    rips::approximation::compute_expanded_sparse_rips_budget
);
impl Sealed for SimplicialFiltration {}
impl PersistenceExt for SimplicialFiltration {
    fn persistence(&self) -> PersistenceBuilder<'_, 'static, Self> {
        PersistenceBuilder::new_filtered(self, explicit)
    }
}
impl Sealed for SimplicialComplex {}
impl PersistenceExt for SimplicialComplex {
    fn persistence(&self) -> PersistenceBuilder<'_, 'static, Self> {
        PersistenceBuilder::new_filtered(self, supplied)
    }
}

fn exact(
    input: &RipsBuilder<'_>,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<PersistenceResult> {
    input.validate()?;
    let result = match input.input {
        Input::Matrix(matrix) => {
            if let Some(through) = input.max_edge.filter(|t| *t < matrix.diameter())
                && options.max_edge().is_some_and(|t| t > through)
            {
                return Err(Error::IncompleteFiltration {
                    requested: options.max_edge().unwrap(),
                    through,
                });
            }
            let effective = match (input.max_edge, options.max_edge()) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            let effective = PersistenceOptions::new(options.max_homology_dimension(), effective)?
                .with_field(options.field());
            rips::compute_rips_from_distances_budget(matrix, &effective, requests, budget)?
        }
        Input::Points(_) if input.max_edge.or(options.max_edge()).is_some() => {
            // Retain only edges needed by this analysis. Construction still scans
            // every pair, so coverage records omitted edges even across a gap in
            // edge values. The caller's source cap is restored in result context.
            let effective = match (input.max_edge, options.max_edge()) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            let prepared = RipsBuilder {
                input: input.input,
                max_edge: effective,
            }
            .prepare_budget(budget)?;
            rips::compute_threshold_rips_budget(&prepared, options, requests, budget)?
        }
        Input::Points(points) => {
            let values = crate::geometry::euclidean_distances_with(points, &mut || budget.step())?;
            budget.check()?;
            let matrix =
                DissimilarityMatrixView::new(&values, points.len(), MatrixLayout::LowerTriangle)?;
            rips::compute_rips_from_distances_budget(matrix, options, requests, budget)?
        }
    };
    let context = ComputationContext::new(
        options.field(),
        crate::filtration::expansion::kind(input.input.kind(), false),
        input.input.len(),
        options.max_edge(),
        input.max_edge,
        None,
    );
    budget.check()?;
    Ok(result.with_context(context))
}
fn approximate(
    input: &ApproximateRipsBuilder<'_>,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<PersistenceResult> {
    input.validate()?;
    let prepared = input.prepare_budget(budget)?;
    rips::approximation::compute_sparse_rips_budget(&prepared, options, requests, budget)
}
fn explicit(
    input: &SimplicialFiltration,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<PersistenceResult> {
    if !input.complete && options.max_homology_dimension() >= input.dimension {
        return Err(Error::InsufficientSkeleton {
            requested_homology_dimension: options.max_homology_dimension(),
            constructed_simplex_dimension: input.dimension,
        });
    }
    let (cutoff, coverage) = super::simplicial::source_range(
        input.coverage,
        input.complex.max_filtration_value(),
        options.max_edge(),
    )?;
    let effective = PersistenceOptions::for_filtration(options.max_homology_dimension(), cutoff)?
        .with_field(options.field());
    let (diagram, representatives) =
        super::simplicial::compute(&input.complex, &effective, requests, coverage, budget)?;
    Ok(PersistenceResult::new(
        diagram,
        ComputationContext::from_filtration(
            input.context.clone(),
            options.field(),
            options.max_edge(),
        ),
        representatives,
    ))
}

fn supplied(
    input: &SimplicialComplex,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<PersistenceResult> {
    let (_, coverage) = super::simplicial::source_range(
        crate::filtration::Coverage::Complete,
        input.max_filtration_value(),
        options.max_edge(),
    )?;
    let (diagram, representatives) =
        super::simplicial::compute(input, options, requests, coverage, budget)?;
    Ok(PersistenceResult::new(
        diagram,
        ComputationContext::new(
            options.field(),
            crate::filtration::FiltrationKind::SuppliedSimplicial,
            input.vertex_count(),
            options.max_edge(),
            None,
            None,
        ),
        representatives,
    ))
}
