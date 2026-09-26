//! Source-specific expansion into contextual simplicial filtrations.
use crate::Result;
use crate::execution::{Execution, WorkBudget};
use crate::filtration::flag::{CliqueAccess, expand_access_with};
use crate::filtration::rips::approximation::SparseRipsAccess;
use crate::filtration::{
    Coverage, FiltrationKind, FlagFiltration, RipsInputKind, SparseRips, ThresholdRips,
};

use crate::filtration::{FiltrationContext, SimplicialFiltration};
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
            complete,
            dimension,
            coverage: self.coverage(),
            complex,
            context: FiltrationContext::new(
                kind(self.input_kind(), false),
                self.graph().vertex_count(),
                self.requested_cutoff(),
                None,
            ),
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
            complete,
            dimension,
            coverage: self.coverage(),
            complex,
            context: FiltrationContext::new(
                kind(self.input_kind(), true),
                self.graph().vertex_count(),
                self.approximation().max_scale(),
                Some(self.approximation().clone()),
            ),
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
            complete,
            dimension,
            coverage: Coverage::Complete,
            complex,
            context: FiltrationContext::new(
                FiltrationKind::SuppliedFlag,
                self.graph().vertex_count(),
                None,
                None,
            ),
        })
    }
}
