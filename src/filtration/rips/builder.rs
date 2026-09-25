//! Exact construction requests; expensive work starts only at a terminal.
use super::{RipsInputKind, ThresholdRips, exact};
use crate::Result;
use crate::execution::{Execution, WorkBudget};
use crate::filtration::SimplicialFiltration;
use crate::geometry::{DissimilarityMatrixView, PointCloudView, euclidean_distance};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Input<'a> {
    Points(PointCloudView<'a>),
    Matrix(DissimilarityMatrixView<'a>),
}
impl Input<'_> {
    pub(crate) fn len(self) -> usize {
        match self {
            Self::Points(p) => p.len(),
            Self::Matrix(m) => m.len(),
        }
    }
    pub(crate) fn kind(self) -> RipsInputKind {
        match self {
            Self::Points(_) => RipsInputKind::Euclidean,
            Self::Matrix(_) => RipsInputKind::Dissimilarities,
        }
    }
    pub(crate) fn distance(self, a: usize, b: usize) -> Result<f64> {
        match self {
            Self::Points(p) => euclidean_distance(p, a, b),
            Self::Matrix(m) => Ok(m.get(a, b).unwrap()),
        }
    }
}

/// Reusable exact Rips construction settings borrowing validated input.
///
/// Setters perform no pairwise work. [`Self::build_complex`] produces stored
/// simplices; import [`crate::persistence::PersistenceExt`] for direct analysis.
/// No simplex query is available on this unexpanded request.
///
/// ```
/// use cocycle::filtration::RipsBuilder;
/// use cocycle::geometry::PointCloudView;
/// use cocycle::persistence::PersistenceExt;
/// let coordinates = [0., 0., 1., 0., 1., 1., 0., 1.];
/// let points = PointCloudView::new(&coordinates, 4, 2)?;
/// let rips = RipsBuilder::from_points(points).max_edge_length(1.0);
/// let filtration = rips.build_complex(2)?;
/// assert!(filtration.complex().find(&[0, 1]).is_some());
/// let result = rips.persistence().compute()?;
/// assert_eq!(result.diagram().intervals_in_dimension(1)?.count(), 1);
/// # Ok::<(), cocycle::Error>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct RipsBuilder<'a> {
    pub(crate) input: Input<'a>,
    pub(crate) max_edge: Option<f64>,
}
impl<'a> RipsBuilder<'a> {
    /// Borrow Euclidean points, retaining duplicate points and original IDs.
    pub fn from_points(points: PointCloudView<'a>) -> Self {
        Self {
            input: Input::Points(points),
            max_edge: None,
        }
    }
    /// Borrow any validated matrix layout without copying or requiring a metric.
    pub fn from_distance_matrix(matrix: DissimilarityMatrixView<'a>) -> Self {
        Self {
            input: Input::Matrix(matrix),
            max_edge: None,
        }
    }
    /// Own a one-shot symmetric callback, evaluated once per unordered pair.
    /// Symmetry and zero self-distance are caller assumptions; no diagonal is read.
    pub fn from_distance_fn<P, F>(items: &'a [P], distance: F) -> RipsCallbackBuilder<'a, P, F>
    where
        F: FnMut(&P, &P) -> Result<f64>,
    {
        RipsCallbackBuilder {
            items,
            distance,
            max_edge: None,
        }
    }
    /// Set an inclusive original edge-length cutoff; validated at execution.
    pub fn max_edge_length(mut self, value: f64) -> Self {
        self.max_edge = Some(value);
        self
    }
    /// Remove the construction cutoff.
    pub fn full_range(mut self) -> Self {
        self.max_edge = None;
        self
    }
    /// Prepare an owned graph for repeated operations, without expanding simplices.
    /// All pairs are visited; retained storage is O(n+m).
    /// # Errors
    /// Returns invalid-cutoff, distance, size or allocation errors.
    pub fn prepare(&self) -> Result<ThresholdRips> {
        self.prepare_with(&Execution::default())
    }
    /// Prepare with cooperative controls covering the entire construction.
    /// # Errors
    /// Includes [`Self::prepare`] errors, cancellation and work exhaustion.
    pub fn prepare_with(&self, execution: &Execution<'_>) -> Result<ThresholdRips> {
        self.validate()?;
        self.prepare_budget(&mut WorkBudget::new(execution)?)
    }
    /// Build stored simplices and incidence through the inclusive simplex dimension.
    /// Zero includes only vertices. Storage can be exponential.
    /// # Errors
    /// Includes preparation and expansion allocation/index errors.
    pub fn build_complex(&self, max_simplex_dimension: usize) -> Result<SimplicialFiltration> {
        self.build_complex_with(max_simplex_dimension, &Execution::default())
    }
    /// Build with one budget shared by preparation, expansion and incidence storage.
    /// # Errors
    /// Includes [`Self::build_complex`] errors, cancellation and work exhaustion.
    pub fn build_complex_with(
        &self,
        max_simplex_dimension: usize,
        execution: &Execution<'_>,
    ) -> Result<SimplicialFiltration> {
        self.validate()?;
        let mut budget = WorkBudget::new(execution)?;
        self.prepare_budget(&mut budget)?
            .build_complex_budget(max_simplex_dimension, &mut budget)
    }
    pub(crate) fn validate(&self) -> Result<()> {
        crate::geometry::distance::cutoff(self.max_edge).map(|_| ())
    }
    pub(crate) fn prepare_budget(&self, budget: &mut WorkBudget<'_>) -> Result<ThresholdRips> {
        exact::build_with(
            self.input.len(),
            self.max_edge,
            self.input.kind(),
            |a, b| self.input.distance(a, b),
            &mut || budget.step(),
        )
    }
}

/// One-shot exact construction owning a possibly stateful distance callback.
/// Terminals consume this builder; analysis borrows its prepared result instead.
pub struct RipsCallbackBuilder<'a, P, F> {
    items: &'a [P],
    distance: F,
    max_edge: Option<f64>,
}
impl<P, F: FnMut(&P, &P) -> Result<f64>> RipsCallbackBuilder<'_, P, F> {
    /// Set the inclusive original edge cutoff, validated before any callback call.
    pub fn max_edge_length(mut self, value: f64) -> Self {
        self.max_edge = Some(value);
        self
    }
    /// Remove the construction cutoff.
    pub fn full_range(mut self) -> Self {
        self.max_edge = None;
        self
    }
    /// Evaluate every unordered pair once and retain an owned threshold graph.
    /// # Errors
    /// Propagates callback, numeric, size and allocation errors without partial output.
    pub fn prepare(self) -> Result<ThresholdRips> {
        self.prepare_with(&Execution::default())
    }
    /// Prepare under cooperative controls, polling around callbacks.
    /// # Errors
    /// Includes [`Self::prepare`] errors, cancellation and work exhaustion.
    pub fn prepare_with(self, execution: &Execution<'_>) -> Result<ThresholdRips> {
        crate::geometry::distance::cutoff(self.max_edge)?;
        self.prepare_budget(&mut WorkBudget::new(execution)?)
    }
    /// Consume the callback and build explicit topology through the given dimension.
    /// # Errors
    /// Includes callback/preparation and expansion errors.
    pub fn build_complex(self, max_simplex_dimension: usize) -> Result<SimplicialFiltration> {
        self.build_complex_with(max_simplex_dimension, &Execution::default())
    }
    /// Build with a single budget spanning callbacks and expansion.
    /// # Errors
    /// Includes [`Self::build_complex`] errors, cancellation and work exhaustion.
    pub fn build_complex_with(
        self,
        max_simplex_dimension: usize,
        execution: &Execution<'_>,
    ) -> Result<SimplicialFiltration> {
        crate::geometry::distance::cutoff(self.max_edge)?;
        let mut budget = WorkBudget::new(execution)?;
        self.prepare_budget(&mut budget)?
            .build_complex_budget(max_simplex_dimension, &mut budget)
    }
    fn prepare_budget(mut self, budget: &mut WorkBudget<'_>) -> Result<ThresholdRips> {
        exact::build_with(
            self.items.len(),
            self.max_edge,
            RipsInputKind::Custom,
            |a, b| (self.distance)(&self.items[a], &self.items[b]),
            &mut || budget.step(),
        )
    }
}
