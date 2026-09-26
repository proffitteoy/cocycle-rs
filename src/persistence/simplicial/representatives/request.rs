//! Explicit requests for basis representatives at specified scales.
use crate::Result;

/// Which basis payloads to retain for each active interval at a query scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepresentativeSelection {
    /// Homology cycles compatible with the persistent interval decomposition.
    Cycles,
    /// Cohomology classes dual to those cycles at this query scale.
    Cocycles,
    /// Both bases, with identity evaluation pairing over the selected field.
    Both,
}
/// A dimension, finite scale, and selection for an optional representative computation.
///
/// A call emits one representative of each selected kind per active interval.
/// Death scales are excluded; a censoring cutoff is included. Requested
/// dimensions and scales must lie within the computation's declared coverage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RepresentativeRequest {
    pub(super) dimension: usize,
    pub(super) scale: f64,
    pub(super) selection: RepresentativeSelection,
}
impl RepresentativeRequest {
    /// Validate a representative query; field and coverage come from the computation.
    ///
    /// # Errors
    /// Rejects non-finite scales; signed filtration values are supported. Dimension/coverage are checked at computation.
    pub fn new(dimension: usize, scale: f64, selection: RepresentativeSelection) -> Result<Self> {
        Ok(Self {
            dimension,
            scale: crate::persistence::options::finite_scale(scale)?,
            selection,
        })
    }
    /// Requested homology/cohomology dimension.
    pub fn dimension(&self) -> usize {
        self.dimension
    }
    /// Inclusive query scale.
    pub fn scale(&self) -> f64 {
        self.scale
    }
    /// Selected basis kinds.
    pub fn selection(&self) -> RepresentativeSelection {
        self.selection
    }
}
