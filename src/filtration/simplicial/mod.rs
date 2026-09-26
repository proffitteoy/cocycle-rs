//! Explicit simplicial filtration with certified source coverage.
use crate::complex::SimplicialComplex;
use crate::filtration::{Coverage, FiltrationContext};
/// Owned, frozen filtered topology with certified scale and dimension coverage.
/// Only an explicit construction creates this object; arbitrary topology cannot
/// be assigned an unchecked original-Rips completeness claim.
#[derive(Clone, Debug)]
pub struct SimplicialFiltration {
    pub(crate) complex: SimplicialComplex,
    pub(crate) context: FiltrationContext,
    pub(crate) coverage: Coverage,
    pub(crate) dimension: usize,
    pub(crate) complete: bool,
}
impl SimplicialFiltration {
    /// Borrow stored simplices, values, lookup and oriented incidence.
    pub fn complex(&self) -> &SimplicialComplex {
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

mod access;
pub(crate) use access::{ZeroBornExplicitAccess, ZeroBornSimplicialAccess, next_dimension};
