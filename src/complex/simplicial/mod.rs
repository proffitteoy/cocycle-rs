//! Frozen, face-closed simplicial topology with oriented incidence.
mod incidence;
mod simplex;

use crate::{Error, Result};
pub use incidence::BoundaryTerm;
pub(crate) use simplex::compare_filtration;
pub use simplex::{Simplex, SimplexId};
use std::collections::HashMap;

/// An immutable simplicial complex storing finite filtration values.
///
/// Simplices are face-closed and ordered by value, dimension, then decreasing
/// colexicographic order. Incidence is stored in both directions. Storage is
/// proportional to the simplices and their codimension-one incidences; clique
/// expansion can be exponential in the number of input vertices.
#[derive(Clone, Debug)]
pub struct SimplicialComplex {
    simplices: Vec<Simplex>,
    lookup: HashMap<Vec<usize>, SimplexId>,
    boundaries: Vec<Vec<BoundaryTerm>>,
    cofacets: Vec<Vec<SimplexId>>,
    vertex_count: usize,
    dimension: Option<usize>,
    zero_born: bool,
}
impl SimplicialComplex {
    /// Validate and own a face-closed collection of filtered simplices.
    /// Input order is arbitrary; output is in filtration order. All faces must
    /// be provided, with values no larger than their cofaces. No faces are inferred.
    /// # Errors
    /// Rejects duplicate simplices, missing faces, invalid filtrations and allocation failure.
    pub fn new(simplices: Vec<Simplex>) -> Result<Self> {
        Self::new_with(simplices, &crate::execution::Execution::default())
    }
    /// Construct under one cooperative validation and incidence budget.
    /// # Errors
    /// Includes [`Self::new`] errors, cancellation and work exhaustion.
    pub fn new_with(
        simplices: Vec<Simplex>,
        execution: &crate::execution::Execution<'_>,
    ) -> Result<Self> {
        let mut budget = crate::execution::WorkBudget::new(execution)?;
        Self::from_simplices(simplices, &mut || budget.step())
    }
    /// Largest stored filtration value, or `None` for an empty complex.
    pub fn max_filtration_value(&self) -> Option<f64> {
        self.simplices.last().map(Simplex::value)
    }
    /// Number of stored vertices, independent of their original IDs and birth order.
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    pub(crate) fn has_zero_born_vertices(&self) -> bool {
        self.zero_born
    }

    pub(crate) fn from_simplices(
        mut simplices: Vec<Simplex>,
        checkpoint: &mut impl FnMut() -> Result<()>,
    ) -> Result<Self> {
        checkpoint()?;
        simplices.sort_unstable();
        let mut lookup = HashMap::new();
        lookup
            .try_reserve(simplices.len())
            .map_err(|_| allocation())?;
        let mut boundaries = Vec::new();
        let mut cofacets: Vec<Vec<SimplexId>> = Vec::new();
        boundaries
            .try_reserve_exact(simplices.len())
            .map_err(|_| allocation())?;
        cofacets
            .try_reserve_exact(simplices.len())
            .map_err(|_| allocation())?;
        let mut vertex_count = 0;
        let mut dimension = None;
        let mut zero_born = true;
        for (position, simplex) in simplices.iter().enumerate() {
            checkpoint()?;
            dimension = dimension.max(Some(simplex.dimension()));
            if simplex.dimension() == 0 {
                vertex_count += 1;
                zero_born &= simplex.value() == 0.;
            }
            let id = SimplexId(position);
            let mut boundary = Vec::new();
            if simplex.dimension() > 0 {
                boundary
                    .try_reserve_exact(simplex.vertices.len())
                    .map_err(|_| allocation())?;
                for omitted in 0..simplex.vertices.len() {
                    checkpoint()?;
                    let mut vertices = simplex.vertices.clone();
                    vertices.remove(omitted);
                    let &face = lookup.get(&vertices).ok_or(Error::InvalidComplex {
                        cell: Some(position),
                        reason: "simplex face missing or later in filtration",
                    })?;
                    boundary.push(BoundaryTerm {
                        face,
                        coefficient: if omitted % 2 == 0 { 1 } else { -1 },
                    });
                    cofacets[face.0].try_reserve(1).map_err(|_| allocation())?;
                    cofacets[face.0].push(id);
                }
            }
            if lookup.insert(simplex.vertices.clone(), id).is_some() {
                return Err(Error::InvalidComplex {
                    cell: Some(position),
                    reason: "duplicate simplex",
                });
            }
            boundaries.push(boundary);
            cofacets.push(Vec::new());
        }
        Ok(Self {
            simplices,
            lookup,
            boundaries,
            cofacets,
            vertex_count,
            dimension,
            zero_born,
        })
    }
    /// Simplices in filtration order. The slice position equals the simplex ID.
    pub fn simplices(&self) -> &[Simplex] {
        &self.simplices
    }
    /// Number of stored nonempty simplices.
    pub fn len(&self) -> usize {
        self.simplices.len()
    }
    /// Whether there are no vertices (and hence no simplices).
    pub fn is_empty(&self) -> bool {
        self.simplices.is_empty()
    }
    /// Largest stored dimension, or `None` for an empty complex.
    pub fn dimension(&self) -> Option<usize> {
        self.dimension
    }
    /// Find a nonempty simplex by strictly increasing vertices; invalid order returns `None`.
    pub fn find(&self, vertices: &[usize]) -> Option<SimplexId> {
        self.lookup.get(vertices).copied()
    }
    /// Borrow a simplex at a local ID, or `None` for an out-of-range ID.
    pub fn simplex(&self, id: SimplexId) -> Option<&Simplex> {
        self.simplices.get(id.0)
    }
    /// Oriented codimension-one faces, in omitted-vertex order. Vertices have empty boundaries.
    pub fn boundary(&self, id: SimplexId) -> Option<&[BoundaryTerm]> {
        self.boundaries.get(id.0).map(Vec::as_slice)
    }
    /// Codimension-one cofaces in filtration order; includes only stored simplices.
    pub fn cofacets(&self, id: SimplexId) -> Option<&[SimplexId]> {
        self.cofacets.get(id.0).map(Vec::as_slice)
    }
}
fn allocation() -> Error {
    Error::AllocationFailed {
        context: "simplicial incidence",
    }
}
