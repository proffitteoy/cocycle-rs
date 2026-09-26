//! Explicit clique expansion shared by supplied flags and exact Rips.
use super::cliques::CliqueAccess;
use crate::complex::{SimplicialComplex, WeightedGraph};
use crate::filtration::simplicial::{ZeroBornSimplicialAccess, next_dimension};
use crate::{Error, Result};

pub(crate) fn expand(
    graph: &WeightedGraph,
    max_dimension: usize,
) -> Result<(SimplicialComplex, bool)> {
    let access = CliqueAccess::Sparse(graph, graph.max_edge());
    expand_access(&access, max_dimension)
}

pub(crate) fn expand_access(
    access: &impl ZeroBornSimplicialAccess,
    max_dimension: usize,
) -> Result<(SimplicialComplex, bool)> {
    expand_access_with(access, max_dimension, &mut || Ok(()))
}

pub(crate) fn expand_access_with(
    access: &impl ZeroBornSimplicialAccess,
    max_dimension: usize,
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<(SimplicialComplex, bool)> {
    checkpoint()?;
    let mut level = access.vertices()?;
    let mut simplices = Vec::new();
    let mut complete = level.is_empty();
    for dimension in 0..=max_dimension.min(access.vertex_count().saturating_sub(1)) {
        checkpoint()?;
        simplices
            .try_reserve(level.len())
            .map_err(|_| Error::AllocationFailed {
                context: "expanded simplices",
            })?;
        simplices.extend(level.iter().cloned());
        // Probe the next dimension without materializing it at the requested
        // boundary. This certifies exhaustion even for triangle-free graphs.
        if dimension == max_dimension || dimension + 1 == access.vertex_count() {
            complete = true;
            for simplex in &level {
                access.visit_cofacets(simplex, true, checkpoint, |_| {
                    complete = false;
                    Ok(())
                })?;
                if !complete {
                    break;
                }
            }
            break;
        }
        level = next_dimension(access, &level, checkpoint)?;
        if level.is_empty() {
            complete = true;
            break;
        }
    }
    Ok((
        SimplicialComplex::from_simplices(simplices, checkpoint)?,
        complete,
    ))
}
