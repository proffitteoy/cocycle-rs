//! Persistence and representatives of explicit simplicial topology.
mod input;
pub(super) mod representatives;

use super::{PersistenceOptions, RepresentativeRequest};
use crate::complex::SimplicialComplex;
use crate::diagram::{PersistenceDiagram, Representative};
use crate::execution::WorkBudget;
use crate::filtration::Coverage;
use crate::filtration::simplicial::ZeroBornExplicitAccess;
use crate::{Error, Result};
pub(super) fn source_range(
    coverage: Coverage,
    maximum: Option<f64>,
    requested: Option<f64>,
) -> Result<(Option<f64>, Coverage)> {
    match coverage {
        Coverage::Through(through) => {
            let value = requested.unwrap_or(through);
            if value > through {
                return Err(Error::IncompleteFiltration {
                    requested: value,
                    through,
                });
            }
            Ok((Some(value), Coverage::Through(value)))
        }
        Coverage::Complete => Ok((
            requested,
            match (requested, maximum) {
                (Some(t), Some(m)) if t < m => Coverage::Through(t),
                _ => Coverage::Complete,
            },
        )),
    }
}
pub(super) fn compute(
    source: &SimplicialComplex,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    coverage: Coverage,
    budget: &mut WorkBudget<'_>,
) -> Result<(PersistenceDiagram, Option<Vec<Representative>>)> {
    let result = if !requests.is_empty() {
        let (diagram, representatives) =
            representatives::compute_explicit(source, options, requests, coverage, budget)?;
        (diagram, Some(representatives))
    } else if options.max_edge().is_none_or(|t| t >= 0.) && source.has_zero_born_vertices() {
        // Select by the actual simplex invariant, not source/scale metadata.
        // Stored cofaces retain non-flag topology and arbitrary simplex values.
        let access = ZeroBornExplicitAccess {
            complex: source,
            vertex_count: source.vertex_count(),
            cutoff: options
                .max_edge()
                .unwrap_or_else(|| source.max_filtration_value().unwrap_or(0.)),
        };
        (
            super::assemble_diagram(
                options.max_homology_dimension(),
                coverage,
                cohomology::compute(
                    &access,
                    options.max_homology_dimension(),
                    options.field(),
                    budget,
                )?,
            )?,
            None,
        )
    } else {
        (
            super::boundary::diagram(
                input::read(source, options, budget)?,
                options.max_homology_dimension(),
                options.field(),
                coverage,
                budget,
            )?,
            None,
        )
    };
    budget.check()?;
    Ok(result)
}

pub(super) mod cohomology;

/// Shared dispatch for optional representatives; ordinary calls keep their implicit engine.
pub(super) fn finish_zero_born(
    access: &impl crate::filtration::simplicial::ZeroBornSimplicialAccess,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    coverage: Coverage,
    budget: &mut WorkBudget<'_>,
    implicit: impl FnOnce(&mut WorkBudget<'_>) -> Result<super::RawIntervals>,
) -> Result<(
    crate::diagram::PersistenceDiagram,
    Option<Vec<crate::diagram::Representative>>,
)> {
    let result = if requests.is_empty() {
        (
            super::assemble_diagram(
                options.max_homology_dimension(),
                coverage,
                implicit(budget)?,
            )?,
            None,
        )
    } else {
        let (diagram, representatives) =
            representatives::compute(access, options, requests, coverage, budget)?;
        (diagram, Some(representatives))
    };
    budget.check()?;
    Ok(result)
}
