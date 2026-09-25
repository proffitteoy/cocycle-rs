//! Exact Rips threshold construction with original-input provenance.
pub(crate) mod approximation;
mod exact;
mod expansion;
pub(crate) use exact::cone_radius;
pub use exact::{
    RipsInputKind, ThresholdRips, threshold_rips_from_distances, threshold_rips_from_points,
    threshold_rips_with_distance,
};
pub use expansion::RipsExpansion;

pub(crate) mod builder;
pub use builder::{RipsBuilder, RipsCallbackBuilder};
