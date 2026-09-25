//! Sparse Rips construction, separate from exact thresholds and supplied flags.
mod access;
mod metadata;
pub use metadata::{ApproximationTarget, RipsApproximation, RipsApproximationBound};
mod blocker;
mod edges;
mod expansion;
mod greedy;
mod options;
use super::RipsInputKind;
use crate::complex::{WeightedEdge, WeightedGraph};
use crate::filtration::Coverage;
use crate::geometry::{
    DissimilarityMatrixView, MatrixLayout, PointCloudView, distance::nonnegative,
    euclidean_distance, validate_metric_with_checkpoints,
};
use crate::{Error, Result};
pub(crate) use access::SparseRipsAccess;
pub use expansion::SparseRipsExpansion;
pub use options::SparseRipsOptions;

/// A sparse Rips approximation graph together with its higher-simplex blocker.
///
/// Turning this graph into a supplied flag filtration loses the blocker and can
/// change persistence. Use the sparse Rips expansion and persistence entry points.
#[derive(Clone, Debug)]
pub struct SparseRips {
    pub(crate) graph: WeightedGraph,
    pub(crate) metadata: RipsApproximation,
    pub(crate) coverage: Coverage,
    pub(crate) kind: RipsInputKind,
    radii: Vec<Option<f64>>,
    factor: f64,
}
impl SparseRips {
    /// Borrow the compact graph. Map its IDs using `approximation().retained_vertices()`.
    pub fn graph(&self) -> &WeightedGraph {
        &self.graph
    }
    /// Complete sampling and parameter provenance.
    pub fn approximation(&self) -> &RipsApproximation {
        &self.metadata
    }
    /// Scale coverage of the approximate filtration, not of original exact Rips.
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }
    /// Original distance source kind.
    pub fn input_kind(&self) -> RipsInputKind {
        self.kind
    }
}

/// Construct sparse Rips from a borrowed matrix, without copying distances.
///
/// Uses O(n²) distance evaluations plus O(n³) if checking the metric. Sampling
/// uses O(n) auxiliary space; graph storage is O(n+m). No dimension expansion.
/// # Errors
/// Returns parameter, metric, allocation or numerical errors. Overflowing edge
/// intermediates are rejected instead of silently selecting a different topology.
pub fn sparse_rips_from_distances(
    input: DissimilarityMatrixView<'_>,
    options: &SparseRipsOptions,
) -> Result<SparseRips> {
    build(
        input.len(),
        options,
        RipsInputKind::Dissimilarities,
        |a, b| Ok(input.get(a, b).unwrap()),
    )
}

/// Construct sparse Rips from streamed Euclidean distances, without a dense matrix.
///
/// The explicit metric policy applies to the computed binary64 distances, which
/// need not satisfy every triangle exactly after rounding.
/// # Errors
/// Includes sparse construction errors and nonrepresentable Euclidean norms.
pub fn sparse_rips_from_points(
    input: PointCloudView<'_>,
    options: &SparseRipsOptions,
) -> Result<SparseRips> {
    build(input.len(), options, RipsInputKind::Euclidean, |a, b| {
        euclidean_distance(input, a, b)
    })
}

/// Construct from a callback sampled exactly once per unordered pair.
///
/// Evaluates `(points[a], points[b])` for `a < b`; symmetry and a zero diagonal
/// are declared by this API. Caches O(n²) distances so a stateful callback cannot
/// change the metric between sampling, validation and edge construction.
/// # Errors
/// Propagates callback errors and all matrix/sparse construction errors.
pub fn sparse_rips_with_distance<P>(
    points: &[P],
    options: &SparseRipsOptions,
    mut distance: impl FnMut(&P, &P) -> Result<f64>,
) -> Result<SparseRips> {
    let n = points.len();
    let count = crate::geometry::pair_count(n).ok_or(Error::SizeOverflow {
        operation: "sparse callback pair count",
    })?;
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| allocation())?;
    for b in 0..n {
        for a in 0..b {
            values.push(distance(&points[a], &points[b])?);
        }
    }
    let matrix = DissimilarityMatrixView::new(&values, n, MatrixLayout::LowerTriangle)?;
    build(n, options, RipsInputKind::Custom, |a, b| {
        Ok(matrix.get(a, b).unwrap())
    })
}

fn build(
    n: usize,
    options: &SparseRipsOptions,
    kind: RipsInputKind,
    mut distance: impl FnMut(usize, usize) -> Result<f64>,
) -> Result<SparseRips> {
    build_with(n, options, kind, &mut distance, &mut || Ok(()))
}

pub(super) fn build_with(
    n: usize,
    options: &SparseRipsOptions,
    kind: RipsInputKind,
    mut distance: impl FnMut(usize, usize) -> Result<f64>,
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<SparseRips> {
    if options.start.is_some_and(|v| v >= n) {
        return Err(Error::InvalidParameter {
            parameter: "start_vertex",
            reason: "outside input vertices",
        });
    }
    checkpoint()?;
    // Validate even pairs subsequently removed by sampling or thresholds.
    for b in 0..n {
        for a in 0..b {
            checkpoint()?;
            nonnegative(distance(a, b)?, "sparse distance", None)?;
        }
    }
    let metric = validate_metric_with_checkpoints(n, options.policy, &mut distance, checkpoint)?;
    let greedy = greedy::permutation(n, options.start, &mut distance, checkpoint)?;
    let kept = greedy
        .radii
        .iter()
        .take_while(|r| r.is_none_or(|r| r > 0.0 && r >= options.min_radius))
        .count();
    let covering_radius = greedy.radii.get(kept).copied().flatten().unwrap_or(0.0);
    let mut retained_vertices = Vec::new();
    retained_vertices
        .try_reserve_exact(kept)
        .map_err(|_| allocation())?;
    retained_vertices.extend_from_slice(&greedy.order[..kept]);
    retained_vertices.sort_unstable();
    let mut radii = Vec::new();
    radii.try_reserve_exact(kept).map_err(|_| allocation())?;
    radii.resize(kept, None);
    for (&v, &r) in greedy.order[..kept].iter().zip(&greedy.radii) {
        radii[retained_vertices.binary_search(&v).unwrap()] = r;
    }
    let mut graph_edges = Vec::new();
    let mut omitted_by_scale = false;
    for j in 1..kept {
        for i in 0..j {
            checkpoint()?;
            let a = greedy.order[i];
            let b = greedy.order[j];
            if let Some(value) = edges::value(
                distance(a, b)?,
                greedy.radii[i],
                greedy.radii[j].unwrap(),
                options.epsilon,
            )? {
                if options.max_scale.is_some_and(|t| value > t) {
                    omitted_by_scale = true;
                    continue;
                }
                graph_edges.try_reserve(1).map_err(|_| allocation())?;
                graph_edges.push(WeightedEdge {
                    vertices: [
                        retained_vertices.binary_search(&a).unwrap(),
                        retained_vertices.binary_search(&b).unwrap(),
                    ],
                    value,
                });
            }
        }
    }
    let coverage = if omitted_by_scale {
        Coverage::Through(options.max_scale.unwrap())
    } else {
        Coverage::Complete
    };
    checkpoint()?;
    let graph = WeightedGraph::new(kept, graph_edges)?;
    checkpoint()?;
    Ok(SparseRips {
        graph,
        coverage,
        kind,
        radii,
        factor: blocker::factor(options.epsilon),
        metadata: RipsApproximation {
            epsilon: options.epsilon,
            permutation: greedy.order,
            insertion_radii: greedy.radii,
            retained_vertices,
            min_insertion_radius: options.min_radius,
            max_scale: options.max_scale,
            metric,
            covering_radius,
        },
    })
}
fn allocation() -> Error {
    Error::AllocationFailed {
        context: "sparse Rips construction",
    }
}

mod builder;
pub use builder::{ApproximateRipsBuilder, ApproximateRipsCallbackBuilder};
