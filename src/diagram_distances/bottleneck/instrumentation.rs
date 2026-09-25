//! Private controls and counters, compiled only for tests and the native worker.

use super::Route;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Search {
    #[default]
    Adaptive,
    Quickselect,
    Binary,
    Refinement,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Options {
    pub(crate) search: Search,
    pub(crate) clip_candidates: bool,
    pub(crate) reuse_matching: bool,
    pub(crate) reuse_scratch: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            search: Search::Adaptive,
            clip_candidates: true,
            reuse_matching: true,
            reuse_scratch: true,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Diagnostics {
    pub(crate) route: Route,
    pub(crate) threshold_decisions: usize,
    pub(crate) candidate_count: usize,
    pub(crate) adjacency_checks: usize,
    pub(crate) augment_searches: usize,
    pub(crate) kd_nodes_visited: usize,
    pub(crate) matching_reuses: usize,
    pub(crate) scratch_reuses: usize,
    pub(crate) capacity_edges: usize,
    /// Largest explicitly accounted kernel buffer footprint, not process RSS.
    pub(crate) peak_workspace_bytes: usize,
}

impl Diagnostics {
    pub(super) fn workspace(&mut self, bytes: usize) {
        self.peak_workspace_bytes = self.peak_workspace_bytes.max(bytes);
    }
}
