//! Clique enumeration from distances or a supplied graph.
use crate::complex::{Simplex, WeightedGraph};
use crate::filtration::simplicial::ZeroBornSimplicialAccess;
use crate::geometry::DissimilarityMatrixView;
use crate::{Error, Result};
pub(crate) enum CliqueAccess<'a> {
    Dense(DissimilarityMatrixView<'a>, f64),
    Sparse(&'a WeightedGraph, f64),
}
impl ZeroBornSimplicialAccess for CliqueAccess<'_> {
    fn vertex_count(&self) -> usize {
        match self {
            Self::Dense(matrix, _) => matrix.len(),
            Self::Sparse(graph, _) => graph.vertex_count(),
        }
    }
    fn visit_cofacets(
        &self,
        simplex: &Simplex,
        unique: bool,
        checkpoint: &mut impl FnMut() -> Result<()>,
        mut visitor: impl FnMut(Simplex) -> Result<()>,
    ) -> Result<()> {
        let mut candidate = |vertex: usize| -> Result<()> {
            checkpoint()?;
            if (unique && vertex <= *simplex.vertices.last().unwrap())
                || simplex.vertices.binary_search(&vertex).is_ok()
            {
                return Ok(());
            }
            let mut value = simplex.value;
            for &v in &simplex.vertices {
                checkpoint()?;
                let (edge, cutoff) = match self {
                    Self::Dense(matrix, cutoff) => (matrix.get(v, vertex), cutoff),
                    Self::Sparse(graph, cutoff) => (graph.edge_value(v, vertex), cutoff),
                };
                let Some(edge) = edge.filter(|w| w <= cutoff) else {
                    return Ok(());
                };
                value = value.max(edge);
            }
            let mut vertices = simplex.vertices.clone();
            vertices.try_reserve(1).map_err(|_| allocation())?;
            let position = vertices.partition_point(|&v| v < vertex);
            vertices.insert(position, vertex);
            visitor(Simplex { vertices, value })
        };
        match self {
            Self::Dense(matrix, _) => {
                for vertex in (0..matrix.len()).rev() {
                    candidate(vertex)?;
                }
            }
            Self::Sparse(graph, _) => {
                // Every common neighbor must occur in the shortest adjacency list.
                let neighbors = simplex
                    .vertices
                    .iter()
                    .map(|&v| graph.neighbors(v).unwrap())
                    .min_by_key(|neighbors| neighbors.len())
                    .unwrap();
                for neighbor in neighbors.iter().rev() {
                    candidate(neighbor.vertex)?;
                }
            }
        }
        Ok(())
    }
}

fn allocation() -> Error {
    Error::AllocationFailed {
        context: "clique enumeration",
    }
}
