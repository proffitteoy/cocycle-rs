//! Filtration construction, provenance and shared implicit access.
//!
//! Exact threshold Rips and supplied flag filtrations differ in what missing
//! edges mean. Construction never performs persistence reduction.

pub(crate) mod flag;
pub(crate) mod rips;
pub use flag::FlagFiltration;
pub use rips::{
    RipsExpansion, RipsInputKind, ThresholdRips, threshold_rips_from_distances,
    threshold_rips_from_points, threshold_rips_with_distance,
};

pub use rips::approximation::{
    SparseRips, SparseRipsExpansion, SparseRipsOptions, sparse_rips_from_distances,
    sparse_rips_from_points, sparse_rips_with_distance,
};

mod provenance;
pub use provenance::{Coverage, FiltrationKind};
pub use rips::approximation::{ApproximationTarget, RipsApproximation, RipsApproximationBound};

pub(crate) mod expansion;
pub(crate) mod simplicial;
pub use rips::{RipsBuilder, RipsCallbackBuilder};
pub use simplicial::SimplicialFiltration;
mod context;
pub use context::{FiltrationContext, FiltrationScale, FiltrationSource};
/// Owned exact preparation; compatibility name is [`ThresholdRips`].
pub type RipsFiltration = ThresholdRips;
/// Owned blocker-aware preparation; compatibility name is [`SparseRips`].
pub type ApproximateRipsFiltration = SparseRips;

pub use rips::approximation::{ApproximateRipsBuilder, ApproximateRipsCallbackBuilder};
