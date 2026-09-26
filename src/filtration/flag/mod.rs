//! A supplied graph's clique filtration and shared dense/sparse access.
mod access;
mod cliques;
mod expansion;
pub(crate) use cliques::CliqueAccess;
pub(crate) use expansion::{expand, expand_access, expand_access_with};
mod dense;
mod index;
mod order;
mod sparse;

use crate::complex::WeightedGraph;
pub(crate) use access::FlagAccess;
pub(crate) use dense::DenseFlag;
pub(crate) use order::SimplexEntry;
pub(crate) use sparse::SparseFlag;

/// The complete clique filtration of a supplied weighted graph.
///
/// Vertices enter at zero; each clique enters at its maximum edge weight.
/// Missing edges never enter, so this object can have essential higher classes.
/// It does not certify Rips coverage of an unknown original distance matrix.
#[derive(Clone, Debug)]
pub struct FlagFiltration {
    graph: WeightedGraph,
}
impl FlagFiltration {
    /// Consume a validated graph without copying it.
    pub fn new(graph: WeightedGraph) -> Self {
        Self { graph }
    }
    /// Borrow the graph; vertex indices are unchanged.
    pub fn graph(&self) -> &WeightedGraph {
        &self.graph
    }
}

impl FlagFiltration {
    /// Expand the supplied graph's clique filtration through the given dimension.
    ///
    /// Zero returns vertices only. Missing edges remain absent. The returned
    /// bare complex records stored topology, without certifying Rips provenance.
    /// Expansion stores all simplices and incidences and can be exponential.
    ///
    /// # Errors
    /// Returns allocation or internal invariant errors.
    pub fn expand(
        &self,
        max_simplex_dimension: usize,
    ) -> crate::Result<crate::complex::SimplicialComplex> {
        Ok(expand(self.graph(), max_simplex_dimension)?.0)
    }
}
