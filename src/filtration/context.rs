//! Source identity and scale conventions shared by construction and analysis.
use super::{FiltrationKind, RipsApproximation, RipsInputKind};

/// Construction-specific facts, separate from storage and reduction settings.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum FiltrationSource {
    /// Rips from a declared input, with approximation facts only when applicable.
    Rips {
        /// Convention used to supply the pairwise values.
        input: RipsInputKind,
        /// Sparse Rips hypotheses, parameters and original vertex mapping.
        approximation: Option<RipsApproximation>,
    },
    /// The complete clique filtration of a user-supplied graph.
    SuppliedFlag,
    /// A supplied face-closed simplicial filtration.
    SuppliedSimplicial,
    /// A supplied filtered cell complex.
    SuppliedCells,
}
/// Interpretation of numeric filtration values; never inferred from storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FiltrationScale {
    /// Original or modified edge lengths, as declared by the Rips/flag source.
    /// This identifies the filtration parameter convention, not physical units,
    /// a metric validation, or equality with the original pairwise distances.
    EdgeLength,
    /// Values supplied by the caller, with no geometric unit asserted by the library.
    Unspecified,
}
/// Owned source facts independent of any persistence field or query.
#[derive(Clone, Debug, PartialEq)]
pub struct FiltrationContext {
    pub(crate) source: FiltrationSource,
    pub(crate) vertex_count: usize,
    pub(crate) construction_cutoff: Option<f64>,
}
impl FiltrationContext {
    /// Construction-specific source facts.
    pub fn source(&self) -> &FiltrationSource {
        &self.source
    }
    /// Numeric parameter convention for this source, not a unit certificate.
    /// Matching conventions still require caller-established units and normalization.
    pub fn scale(&self) -> FiltrationScale {
        match self.source {
            FiltrationSource::Rips { .. } | FiltrationSource::SuppliedFlag => {
                FiltrationScale::EdgeLength
            }
            _ => FiltrationScale::Unspecified,
        }
    }
    /// Compatibility classification of construction and input convention.
    pub fn filtration_kind(&self) -> FiltrationKind {
        match &self.source {
            FiltrationSource::Rips {
                input,
                approximation,
            } => match (input, approximation.is_some()) {
                (RipsInputKind::Dissimilarities, false) => FiltrationKind::RipsDissimilarities,
                (RipsInputKind::Euclidean, false) => FiltrationKind::RipsEuclidean,
                (RipsInputKind::Custom, false) => FiltrationKind::RipsCustom,
                (RipsInputKind::Dissimilarities, true) => FiltrationKind::SparseRipsDissimilarities,
                (RipsInputKind::Euclidean, true) => FiltrationKind::SparseRipsEuclidean,
                (RipsInputKind::Custom, true) => FiltrationKind::SparseRipsCustom,
            },
            FiltrationSource::SuppliedFlag => FiltrationKind::SuppliedFlag,
            FiltrationSource::SuppliedSimplicial => FiltrationKind::SuppliedSimplicial,
            FiltrationSource::SuppliedCells => FiltrationKind::SuppliedCells,
        }
    }
    /// Number of source vertices, independent of original IDs and query cutoff.
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    /// Requested construction cutoff, if the source was truncated during construction.
    pub fn construction_cutoff(&self) -> Option<f64> {
        self.construction_cutoff
    }
    /// Sparse Rips provenance when this is a Rips approximation.
    pub fn approximation(&self) -> Option<&RipsApproximation> {
        match &self.source {
            FiltrationSource::Rips { approximation, .. } => approximation.as_ref(),
            _ => None,
        }
    }
    pub(crate) fn new(
        kind: FiltrationKind,
        vertex_count: usize,
        construction_cutoff: Option<f64>,
        approximation: Option<RipsApproximation>,
    ) -> Self {
        let source = match kind {
            FiltrationKind::RipsDissimilarities | FiltrationKind::SparseRipsDissimilarities => {
                FiltrationSource::Rips {
                    input: RipsInputKind::Dissimilarities,
                    approximation,
                }
            }
            FiltrationKind::RipsEuclidean | FiltrationKind::SparseRipsEuclidean => {
                FiltrationSource::Rips {
                    input: RipsInputKind::Euclidean,
                    approximation,
                }
            }
            FiltrationKind::RipsCustom | FiltrationKind::SparseRipsCustom => {
                FiltrationSource::Rips {
                    input: RipsInputKind::Custom,
                    approximation,
                }
            }
            FiltrationKind::SuppliedFlag => FiltrationSource::SuppliedFlag,
            FiltrationKind::SuppliedSimplicial => FiltrationSource::SuppliedSimplicial,
            FiltrationKind::SuppliedCells => FiltrationSource::SuppliedCells,
        };
        Self {
            source,
            vertex_count,
            construction_cutoff,
        }
    }
}
