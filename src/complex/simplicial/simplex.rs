//! Canonical simplices and their filtration order.
use std::cmp::Ordering;

/// A nonempty simplex with increasing vertex indices and a finite filtration value.
///
/// Values are immutable. Order is increasing value, then dimension, then
/// decreasing colexicographic vertex order. Faces precede their cofaces.
#[derive(Clone, Debug)]
pub struct Simplex {
    pub(crate) vertices: Vec<usize>,
    pub(crate) value: f64,
}
impl Simplex {
    /// Create a nonempty simplex with strictly increasing vertex IDs.
    /// Negative finite filtration values are allowed; no faces are inserted.
    /// # Errors
    /// Rejects empty, repeated/unsorted vertices and nonfinite values.
    pub fn new(vertices: Vec<usize>, value: f64) -> crate::Result<Self> {
        if vertices.is_empty() || vertices.windows(2).any(|p| p[0] >= p[1]) {
            return Err(crate::Error::InvalidComplex {
                cell: None,
                reason: "vertices must be nonempty and strictly increasing",
            });
        }
        if !value.is_finite() {
            return Err(crate::Error::NonFiniteValue {
                field: "filtration value",
                index: None,
            });
        }
        Ok(Self {
            vertices,
            value: crate::canonical_zero(value),
        })
    }

    /// Increasing original vertex indices.
    pub fn vertices(&self) -> &[usize] {
        &self.vertices
    }
    /// Number of vertices minus one.
    pub fn dimension(&self) -> usize {
        self.vertices.len() - 1
    }
    /// Filtration entry value, in the source's declared units.
    pub fn value(&self) -> f64 {
        self.value
    }
}
impl PartialEq for Simplex {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.vertices == other.vertices
    }
}
impl Eq for Simplex {}
impl PartialOrd for Simplex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Simplex {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_filtration(
            self.value,
            other.value,
            self.dimension().cmp(&other.dimension()),
            other.vertices.iter().rev().cmp(self.vertices.iter().rev()),
        )
    }
}

/// Position in one frozen complex's filtration order. IDs are local to that complex.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimplexId(pub(crate) usize);
impl SimplexId {
    /// Zero-based position in filtration order.
    pub fn index(self) -> usize {
        self.0
    }
}

/// Shared ordering authority for tuple keys and compact H1 combinatorial IDs.
/// The final argument compares decreasing colex order within one dimension.
pub(crate) fn compare_filtration(
    left: f64,
    right: f64,
    dimensions: Ordering,
    decreasing_colex: Ordering,
) -> Ordering {
    left.total_cmp(&right)
        .then(dimensions)
        .then(decreasing_colex)
}
