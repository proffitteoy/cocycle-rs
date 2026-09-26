//! Blocker-aware cofaces preserving original vertex labels and orientation.
use super::SparseRips;
use crate::Result;
use crate::complex::Simplex;
use crate::filtration::flag::CliqueAccess;
use crate::filtration::simplicial::ZeroBornSimplicialAccess;

pub(crate) struct SparseRipsAccess<'a> {
    pub(crate) input: &'a SparseRips,
    pub(crate) cutoff: f64,
}
impl ZeroBornSimplicialAccess for SparseRipsAccess<'_> {
    fn vertex_count(&self) -> usize {
        self.input.graph.vertex_count()
    }
    fn vertex_position(&self, original: usize) -> usize {
        self.input
            .metadata
            .retained_vertices
            .binary_search(&original)
            .unwrap()
    }
    fn vertices(&self) -> Result<Vec<Simplex>> {
        let mut result = Vec::new();
        result
            .try_reserve_exact(self.vertex_count())
            .map_err(|_| super::allocation())?;
        result.extend(
            self.input
                .metadata
                .retained_vertices
                .iter()
                .rev()
                .map(|&v| Simplex {
                    vertices: vec![v],
                    value: 0.0,
                }),
        );
        Ok(result)
    }
    fn visit_cofacets(
        &self,
        simplex: &Simplex,
        unique: bool,
        checkpoint: &mut impl FnMut() -> Result<()>,
        mut visitor: impl FnMut(Simplex) -> Result<()>,
    ) -> Result<()> {
        let mut local = simplex.clone();
        for v in &mut local.vertices {
            *v = self.vertex_position(*v);
        }
        CliqueAccess::Sparse(&self.input.graph, self.cutoff).visit_cofacets(
            &local,
            unique,
            checkpoint,
            |mut coface| {
                if super::blocker::allows(
                    coface.value,
                    self.input.factor,
                    coface.vertices.iter().map(|&v| self.input.radii[v]),
                ) {
                    for v in &mut coface.vertices {
                        *v = self.input.metadata.retained_vertices[*v];
                    }
                    visitor(coface)?;
                }
                Ok(())
            },
        )
    }
}
