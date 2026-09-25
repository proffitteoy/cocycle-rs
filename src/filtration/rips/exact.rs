//! Streamed exact Rips graph construction and original-input coverage.
use crate::complex::{WeightedEdge, WeightedGraph};
use crate::filtration::Coverage;
use crate::geometry::distance::{cutoff, nonnegative};
use crate::geometry::{DissimilarityMatrixView, PointCloudView, euclidean_distance};
use crate::{Error, Result};

/// How the exact Rips dissimilarities were supplied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RipsInputKind {
    /// An explicitly supplied symmetric dissimilarity matrix.
    Dissimilarities,
    /// Euclidean distances computed from coordinates using stable norms.
    Euclidean,
    /// Validated values sampled from a caller's pairwise callback.
    Custom,
}

/// Owned exact Rips graph and its certified range in the original filtration.
///
/// Vertex indices are unchanged. Edges omitted above the cutoff have unknown
/// later weights in this object. Only constructors that inspect every pair can
/// create this provenance. The graph accessor does not transfer that claim to
/// an arbitrary graph. No source buffer is retained.
#[derive(Clone, Debug)]
pub struct ThresholdRips {
    graph: WeightedGraph,
    coverage: Coverage,
    kind: RipsInputKind,
    requested_cutoff: Option<f64>,
}
impl ThresholdRips {
    /// Borrow the stored graph for inspection.
    pub fn graph(&self) -> &WeightedGraph {
        &self.graph
    }
    /// Range verified against the original pairwise input.
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }
    /// Distance source convention.
    pub fn input_kind(&self) -> RipsInputKind {
        self.kind
    }
    /// Construction cutoff requested by the caller, even when coverage is complete.
    pub fn requested_cutoff(&self) -> Option<f64> {
        self.requested_cutoff
    }
}

/// Build an exact graph from a validated matrix without copying its buffer.
///
/// All n(n-1)/2 pairs are inspected. Retained graph storage is O(n+m).
/// The cutoff is inclusive; `None` retains all pairs. Vertices are born at zero.
///
/// # Errors
/// Rejects an invalid cutoff, size overflow, or failed memory reservation.
pub fn threshold_rips_from_distances(
    input: DissimilarityMatrixView<'_>,
    max_edge: Option<f64>,
) -> Result<ThresholdRips> {
    build(
        input.len(),
        max_edge,
        RipsInputKind::Dissimilarities,
        |a, b| Ok(input.get(a, b).unwrap()),
    )
}

/// Build a Euclidean threshold graph without a dense intermediate matrix.
///
/// Retains original point indices, including coincident points. O(n^2 d)
/// distance work and O(n+m) retained graph storage; this is not a spatial index.
///
/// # Errors
/// Rejects an invalid cutoff, unrepresentable distance, size overflow or failed
/// memory reservation. No partial graph is returned.
pub fn threshold_rips_from_points(
    input: PointCloudView<'_>,
    max_edge: Option<f64>,
) -> Result<ThresholdRips> {
    build(input.len(), max_edge, RipsInputKind::Euclidean, |a, b| {
        euclidean_distance(input, a, b)
    })
}

/// Build from arbitrary borrowed points and a symmetric distance callback.
///
/// Calls `distance(&points[a], &points[b])` once per pair in lower-triangle
/// order, with `a < b`, never on the diagonal. Symmetry and zero self-distance
/// are caller assumptions; no triangle inequality is required. Every returned
/// value is validated, even above the cutoff. Points are neither cloned nor kept.
///
/// # Errors
/// Propagates callback errors and rejects non-finite/negative distances, invalid
/// cutoffs, size overflow or failed reservations. No partial graph is returned.
pub fn threshold_rips_with_distance<P>(
    points: &[P],
    max_edge: Option<f64>,
    mut distance: impl FnMut(&P, &P) -> Result<f64>,
) -> Result<ThresholdRips> {
    build(points.len(), max_edge, RipsInputKind::Custom, |a, b| {
        distance(&points[a], &points[b])
    })
}

fn build(
    n: usize,
    max_edge: Option<f64>,
    kind: RipsInputKind,
    mut distance: impl FnMut(usize, usize) -> Result<f64>,
) -> Result<ThresholdRips> {
    build_with(n, max_edge, kind, &mut distance, &mut || Ok(()))
}

pub(super) fn build_with(
    n: usize,
    max_edge: Option<f64>,
    kind: RipsInputKind,
    mut distance: impl FnMut(usize, usize) -> Result<f64>,
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<ThresholdRips> {
    let max_edge = cutoff(max_edge)?;
    n.checked_add(1).ok_or(Error::SizeOverflow {
        operation: "graph offsets",
    })?;
    checkpoint()?;
    let mut diameter: f64 = 0.0;
    let mut edges = Vec::new();
    for b in 0..n {
        for a in 0..b {
            checkpoint()?;
            let value = nonnegative(distance(a, b)?, "distance callback", None)?;
            checkpoint()?;
            diameter = diameter.max(value);
            if max_edge.is_none_or(|t| value <= t) {
                edges.try_reserve(1).map_err(|_| Error::AllocationFailed {
                    context: "threshold graph edges",
                })?;
                edges.push(WeightedEdge {
                    vertices: [a, b],
                    value,
                });
            }
        }
    }
    let coverage = match max_edge {
        Some(t) if t < diameter => Coverage::Through(t),
        _ => Coverage::Complete,
    };
    checkpoint()?;
    let graph = WeightedGraph::new(n, edges)?;
    checkpoint()?;
    Ok(ThresholdRips {
        graph,
        coverage,
        kind,
        requested_cutoff: max_edge,
    })
}

pub(crate) fn cone_radius(
    input: DissimilarityMatrixView<'_>,
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<f64> {
    let mut radius: f64 = input.diameter();
    for a in 0..input.len() {
        let mut row: f64 = 0.0;
        for b in 0..input.len() {
            checkpoint()?;
            row = row.max(input.get(a, b).unwrap());
        }
        radius = radius.min(row);
    }
    Ok(radius)
}
