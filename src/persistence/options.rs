//! Mathematical options shared by exact Rips and supplied flag computation.
use crate::algebra::PrimeField;
use crate::filtration::Coverage;
use crate::geometry::distance::cutoff;
use crate::{Error, Result};

/// Ordinary prime-field persistence options in arbitrary homology dimensions.
///
/// The default coefficient field is F2.
/// `None` uses the supplied source's available scale range; a threshold Rips
/// source may therefore yield a censored diagram. No approximation is selected.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PersistenceOptions {
    max_homology_dimension: usize,
    max_edge: Option<f64>,
    field: PrimeField,
}
impl PersistenceOptions {
    /// Validate the maximum homology dimension and optional inclusive edge cutoff.
    ///
    /// # Errors
    /// Rejects non-finite or negative cutoffs. Dimensions above the vertex count
    /// are valid and have no nonzero homology.
    pub fn new(max_homology_dimension: usize, max_edge: Option<f64>) -> Result<Self> {
        Ok(Self {
            max_homology_dimension,
            max_edge: cutoff(max_edge)?,
            field: PrimeField::default(),
        })
    }
    pub(super) fn for_filtration(
        max_homology_dimension: usize,
        value: Option<f64>,
    ) -> Result<Self> {
        Ok(Self {
            max_homology_dimension,
            max_edge: value.map(finite_scale).transpose()?,
            field: PrimeField::default(),
        })
    }
    /// Select a validated prime coefficient field; all other options are preserved.
    pub fn with_field(mut self, field: PrimeField) -> Self {
        self.field = field;
        self
    }
    /// Coefficient field used for both persistence and requested representatives.
    pub fn field(&self) -> PrimeField {
        self.field
    }
    /// Largest requested homology dimension, including all smaller ones.
    pub fn max_homology_dimension(&self) -> usize {
        self.max_homology_dimension
    }
    /// Requested scale cutoff, or the source's available range.
    pub fn max_edge(&self) -> Option<f64> {
        self.max_edge
    }
}
impl Default for PersistenceOptions {
    fn default() -> Self {
        Self {
            max_homology_dimension: 1,
            max_edge: None,
            field: PrimeField::default(),
        }
    }
}

// Preserve original source coverage instead of inferring it from retained edges.
pub(super) fn source_range(
    coverage: Coverage,
    max_edge: f64,
    requested: Option<f64>,
) -> Result<(f64, Coverage)> {
    match coverage {
        Coverage::Through(through) => {
            let cutoff = requested.unwrap_or(through);
            if cutoff > through {
                return Err(Error::IncompleteFiltration {
                    requested: cutoff,
                    through,
                });
            }
            Ok((cutoff, Coverage::Through(cutoff)))
        }
        Coverage::Complete => match requested {
            Some(cutoff) if cutoff < max_edge => Ok((cutoff, Coverage::Through(cutoff))),
            _ => Ok((max_edge, Coverage::Complete)),
        },
    }
}

pub(super) fn finite_scale(value: f64) -> Result<f64> {
    if !value.is_finite() {
        return Err(Error::NonFiniteValue {
            field: "filtration scale",
            index: None,
        });
    }
    Ok(crate::canonical_zero(value))
}
