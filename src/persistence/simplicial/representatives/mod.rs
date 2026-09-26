//! Requested persistent cycle bases and their scale-specific dual cocycles.
mod basis;
mod complex;
mod dual;
mod request;
pub(in crate::persistence) use basis::{compute, compute_explicit};
pub use request::{RepresentativeRequest, RepresentativeSelection};
