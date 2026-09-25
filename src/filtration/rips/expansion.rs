//! Explicit Rips topology with retained construction provenance.
use super::{RipsInputKind, ThresholdRips};
use crate::Result;
use crate::complex::FilteredSimplicialComplex;
use crate::filtration::Coverage;

/// A frozen exact Rips skeleton and the provenance needed to interpret it.
///
/// Unlike a bare complex, this object distinguishes a dimension truncation
/// from a scale truncation. It owns its topology and can outlive the input.
#[derive(Clone, Debug)]
pub struct RipsExpansion {
    pub(crate) complex: FilteredSimplicialComplex,
    pub(crate) max_simplex_dimension: usize,
    pub(crate) complete: bool,
    pub(crate) vertex_count: usize,
    pub(crate) coverage: Coverage,
    pub(crate) kind: RipsInputKind,
    pub(crate) requested_cutoff: Option<f64>,
    pub(crate) max_edge: f64,
}
impl RipsExpansion {
    /// Borrow the explicit filtered topology and its incidence queries.
    pub fn complex(&self) -> &FilteredSimplicialComplex {
        &self.complex
    }
    /// Requested construction dimension, distinct from a homology dimension.
    pub fn max_simplex_dimension(&self) -> usize {
        self.max_simplex_dimension
    }
    /// Whether every clique in the constructed threshold graph was included.
    /// This does not certify completeness beyond the recorded scale coverage.
    pub fn is_dimension_complete(&self) -> bool {
        self.complete
    }
    /// Known scale range with respect to the original distance input.
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }
    /// Original input source kind.
    pub fn input_kind(&self) -> RipsInputKind {
        self.kind
    }
    /// Requested threshold used to construct the source graph.
    pub fn requested_cutoff(&self) -> Option<f64> {
        self.requested_cutoff
    }
}
impl ThresholdRips {
    /// Expand cliques through an inclusive simplex dimension. Zero builds only vertices.
    ///
    /// Values are maximum edge lengths. Stores all simplices and oriented
    /// incidence, potentially exponentially many. No persistence is performed.
    ///
    /// # Errors
    /// Returns allocation or invariant errors; never a partial expansion.
    pub fn expand(&self, max_simplex_dimension: usize) -> Result<RipsExpansion> {
        let (complex, complete) =
            crate::filtration::flag::expand(self.graph(), max_simplex_dimension)?;
        Ok(RipsExpansion {
            complex,
            complete,
            max_simplex_dimension,
            vertex_count: self.graph().vertex_count(),
            coverage: self.coverage(),
            kind: self.input_kind(),
            requested_cutoff: self.requested_cutoff(),
            max_edge: self.graph().max_edge(),
        })
    }
}
