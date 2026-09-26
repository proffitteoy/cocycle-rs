//! Validated topology storage and a read-only filtered-cell contract.
//! A simplex is one cell; a simplicial complex is a face-closed collection.

mod graph;
pub use graph::{Neighbor, WeightedEdge, WeightedGraph};

mod simplicial;
pub use simplicial::{BoundaryTerm, Simplex, SimplexId, SimplicialComplex};

pub(crate) use simplicial::compare_filtration;

mod filtered;
pub use filtered::FilteredComplex;
/// Compatibility name for [`SimplicialComplex`].
pub type FilteredSimplicialComplex = SimplicialComplex;
