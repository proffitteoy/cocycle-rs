//! The read-only filtered-cell contract used by boundary reduction.
use super::{SimplexId, SimplicialComplex};

/// A finite filtered cell complex with stable handles and oriented boundaries.
///
/// Cells must be unique and ordered by nondecreasing finite filtration value;
/// every boundary cell precedes its coface, including ties. Boundary terms have
/// dimension one less, reference existing cells, and satisfy boundary-of-boundary
/// zero over the integers. Values may be negative and vertices need not be born
/// together. IDs need not be contiguous or encode vertices.
///
/// Implementations own validation of this mathematical contract. The generic
/// persistence path additionally checks traversal/face order and boundary-square
/// zero over the selected field. This is a safe trait: violations may return an
/// error or an incorrect result, but never justify unsafe memory access.
///
/// This contract describes the supplied complex, not its relationship to a larger
/// Rips/Alpha source. Source coverage and construction-dimension certificates live
/// outside it. Operations borrow stable storage; expensive lazy construction and
/// mutation do not belong in these accessors. Cancellation cannot interrupt the
/// interior of an accessor or iterator's `next` call.
pub trait FilteredComplex {
    /// Stable identity of one cell for the duration of a computation.
    type CellId: Copy + Eq + std::hash::Hash;
    /// Cells in filtration order, with boundary cells before cofaces.
    fn cells(&self) -> impl Iterator<Item = Self::CellId> + '_;
    /// Dimension of a valid cell returned by [`Self::cells`].
    fn dimension(&self, cell: Self::CellId) -> usize;
    /// Finite filtration value of a valid cell.
    fn filtration_value(&self, cell: Self::CellId) -> f64;
    /// Oriented codimension-one boundary with integer incidence coefficients.
    /// Duplicate terms are permitted and are summed; zero terms have no effect.
    fn boundary(&self, cell: Self::CellId) -> impl Iterator<Item = (Self::CellId, i32)> + '_;
}

impl FilteredComplex for SimplicialComplex {
    type CellId = SimplexId;
    fn cells(&self) -> impl Iterator<Item = SimplexId> + '_ {
        (0..self.len()).map(SimplexId)
    }
    fn dimension(&self, cell: SimplexId) -> usize {
        self.simplex(cell).expect("valid simplex ID").dimension()
    }
    fn filtration_value(&self, cell: SimplexId) -> f64 {
        self.simplex(cell).expect("valid simplex ID").value()
    }
    fn boundary(&self, cell: SimplexId) -> impl Iterator<Item = (SimplexId, i32)> + '_ {
        self.boundary(cell)
            .expect("valid simplex ID")
            .iter()
            .map(|term| (term.face, i32::from(term.coefficient)))
    }
}
