//! Owned provenance of a sparse Rips construction.
use crate::geometry::MetricValidation;

/// Exact Rips filtration to which the conditional approximation theorem applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApproximationTarget {
    /// Original input, with zero-distance duplicates identified.
    OriginalInput,
    /// Retained greedy prefix; positive-radius omissions require a separate bound.
    RetainedSubset,
}

/// Ideal-arithmetic sparse Rips bound; not a floating-point error certificate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RipsApproximationBound {
    pub(crate) factor: f64,
    pub(crate) target: ApproximationTarget,
}
impl RipsApproximationBound {
    /// Nominal binary64 value of the multiplicative factor `1 / (1 - epsilon)`.
    pub fn factor(&self) -> f64 {
        self.factor
    }
    /// Input against which the factor is stated.
    pub fn target(&self) -> ApproximationTarget {
        self.target
    }
}

/// Reproducible sparse Rips parameters and vertex provenance.
///
/// Bounds assume exact arithmetic and a metric. Checked binary64 inputs do not
/// certify rounding in the construction. Coverage belongs to the approximate
/// filtration, never to the original exact Rips filtration.
#[derive(Clone, Debug, PartialEq)]
pub struct RipsApproximation {
    pub(crate) epsilon: f64,
    pub(crate) permutation: Vec<usize>,
    pub(crate) insertion_radii: Vec<Option<f64>>,
    pub(crate) retained_vertices: Vec<usize>,
    pub(crate) min_insertion_radius: f64,
    pub(crate) max_scale: Option<f64>,
    pub(crate) metric: MetricValidation,
    pub(crate) covering_radius: f64,
}
impl RipsApproximation {
    /// Sparsification parameter, strictly positive; values at least one have no bound.
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }
    /// Complete deterministic farthest-point permutation in original vertex IDs.
    pub fn permutation(&self) -> &[usize] {
        &self.permutation
    }
    /// Insertion radii aligned with the permutation; the first `None` means infinity.
    pub fn insertion_radii(&self) -> &[Option<f64>] {
        &self.insertion_radii
    }
    /// Compact graph index to original vertex ID, in increasing original order.
    pub fn retained_vertices(&self) -> &[usize] {
        &self.retained_vertices
    }
    /// Minimum insertion radius; noninitial zero-radius vertices are always omitted.
    pub fn min_insertion_radius(&self) -> f64 {
        self.min_insertion_radius
    }
    /// Construction threshold in edge-length units.
    pub fn max_scale(&self) -> Option<f64> {
        self.max_scale
    }
    /// Evidence or caller assumption underlying the metric hypothesis.
    pub fn metric_validation(&self) -> MetricValidation {
        self.metric
    }
    /// Maximum original-to-retained nearest distance (zero for an empty input).
    pub fn covering_radius(&self) -> f64 {
        self.covering_radius
    }
    /// Conditional ideal-arithmetic bound, absent for unchecked inputs or epsilon >= 1.
    ///
    /// The factor uses GUDHI's edge-length convention and the metric sparse Rips
    /// theorem. Positive-radius subsampling changes the target to the retained
    /// subset; no multiplicative bound to the original input is claimed then.
    pub fn bound(&self) -> Option<RipsApproximationBound> {
        if self.epsilon >= 1.0 || self.metric == MetricValidation::Unchecked {
            return None;
        }
        Some(RipsApproximationBound {
            factor: 1.0 / (1.0 - self.epsilon),
            target: if self.covering_radius > 0.0 {
                ApproximationTarget::RetainedSubset
            } else {
                ApproximationTarget::OriginalInput
            },
        })
    }
}
