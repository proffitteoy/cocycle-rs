//! Approximate construction settings and single-evaluation callback ownership.
use super::{SparseRips, SparseRipsOptions, build_with};
use crate::execution::{Execution, WorkBudget};
use crate::filtration::rips::builder::Input;
use crate::filtration::{RipsInputKind, SimplicialFiltration};
use crate::geometry::{DissimilarityMatrixView, MatrixLayout, MetricPolicy, PointCloudView};
use crate::{Error, Result};

#[derive(Clone, Copy, Debug)]
struct Settings {
    epsilon: f64,
    policy: MetricPolicy,
    start: Option<usize>,
    min_radius: f64,
    max_scale: Option<f64>,
}
impl Settings {
    fn new(epsilon: f64, policy: MetricPolicy) -> Self {
        Self {
            epsilon,
            policy,
            start: None,
            min_radius: 0.0,
            max_scale: None,
        }
    }
    fn validate(self, n: usize) -> Result<SparseRipsOptions> {
        let mut options = SparseRipsOptions::new(self.epsilon, self.policy)?
            .with_min_insertion_radius(self.min_radius)?
            .with_max_scale(self.max_scale)?;
        if let Some(start) = self.start {
            if start >= n {
                return Err(Error::InvalidParameter {
                    parameter: "start_vertex",
                    reason: "outside input vertices",
                });
            }
            options = options.with_start_vertex(start);
        }
        Ok(options)
    }
}

/// Reusable sparse Rips approximation request with explicit metric hypotheses.
///
/// No sampling occurs before a terminal. Both explicit construction and direct
/// analysis apply the higher-simplex blocker; sparse input storage alone never
/// selects approximation. Epsilon >= 1 has no approximation bound.
#[derive(Clone, Copy, Debug)]
pub struct ApproximateRipsBuilder<'a> {
    input: Input<'a>,
    settings: Settings,
}
impl<'a> ApproximateRipsBuilder<'a> {
    /// Borrow Euclidean points and select epsilon and metric evidence policy.
    pub fn from_points(points: PointCloudView<'a>, epsilon: f64, policy: MetricPolicy) -> Self {
        Self {
            input: Input::Points(points),
            settings: Settings::new(epsilon, policy),
        }
    }
    /// Borrow a validated matrix in any layout; metric policy remains explicit.
    pub fn from_distance_matrix(
        matrix: DissimilarityMatrixView<'a>,
        epsilon: f64,
        policy: MetricPolicy,
    ) -> Self {
        Self {
            input: Input::Matrix(matrix),
            settings: Settings::new(epsilon, policy),
        }
    }
    /// Own a one-shot callback whose O(n²) cached samples are shared by all phases.
    pub fn from_distance_fn<P, F>(
        items: &'a [P],
        distance: F,
        epsilon: f64,
        policy: MetricPolicy,
    ) -> ApproximateRipsCallbackBuilder<'a, P, F>
    where
        F: FnMut(&P, &P) -> Result<f64>,
    {
        ApproximateRipsCallbackBuilder {
            items,
            distance,
            settings: Settings::new(epsilon, policy),
        }
    }
    /// Select the deterministic first vertex; default is zero for nonempty input.
    pub fn start_vertex(mut self, vertex: usize) -> Self {
        self.settings.start = Some(vertex);
        self
    }
    /// Omit noninitial vertices below this insertion radius; validated at execution.
    pub fn min_insertion_radius(mut self, radius: f64) -> Self {
        self.settings.min_radius = radius;
        self
    }
    /// Cap modified filtration values, not original edge distances.
    pub fn max_filtration_value(mut self, value: f64) -> Self {
        self.settings.max_scale = Some(value);
        self
    }
    /// Remove the construction scale cap.
    pub fn full_range(mut self) -> Self {
        self.settings.max_scale = None;
        self
    }
    /// Prepare owned graph, blocker and sampling metadata for reuse.
    /// # Errors
    /// Returns parameter, metric, numeric, size or allocation errors.
    pub fn prepare(&self) -> Result<SparseRips> {
        self.prepare_with(&Execution::default())
    }
    /// Prepare with one budget spanning validation, sampling and graph construction.
    /// # Errors
    /// Includes [`Self::prepare`] errors, cancellation and work exhaustion.
    pub fn prepare_with(&self, execution: &Execution<'_>) -> Result<SparseRips> {
        self.validate()?;
        self.prepare_budget(&mut WorkBudget::new(execution)?)
    }
    /// Build explicit blocker-aware topology through the inclusive dimension.
    /// # Errors
    /// Includes preparation and expansion allocation/index errors.
    pub fn build_complex(&self, max_simplex_dimension: usize) -> Result<SimplicialFiltration> {
        self.build_complex_with(max_simplex_dimension, &Execution::default())
    }
    /// Share one budget across preparation and explicit expansion.
    /// # Errors
    /// Includes [`Self::build_complex`] errors, cancellation and work exhaustion.
    pub fn build_complex_with(
        &self,
        dimension: usize,
        execution: &Execution<'_>,
    ) -> Result<SimplicialFiltration> {
        self.validate()?;
        let mut budget = WorkBudget::new(execution)?;
        self.prepare_budget(&mut budget)?
            .build_complex_budget(dimension, &mut budget)
    }
    pub(crate) fn validate(&self) -> Result<()> {
        self.settings.validate(self.input.len()).map(|_| ())
    }
    pub(crate) fn prepare_budget(&self, budget: &mut WorkBudget<'_>) -> Result<SparseRips> {
        let options = self.settings.validate(self.input.len())?;
        build_with(
            self.input.len(),
            &options,
            self.input.kind(),
            |a, b| self.input.distance(a, b),
            &mut || budget.step(),
        )
    }
}

/// One-shot approximate construction; every unordered callback pair is cached once.
/// No analysis extension or automatic cloning is provided for a stateful callback.
pub struct ApproximateRipsCallbackBuilder<'a, P, F> {
    items: &'a [P],
    distance: F,
    settings: Settings,
}
impl<P, F: FnMut(&P, &P) -> Result<f64>> ApproximateRipsCallbackBuilder<'_, P, F> {
    /// Select the first original vertex, validated before evaluating the callback.
    pub fn start_vertex(mut self, vertex: usize) -> Self {
        self.settings.start = Some(vertex);
        self
    }
    /// Select a nonnegative finite insertion-radius threshold.
    pub fn min_insertion_radius(mut self, radius: f64) -> Self {
        self.settings.min_radius = radius;
        self
    }
    /// Cap modified filtration values, not original pair distances.
    pub fn max_filtration_value(mut self, value: f64) -> Self {
        self.settings.max_scale = Some(value);
        self
    }
    /// Remove the construction scale cap.
    pub fn full_range(mut self) -> Self {
        self.settings.max_scale = None;
        self
    }
    /// Sample once, validate metric policy, and construct the owned approximation.
    /// # Errors
    /// Propagates callback errors and rejects invalid parameters, distances or metrics.
    pub fn prepare(self) -> Result<SparseRips> {
        self.prepare_with(&Execution::default())
    }
    /// Prepare with cooperative controls around callbacks and subsequent phases.
    /// # Errors
    /// Includes [`Self::prepare`] errors, cancellation and work exhaustion.
    pub fn prepare_with(self, execution: &Execution<'_>) -> Result<SparseRips> {
        self.settings.validate(self.items.len())?;
        self.prepare_budget(&mut WorkBudget::new(execution)?)
    }
    /// Consume the callback and build stored blocker-aware simplices.
    /// # Errors
    /// Includes callback, preparation and expansion errors.
    pub fn build_complex(self, max_simplex_dimension: usize) -> Result<SimplicialFiltration> {
        self.build_complex_with(max_simplex_dimension, &Execution::default())
    }
    /// Share one budget across cached callback input, preparation and expansion.
    /// # Errors
    /// Includes [`Self::build_complex`] errors, cancellation and work exhaustion.
    pub fn build_complex_with(
        self,
        dimension: usize,
        execution: &Execution<'_>,
    ) -> Result<SimplicialFiltration> {
        self.settings.validate(self.items.len())?;
        let mut budget = WorkBudget::new(execution)?;
        self.prepare_budget(&mut budget)?
            .build_complex_budget(dimension, &mut budget)
    }
    fn prepare_budget(mut self, budget: &mut WorkBudget<'_>) -> Result<SparseRips> {
        let n = self.items.len();
        let options = self.settings.validate(n)?;
        let count = crate::geometry::pair_count(n).ok_or(Error::SizeOverflow {
            operation: "callback pair count",
        })?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| super::allocation())?;
        for b in 0..n {
            for a in 0..b {
                budget.step()?;
                values.push(crate::geometry::distance::nonnegative(
                    (self.distance)(&self.items[a], &self.items[b])?,
                    "distance callback",
                    None,
                )?);
                budget.check()?;
            }
        }
        let matrix = DissimilarityMatrixView::new(&values, n, MatrixLayout::LowerTriangle)?;
        budget.check()?;
        build_with(
            n,
            &options,
            RipsInputKind::Custom,
            |a, b| Ok(matrix.get(a, b).unwrap()),
            &mut || budget.step(),
        )
    }
}
