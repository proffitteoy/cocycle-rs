//! Owned mathematical context without borrowed inputs or reducer state.
use super::PersistenceDiagram;

use crate::filtration::FiltrationKind;

/// Owned context interpreting an ordinary persistence computation.
///
/// All currently supported paths use edge-length scales, zero vertex births,
/// prime-field coefficients and exact reduction of the declared filtration.
/// Approximate constructions additionally record their parameters and vertex mapping.
#[derive(Clone, Debug, PartialEq)]
pub struct ComputationContext {
    pub(crate) approximation: Option<super::RipsApproximation>,
    pub(crate) field: crate::algebra::PrimeField,
    pub(crate) kind: FiltrationKind,
    pub(crate) vertex_count: usize,
    pub(crate) requested_cutoff: Option<f64>,
    pub(crate) construction_cutoff: Option<f64>,
}
impl ComputationContext {
    /// Sparse Rips provenance, or `None` for an exact construction.
    pub fn approximation(&self) -> Option<&super::RipsApproximation> {
        self.approximation.as_ref()
    }
    /// The mathematical object computed.
    pub fn filtration_kind(&self) -> FiltrationKind {
        self.kind
    }
    /// Number of vertices in the computed filtration, including isolated ones.
    /// For sparse approximations the original count is the permutation length.
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    /// Requested computation cutoff, before internal stopping optimizations.
    pub fn requested_cutoff(&self) -> Option<f64> {
        self.requested_cutoff
    }
    /// Source construction cutoff, if a threshold graph was constructed.
    pub fn construction_cutoff(&self) -> Option<f64> {
        self.construction_cutoff
    }
    /// Coefficient field characteristic.
    pub fn characteristic(&self) -> u32 {
        self.field.characteristic()
    }
}

/// Owned diagram and mathematical context, independent of source lifetimes.
/// Coverage and computed dimensions are recorded in the diagram, not duplicated.
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceResult {
    pub(crate) diagram: PersistenceDiagram,
    pub(crate) context: ComputationContext,
    pub(crate) representatives: Option<Vec<super::Representative>>,
}
impl PersistenceResult {
    /// Borrow the diagram for existing descriptor operations.
    pub fn diagram(&self) -> &PersistenceDiagram {
        &self.diagram
    }
    /// Borrow the mathematical context.
    pub fn context(&self) -> &ComputationContext {
        &self.context
    }
    /// Requested representatives, or `None` when no requests were supplied.
    /// An empty slice means requests were made but no intervals were active.
    pub fn representatives(&self) -> Option<&[super::Representative]> {
        self.representatives.as_deref()
    }
    /// Consume the result, explicitly discarding context and representatives.
    pub fn into_diagram(self) -> PersistenceDiagram {
        self.diagram
    }
}
