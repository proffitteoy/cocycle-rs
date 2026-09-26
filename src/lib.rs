//! Cocycle is a general-purpose topological data analysis library in pure Rust.
//!
//! Computes Vietoris-Rips persistent homology in arbitrary dimensions over prime fields
//! and descriptors and matching distances of persistence diagrams. Constructs exact
//! threshold graphs and computes supplied weighted flag filtrations with sparse
//! adjacency access and explicit original-input coverage. Accepts supplied
//! simplicial and filtered-cell complexes, including signed filtration values
//! and unequal vertex births. Algorithms are
//! implemented in Rust without a foreign TDA backend or runtime dependencies.
//!
//! # Status
//!
//! Implicit Rips computation uses a specialized F2 H0/H1 engine and prime-field
//! cohomology with clearing for other requests. Explicit builder results and
//! supplied complexes use boundary reduction. Rips expansion provides frozen
//! simplices and oriented incidence queries. Generic filtered-cell inputs return
//! diagrams; concrete simplicial sources also support vertex-labelled representatives.
//! Requested cycle/cocycle representatives use production boundary reduction;
//! independent test oracles verify ranks and interval semantics.
//! Work and memory can still grow substantially with input size and reduction
//! fill-in. See the mathematical specification and benchmark records for limits.
//!
//! ```
//! use cocycle::descriptors::betti_curve;
//! use cocycle::geometry::PointCloudView;
//! use cocycle::filtration::RipsBuilder;
//! use cocycle::persistence::PersistenceExt;
//!
//! let coordinates = [0., 0., 1., 0., 1., 1., 0., 1.];
//! let points = PointCloudView::new(&coordinates, 4, 2)?;
//! let result = RipsBuilder::from_points(points).persistence().compute()?;
//! let diagram = result.diagram();
//! assert_eq!(betti_curve(diagram, 1, &[0., 1., 2.])?, [0, 1, 0]);
//! # Ok::<(), cocycle::Error>(())
//! ```

pub mod algebra;
pub mod complex;
pub mod descriptors;
pub mod diagram;
pub mod diagram_distances;
mod error;
pub mod execution;
pub mod filtration;
pub mod geometry;
pub mod persistence;

pub use error::{Error, Result};

/// Canonicalize signed zero without changing any other finite value.
pub(crate) fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}
