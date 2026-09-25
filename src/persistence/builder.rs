//! One analysis request independent of input representation and engine state.
use super::{PersistenceOptions, RepresentativeRequest};
use crate::algebra::PrimeField;
use crate::diagram::PersistenceResult;
use crate::execution::{Execution, WorkBudget};
use crate::{Error, Result};

type Compute<S> = fn(
    &S,
    &PersistenceOptions,
    &[RepresentativeRequest],
    &mut WorkBudget<'_>,
) -> Result<PersistenceResult>;

/// Configure ordinary persistence for a borrowed supported mathematical source.
///
/// Obtain this request through [`super::PersistenceExt::persistence`]. Setters are
/// last-call-wins; only `compute` executes. Source and representative requests
/// remain borrowed until execution finishes; the result owns its data.
/// Default coefficients are F2, the largest homology dimension is one, and the
/// source's entire available scale range is used.
pub struct PersistenceBuilder<'s, 'r, S> {
    source: &'s S,
    compute: Compute<S>,
    dimension: usize,
    cutoff: Option<f64>,
    field: PrimeField,
    requests: &'r [RepresentativeRequest],
}
impl<'s, S> PersistenceBuilder<'s, 'static, S> {
    pub(super) fn new(source: &'s S, compute: Compute<S>) -> Self {
        Self {
            source,
            compute,
            dimension: 1,
            cutoff: None,
            field: PrimeField::default(),
            requests: &[],
        }
    }
}
impl<'s, S> PersistenceBuilder<'s, '_, S> {
    /// Compute all homology dimensions from zero through this inclusive maximum.
    pub fn max_homology_dimension(mut self, dimension: usize) -> Self {
        self.dimension = dimension;
        self
    }
    /// Select a validated prime field without narrowing its characteristic.
    pub fn field(mut self, field: PrimeField) -> Self {
        self.field = field;
        self
    }
    /// Restrict analysis in the source's filtration units; cannot widen coverage.
    pub fn max_filtration_value(mut self, value: f64) -> Self {
        self.cutoff = Some(value);
        self
    }
    /// Clear the analysis cutoff, leaving source construction coverage unchanged.
    pub fn available_range(mut self) -> Self {
        self.cutoff = None;
        self
    }
    /// Replace representative requests; their output bases are owned by the result.
    pub fn representatives<'r>(
        self,
        requests: &'r [RepresentativeRequest],
    ) -> PersistenceBuilder<'s, 'r, S> {
        PersistenceBuilder {
            source: self.source,
            compute: self.compute,
            dimension: self.dimension,
            cutoff: self.cutoff,
            field: self.field,
            requests,
        }
    }
    /// Execute with unlimited cooperative work and no cancellation flag.
    /// # Errors
    /// Rejects invalid parameters, insufficient source dimensions/range, numerical
    /// failures, checked size/allocation errors and unsatisfied representative requests.
    pub fn compute(self) -> Result<PersistenceResult> {
        self.compute_with(&Execution::default())
    }
    /// Execute with one budget spanning preparation, reduction and representatives.
    /// # Errors
    /// Includes [`Self::compute`] errors, cancellation and work exhaustion.
    pub fn compute_with(self, execution: &Execution<'_>) -> Result<PersistenceResult> {
        let options = PersistenceOptions::new(self.dimension, self.cutoff)?.with_field(self.field);
        for request in self.requests {
            if request.dimension() > self.dimension {
                return Err(Error::DimensionNotComputed {
                    requested: request.dimension(),
                    computed_max: self.dimension,
                });
            }
            // Coverage is source-dependent: a cap at/above the diameter can
            // still certify completeness, allowing later representative scales.
        }
        (self.compute)(
            self.source,
            &options,
            self.requests,
            &mut WorkBudget::new(execution)?,
        )
    }
}
