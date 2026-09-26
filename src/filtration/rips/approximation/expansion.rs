//! Frozen blocker-aware topology with independent scale and dimension provenance.
use super::{SparseRips, SparseRipsAccess};
use crate::Result;
use crate::complex::SimplicialComplex;
use crate::filtration::RipsInputKind;
use crate::filtration::{Coverage, RipsApproximation};

/// Explicit sparse Rips skeleton with original vertex labels.
#[derive(Clone, Debug)]
pub struct SparseRipsExpansion {
    pub(crate) complex: SimplicialComplex,
    pub(crate) metadata: RipsApproximation,
    pub(crate) coverage: Coverage,
    pub(crate) kind: RipsInputKind,
    pub(crate) max_simplex_dimension: usize,
    pub(crate) complete: bool,
    pub(crate) max_edge: f64,
}
impl SparseRipsExpansion {
    /// Frozen oriented incidence. Every simplex uses original vertex IDs.
    pub fn complex(&self) -> &SimplicialComplex {
        &self.complex
    }
    /// Sampling and approximation provenance.
    pub fn approximation(&self) -> &RipsApproximation {
        &self.metadata
    }
    /// Scale coverage of this approximate filtration.
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }
    /// Original input kind.
    pub fn input_kind(&self) -> RipsInputKind {
        self.kind
    }
    /// Requested construction dimension, distinct from homology dimension.
    pub fn max_simplex_dimension(&self) -> usize {
        self.max_simplex_dimension
    }
    /// Whether every allowed simplex in the constructed range was included.
    pub fn is_dimension_complete(&self) -> bool {
        self.complete
    }
}
impl SparseRips {
    /// Expand through an inclusive simplex dimension, applying the radius blocker.
    ///
    /// Zero constructs only vertices. Values and vertex labels are those of the
    /// approximate filtration. Storage can be exponential; no reduction occurs.
    /// # Errors
    /// Returns allocation or invariant errors, without partial output.
    pub fn expand(&self, max_simplex_dimension: usize) -> Result<SparseRipsExpansion> {
        let access = SparseRipsAccess {
            input: self,
            cutoff: self.graph.max_edge(),
        };
        let (complex, complete) =
            crate::filtration::flag::expand_access(&access, max_simplex_dimension)?;
        Ok(SparseRipsExpansion {
            complex,
            complete,
            max_simplex_dimension,
            metadata: self.metadata.clone(),
            coverage: self.coverage,
            kind: self.kind,
            max_edge: self.graph.max_edge(),
        })
    }
}
