//! Forward boundary reduction with transformations for requested representatives.
//!
//! The caller supplies face-before-coface columns. This production reducer is
//! independent of the test oracle and does not know simplices or diagrams.
mod boundary;
pub(crate) use boundary::{BoundaryReduction, reduce};

pub(crate) use boundary::reduce_pairs;
