//! Private coface enumeration used by zero-born Rips/flag engines.
use crate::complex::{Simplex, SimplicialComplex};
use crate::{Error, Result};
/// Implementations supply zero-valued vertices and all codimension-one cofaces
/// within their cutoff. Unique extension adds only vertices greater than the
/// last vertex, so dimension traversal emits every clique exactly once.
pub(crate) trait ZeroBornSimplicialAccess {
    fn vertex_count(&self) -> usize;
    fn vertices(&self) -> Result<Vec<Simplex>> {
        vertices(self.vertex_count())
    }
    fn vertex_position(&self, original: usize) -> usize {
        original
    }
    fn visit_cofacets(
        &self,
        simplex: &Simplex,
        unique: bool,
        checkpoint: &mut impl FnMut() -> Result<()>,
        visitor: impl FnMut(Simplex) -> Result<()>,
    ) -> Result<()>;
}

pub(crate) struct ZeroBornExplicitAccess<'a> {
    pub(crate) complex: &'a SimplicialComplex,
    pub(crate) vertex_count: usize,
    pub(crate) cutoff: f64,
}
impl ZeroBornSimplicialAccess for ZeroBornExplicitAccess<'_> {
    fn vertices(&self) -> Result<Vec<Simplex>> {
        let mut result = Vec::new();
        result
            .try_reserve_exact(self.vertex_count)
            .map_err(|_| allocation())?;
        result.extend(
            self.complex
                .simplices()
                .iter()
                // Face monotonicity and value/dimension order put all zero-born
                // vertices first. This access is only used under that invariant.
                .take(self.vertex_count)
                .cloned(),
        );
        Ok(result)
    }
    fn vertex_position(&self, original: usize) -> usize {
        // Zero-born vertices precede every positive-dimensional simplex.
        self.complex.find(&[original]).unwrap().index()
    }
    fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    fn visit_cofacets(
        &self,
        simplex: &Simplex,
        unique: bool,
        checkpoint: &mut impl FnMut() -> Result<()>,
        mut visitor: impl FnMut(Simplex) -> Result<()>,
    ) -> Result<()> {
        let id = self
            .complex
            .find(&simplex.vertices)
            .ok_or(Error::InternalInvariant {
                reason: "explicit column missing",
            })?;
        for &cofacet in self.complex.cofacets(id).unwrap() {
            checkpoint()?;
            let cofacet = self.complex.simplex(cofacet).unwrap();
            if cofacet.value <= self.cutoff
                && (!unique || cofacet.vertices[..simplex.vertices.len()] == simplex.vertices)
            {
                visitor(cofacet.clone())?;
            }
        }
        Ok(())
    }
}

pub(crate) fn vertices(n: usize) -> Result<Vec<Simplex>> {
    let mut result = Vec::new();
    result.try_reserve_exact(n).map_err(|_| allocation())?;
    result.extend((0..n).map(|v| Simplex {
        vertices: vec![v],
        value: 0.0,
    }));
    result.sort_unstable();
    Ok(result)
}
pub(crate) fn next_dimension(
    access: &impl ZeroBornSimplicialAccess,
    level: &[Simplex],
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<Vec<Simplex>> {
    let mut result = Vec::new();
    for simplex in level {
        checkpoint()?;
        access.visit_cofacets(simplex, true, checkpoint, |cofacet| {
            result.try_reserve(1).map_err(|_| allocation())?;
            result.push(cofacet);
            Ok(())
        })?;
    }
    result.sort_unstable();
    checkpoint()?;
    Ok(result)
}
fn allocation() -> Error {
    Error::AllocationFailed {
        context: "clique enumeration",
    }
}
