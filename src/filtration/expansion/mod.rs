//! Explicit topology with source context and dimension sufficiency.
use crate::Result;
use crate::complex::FilteredSimplicialComplex;
use crate::execution::{Execution, WorkBudget};
use crate::filtration::flag::{CliqueAccess, expand_access_with};
use crate::filtration::rips::approximation::SparseRipsAccess;
use crate::filtration::{
    Coverage, FiltrationKind, FlagFiltration, RipsApproximation, RipsInputKind, SparseRips,
    ThresholdRips,
};

/// Source facts retained independently of a persistence field or analysis request.
#[derive(Clone, Debug, PartialEq)]
pub struct FiltrationContext {
    pub(crate) kind: FiltrationKind,
    pub(crate) vertex_count: usize,
    pub(crate) construction_cutoff: Option<f64>,
    pub(crate) approximation: Option<RipsApproximation>,
}
impl FiltrationContext {
    /// Mathematical source and input convention.
    pub fn filtration_kind(&self) -> FiltrationKind {
        self.kind
    }
    /// Vertices in this filtration, including isolates and retained approximate IDs.
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    /// Requested construction cutoff, even if all edges were retained.
    pub fn construction_cutoff(&self) -> Option<f64> {
        self.construction_cutoff
    }
    /// Sampling, metric evidence and original vertex mapping, when approximate.
    pub fn approximation(&self) -> Option<&RipsApproximation> {
        self.approximation.as_ref()
    }
}

/// Owned, frozen filtered topology with certified scale and dimension coverage.
/// Only an explicit construction creates this object; arbitrary topology cannot
/// be assigned an unchecked original-Rips completeness claim.
#[derive(Clone, Debug)]
pub struct SimplicialFiltration {
    pub(crate) complex: FilteredSimplicialComplex,
    pub(crate) context: FiltrationContext,
    pub(crate) coverage: Coverage,
    pub(crate) dimension: usize,
    pub(crate) complete: bool,
    pub(crate) max_edge: f64,
}
impl SimplicialFiltration {
    /// Borrow stored simplices, values, lookup and oriented incidence.
    pub fn complex(&self) -> &FilteredSimplicialComplex {
        &self.complex
    }
    /// Original construction facts, independent of any later analysis.
    pub fn context(&self) -> &FiltrationContext {
        &self.context
    }
    /// Certified range of this mathematical source.
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }
    /// Requested maximum simplex dimension, not the homology dimension.
    pub fn max_simplex_dimension(&self) -> usize {
        self.dimension
    }
    /// Whether expansion exhausted the source's simplices within its scale range.
    pub fn is_dimension_complete(&self) -> bool {
        self.complete
    }
}

pub(crate) fn kind(input: RipsInputKind, approximate: bool) -> FiltrationKind {
    match (input, approximate) {
        (RipsInputKind::Dissimilarities, false) => FiltrationKind::RipsDissimilarities,
        (RipsInputKind::Euclidean, false) => FiltrationKind::RipsEuclidean,
        (RipsInputKind::Custom, false) => FiltrationKind::RipsCustom,
        (RipsInputKind::Dissimilarities, true) => FiltrationKind::SparseRipsDissimilarities,
        (RipsInputKind::Euclidean, true) => FiltrationKind::SparseRipsEuclidean,
        (RipsInputKind::Custom, true) => FiltrationKind::SparseRipsCustom,
    }
}

impl ThresholdRips {
    /// Build stored simplices through the inclusive dimension; zero means vertices.
    /// # Errors
    /// Returns allocation/index errors; no partial expansion is returned.
    pub fn build_complex(&self, max_simplex_dimension: usize) -> Result<SimplicialFiltration> {
        self.build_complex_with(max_simplex_dimension, &Execution::default())
    }
    /// Expand with cooperative work and cancellation controls.
    /// # Errors
    /// Includes construction, work-limit and cancellation errors.
    pub fn build_complex_with(
        &self,
        max_simplex_dimension: usize,
        execution: &Execution<'_>,
    ) -> Result<SimplicialFiltration> {
        self.build_complex_budget(max_simplex_dimension, &mut WorkBudget::new(execution)?)
    }
    pub(crate) fn build_complex_budget(
        &self,
        dimension: usize,
        budget: &mut WorkBudget<'_>,
    ) -> Result<SimplicialFiltration> {
        let (complex, complete) = expand_access_with(
            &CliqueAccess::Sparse(self.graph(), self.graph().max_edge()),
            dimension,
            &mut || budget.step(),
        )?;
        budget.check()?;
        Ok(SimplicialFiltration {
            complex,
            complete,
            dimension,
            coverage: self.coverage(),
            max_edge: self.graph().max_edge(),
            context: FiltrationContext {
                kind: kind(self.input_kind(), false),
                vertex_count: self.graph().vertex_count(),
                construction_cutoff: self.requested_cutoff(),
                approximation: None,
            },
        })
    }
}
impl SparseRips {
    /// Build blocker-aware stored simplices with original vertex labels.
    /// # Errors
    /// Returns expansion/index/allocation errors without partial topology.
    pub fn build_complex(&self, max_simplex_dimension: usize) -> Result<SimplicialFiltration> {
        self.build_complex_with(max_simplex_dimension, &Execution::default())
    }
    /// Expand with cooperative controls, including stored incidence work.
    /// # Errors
    /// Includes construction, work-limit and cancellation errors.
    pub fn build_complex_with(
        &self,
        max_simplex_dimension: usize,
        execution: &Execution<'_>,
    ) -> Result<SimplicialFiltration> {
        self.build_complex_budget(max_simplex_dimension, &mut WorkBudget::new(execution)?)
    }
    pub(crate) fn build_complex_budget(
        &self,
        dimension: usize,
        budget: &mut WorkBudget<'_>,
    ) -> Result<SimplicialFiltration> {
        let access = SparseRipsAccess {
            input: self,
            cutoff: self.graph().max_edge(),
        };
        let (complex, complete) = expand_access_with(&access, dimension, &mut || budget.step())?;
        budget.check()?;
        Ok(SimplicialFiltration {
            complex,
            complete,
            dimension,
            coverage: self.coverage(),
            max_edge: self.graph().max_edge(),
            context: FiltrationContext {
                kind: kind(self.input_kind(), true),
                vertex_count: self.graph().vertex_count(),
                construction_cutoff: self.approximation().max_scale(),
                approximation: Some(self.approximation().clone()),
            },
        })
    }
}
impl FlagFiltration {
    /// Build stored cliques with supplied-graph provenance and dimension coverage.
    /// # Errors
    /// Returns expansion/index/allocation errors without partial topology.
    pub fn build_complex(&self, max_simplex_dimension: usize) -> Result<SimplicialFiltration> {
        self.build_complex_with(max_simplex_dimension, &Execution::default())
    }
    /// Expand under cooperative controls without densifying the graph.
    /// # Errors
    /// Includes construction, work-limit and cancellation errors.
    pub fn build_complex_with(
        &self,
        dimension: usize,
        execution: &Execution<'_>,
    ) -> Result<SimplicialFiltration> {
        let mut budget = WorkBudget::new(execution)?;
        let (complex, complete) = expand_access_with(
            &CliqueAccess::Sparse(self.graph(), self.graph().max_edge()),
            dimension,
            &mut || budget.step(),
        )?;
        budget.check()?;
        Ok(SimplicialFiltration {
            complex,
            complete,
            dimension,
            coverage: Coverage::Complete,
            max_edge: self.graph().max_edge(),
            context: FiltrationContext {
                kind: FiltrationKind::SuppliedFlag,
                vertex_count: self.graph().vertex_count(),
                construction_cutoff: None,
                approximation: None,
            },
        })
    }
}
