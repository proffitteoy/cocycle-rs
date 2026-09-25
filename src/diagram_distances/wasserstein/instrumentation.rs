//! Private controls and counters, compiled only for tests and the native worker.

use super::SparseLayout;

/// These switches are consumed by the standalone experimental worker, not the
/// public library API. Forced sparse mode isolates the residual-network change.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Options {
    pub(crate) sparse: SparseLayout,
    pub(crate) force_sparse: bool,
}

#[derive(Debug, Default)]
pub(crate) struct Stats {
    pub(crate) candidate_pairs: usize,
    pub(crate) positive_edges: usize,
    pub(crate) dense_solves: usize,
    pub(crate) sparse_solves: usize,
    pub(crate) augmentations: usize,
    pub(crate) components: usize,
    pub(crate) tiny_components: usize,
    pub(crate) duplicate_groups: usize,
    pub(crate) greedy_certificates: usize,
    pub(crate) scratch_reuses: usize,
    pub(crate) direct_cost_fallbacks: usize,
    /// Maximum capacities of individual retained graphs, excluding temporaries.
    pub(crate) peak_graph_storage_bytes: usize,
    pub(crate) peak_residual_storage_bytes: usize,
    /// Search vectors and heap only; not simultaneous total storage or RSS.
    pub(crate) peak_sparse_scratch_bytes: usize,
}
