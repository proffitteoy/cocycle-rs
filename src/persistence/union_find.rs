//! Shared component connectivity for the Rips H0 and H1 computations.
//!
//! Callers decide which edges to scan and how merges create persistence pairs.
//! A component root is a union-by-size choice, not a persistence representative.

use crate::{Error, Result};

/// Component partition with path halving and union by size.
pub(in crate::persistence) struct UnionFind {
    parents: Vec<usize>,
    sizes: Vec<usize>,
    components: usize,
}

impl UnionFind {
    pub(in crate::persistence) fn new(n: usize) -> Result<Self> {
        let mut parents = Vec::new();
        let mut sizes = Vec::new();
        parents
            .try_reserve_exact(n)
            .map_err(|_| Error::AllocationFailed {
                context: "H0 parents",
            })?;
        sizes
            .try_reserve_exact(n)
            .map_err(|_| Error::AllocationFailed {
                context: "H0 sizes",
            })?;
        parents.extend(0..n);
        sizes.resize(n, 1);
        Ok(Self {
            parents,
            sizes,
            components: n,
        })
    }

    /// Number of components after all merges performed so far.
    pub(in crate::persistence) fn components(&self) -> usize {
        self.components
    }

    pub(in crate::persistence) fn merge(&mut self, a: usize, b: usize) -> bool {
        let (mut a, mut b) = (root(&mut self.parents, a), root(&mut self.parents, b));
        if a == b {
            return false;
        }
        if self.sizes[a] < self.sizes[b] {
            std::mem::swap(&mut a, &mut b);
        }
        self.parents[b] = a;
        self.sizes[a] += self.sizes[b];
        self.components -= 1;
        true
    }
}

fn root(parents: &mut [usize], mut vertex: usize) -> usize {
    while parents[vertex] != vertex {
        parents[vertex] = parents[parents[vertex]];
        vertex = parents[vertex];
    }
    vertex
}
