//! Shared owned diagram and context, without algorithm-specific auxiliary output.
use super::{ComputationContext, PersistenceDiagram};

/// Owned ordinary persistence data shared by compatible result types.
///
/// Coverage and computed dimensions belong to the diagram. Source facts, field
/// and requested range belong to the context. Library computations create this
/// data; external wrappers can retain it and implement `AsRef<PersistenceData>`
/// by borrowing. Such a borrow establishes no additional mathematical guarantees.
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceData {
    diagram: PersistenceDiagram,
    context: ComputationContext,
}

impl PersistenceData {
    pub(crate) fn new(diagram: PersistenceDiagram, context: ComputationContext) -> Self {
        Self { diagram, context }
    }
    /// Borrow the diagram, including its computed dimensions and coverage.
    pub fn diagram(&self) -> &PersistenceDiagram {
        &self.diagram
    }
    /// Borrow the source facts and computation settings.
    pub fn context(&self) -> &ComputationContext {
        &self.context
    }
    /// Consume the data, explicitly discarding its mathematical context.
    pub fn into_diagram(self) -> PersistenceDiagram {
        self.diagram
    }
    pub(crate) fn with_context(mut self, context: ComputationContext) -> Self {
        self.context = context;
        self
    }
}

impl AsRef<PersistenceData> for PersistenceData {
    fn as_ref(&self) -> &PersistenceData {
        self
    }
}
