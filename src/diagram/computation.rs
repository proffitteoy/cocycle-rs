//! Owned mathematical context without borrowed inputs or reducer state.
use super::{PersistenceData, PersistenceDiagram, Representative};

use crate::filtration::FiltrationKind;

/// Owned source context and ordinary persistence analysis settings.
#[derive(Clone, Debug, PartialEq)]
pub struct ComputationContext {
    filtration: crate::filtration::FiltrationContext,
    field: crate::algebra::PrimeField,
    requested_cutoff: Option<f64>,
}
impl ComputationContext {
    /// Reusable source facts, including scale convention and construction metadata.
    pub fn filtration(&self) -> &crate::filtration::FiltrationContext {
        &self.filtration
    }
    /// Sparse Rips provenance, when applicable.
    pub fn approximation(&self) -> Option<&super::RipsApproximation> {
        self.filtration.approximation()
    }
    /// Compatibility classification of the mathematical source.
    pub fn filtration_kind(&self) -> FiltrationKind {
        self.filtration.filtration_kind()
    }
    /// Source vertices, including vertices outside a smaller analysis cutoff.
    pub fn vertex_count(&self) -> usize {
        self.filtration.vertex_count()
    }
    /// Requested computation cutoff before internal stopping optimizations.
    pub fn requested_cutoff(&self) -> Option<f64> {
        self.requested_cutoff
    }
    /// Requested construction cutoff in the source's declared units.
    pub fn construction_cutoff(&self) -> Option<f64> {
        self.filtration.construction_cutoff()
    }
    /// Coefficient field characteristic.
    pub fn characteristic(&self) -> u32 {
        self.field.characteristic()
    }
    pub(crate) fn new(
        field: crate::algebra::PrimeField,
        kind: FiltrationKind,
        vertex_count: usize,
        requested_cutoff: Option<f64>,
        construction_cutoff: Option<f64>,
        approximation: Option<super::RipsApproximation>,
    ) -> Self {
        Self::from_filtration(
            crate::filtration::FiltrationContext::new(
                kind,
                vertex_count,
                construction_cutoff,
                approximation,
            ),
            field,
            requested_cutoff,
        )
    }
    pub(crate) fn from_filtration(
        filtration: crate::filtration::FiltrationContext,
        field: crate::algebra::PrimeField,
        requested_cutoff: Option<f64>,
    ) -> Self {
        Self {
            filtration,
            field,
            requested_cutoff,
        }
    }
}

/// Owned diagram and mathematical context, independent of source lifetimes.
/// Coverage and computed dimensions are recorded in the diagram, not duplicated.
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceResult {
    data: PersistenceData,
    representatives: Option<Vec<Representative>>,
}
impl PersistenceResult {
    pub(crate) fn new(
        diagram: PersistenceDiagram,
        context: ComputationContext,
        representatives: Option<Vec<Representative>>,
    ) -> Self {
        Self {
            data: PersistenceData::new(diagram, context),
            representatives,
        }
    }
    /// Borrow the diagram for existing descriptor operations.
    pub fn diagram(&self) -> &PersistenceDiagram {
        self.data.diagram()
    }
    /// Borrow the mathematical context.
    pub fn context(&self) -> &ComputationContext {
        self.data.context()
    }
    /// Requested representatives, or `None` when no requests were supplied.
    /// An empty slice means requests were made but no intervals were active.
    pub fn representatives(&self) -> Option<&[super::Representative]> {
        self.representatives.as_deref()
    }
    /// Consume the result, explicitly discarding context and representatives.
    pub fn into_diagram(self) -> PersistenceDiagram {
        self.data.into_diagram()
    }
    /// Consume the result, retaining diagram/context and discarding representatives.
    pub fn into_data(self) -> PersistenceData {
        self.data
    }
    /// Move out common data and optional representatives without cloning or sorting.
    /// Representative interval indices still address the returned data's diagram.
    pub fn into_parts(self) -> (PersistenceData, Option<Vec<Representative>>) {
        (self.data, self.representatives)
    }
    pub(crate) fn with_context(mut self, context: ComputationContext) -> Self {
        self.data = self.data.with_context(context);
        self
    }
}
impl AsRef<PersistenceData> for PersistenceResult {
    fn as_ref(&self) -> &PersistenceData {
        &self.data
    }
}
