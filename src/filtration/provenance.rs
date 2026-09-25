//! Source coverage and mathematical provenance shared by construction and analysis.
/// The range over which a diagram was computed.
///
/// This describes coverage of the filtration, not a memory limit. A raw `Through`
/// value is validated by [`crate::diagram::PersistenceDiagram::new`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Coverage {
    /// All changes in the filtration are accounted for.
    Complete,
    /// All changes up to and including the given finite scale are accounted for.
    Through(f64),
}

/// Mathematical source of a computed ordinary persistence diagram.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FiltrationKind {
    /// Exact Rips of supplied symmetric dissimilarities (not necessarily a metric).
    RipsDissimilarities,
    /// Exact Rips of computed Euclidean distances.
    RipsEuclidean,
    /// Exact Rips of sampled custom symmetric dissimilarities.
    RipsCustom,
    /// Clique filtration of a supplied graph; absent edges never enter.
    SuppliedFlag,
    /// Sparse Rips of dissimilarities, with owned hypothesis and mapping metadata.
    SparseRipsDissimilarities,
    /// Sparse Rips of computed Euclidean distances.
    SparseRipsEuclidean,
    /// Sparse Rips of sampled custom symmetric distances.
    SparseRipsCustom,
}
