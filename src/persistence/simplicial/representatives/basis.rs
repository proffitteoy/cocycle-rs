//! Persistent cycle selection, interval association and representative assembly.
use super::{RepresentativeRequest, RepresentativeSelection, complex, dual};
use crate::algebra::column::Column;
use crate::diagram::{
    Coverage, PersistenceDiagram, Representative, RepresentativeKind, RepresentativeTerm,
};
use crate::filtration::simplicial::ZeroBornSimplicialAccess;
use crate::persistence::{PersistenceOptions, assemble_diagram, execution::WorkBudget};
use crate::{Error, Result};

pub(in crate::persistence) fn compute(
    access: &impl ZeroBornSimplicialAccess,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    coverage: Coverage,
    budget: &mut WorkBudget<'_>,
) -> Result<(PersistenceDiagram, Vec<Representative>)> {
    validate(options, requests, coverage)?;
    let field = options.field();
    let (simplices, reduction) =
        complex::reduce(access, options.max_homology_dimension(), field, budget)?;
    finish(simplices, reduction, options, requests, coverage, budget)
}
pub(in crate::persistence) fn compute_explicit(
    source: &crate::complex::SimplicialComplex,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    coverage: Coverage,
    budget: &mut WorkBudget<'_>,
) -> Result<(PersistenceDiagram, Vec<Representative>)> {
    validate(options, requests, coverage)?;
    let input = super::super::input::read(source, options, budget)?;
    let mut simplices = Vec::new();
    simplices
        .try_reserve_exact(input.cells.len())
        .map_err(|_| allocation())?;
    for &id in &input.cells {
        budget.step()?;
        // IDs came from this same borrowed immutable source.
        simplices.push(source.simplex(id).unwrap().clone());
    }
    let reduction =
        crate::algebra::reduction::reduce(input.columns, options.field(), &mut || budget.step())?;
    finish(simplices, reduction, options, requests, coverage, budget)
}
fn validate(
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    coverage: Coverage,
) -> Result<()> {
    for (index, request) in requests.iter().enumerate() {
        if request.dimension > options.max_homology_dimension() {
            return Err(Error::DimensionNotComputed {
                requested: request.dimension,
                computed_max: options.max_homology_dimension(),
            });
        }
        if let Coverage::Through(through) = coverage
            && request.scale > through
        {
            return Err(Error::QueryOutsideCoverage {
                index,
                value: request.scale,
                through,
            });
        }
    }
    Ok(())
}
fn finish(
    simplices: Vec<crate::complex::Simplex>,
    reduction: crate::algebra::reduction::BoundaryReduction,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    coverage: Coverage,
    budget: &mut WorkBudget<'_>,
) -> Result<(PersistenceDiagram, Vec<Representative>)> {
    let field = options.field();
    let mut records = Vec::new();
    for (birth, simplex) in simplices.iter().enumerate() {
        budget.step()?;
        if simplex.dimension() > options.max_homology_dimension()
            || !reduction.reduced[birth].is_empty()
        {
            continue;
        }
        let death = reduction.deaths[birth];
        if death.is_some_and(|j| simplices[j].value == simplex.value) {
            continue;
        }
        // For finite intervals, R_death is born at this pivot and becomes a
        // boundary at death. V_birth alone need not die with its paired interval.
        let cycle = death.map_or(&reduction.transforms[birth], |j| &reduction.reduced[j]);
        records.try_reserve(1).map_err(|_| allocation())?;
        records.push((
            (
                simplex.dimension(),
                simplex.value,
                death.map(|j| simplices[j].value),
            ),
            cycle,
        ));
    }
    records.sort_by(|a, b| {
        a.0.0.cmp(&b.0.0).then(a.0.1.total_cmp(&b.0.1)).then(
            a.0.2
                .unwrap_or(f64::INFINITY)
                .total_cmp(&b.0.2.unwrap_or(f64::INFINITY)),
        )
    });
    let diagram = assemble_diagram(
        options.max_homology_dimension(),
        coverage,
        records.iter().map(|r| r.0),
    )?;
    let mut representatives = Vec::new();
    for (request_index, request) in requests.iter().enumerate() {
        budget.step()?;
        let active: Vec<_> = records
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.0.0 == request.dimension
                    && r.0.1 <= request.scale
                    && r.0.2.is_none_or(|death| request.scale < death)
            })
            .collect();
        if active.is_empty() {
            continue;
        }
        let cocycles = if request.selection != RepresentativeSelection::Cycles {
            let boundaries = reduction
                .reduced
                .iter()
                .enumerate()
                .filter(|(j, c)| {
                    !c.is_empty()
                        && simplices[*j].dimension() == request.dimension + 1
                        && simplices[*j].value <= request.scale
                })
                .map(|(_, c)| c);
            let cycles: Vec<_> = active.iter().map(|(_, r)| r.1).collect();
            dual::cocycles(boundaries, &cycles, field, budget)?
        } else {
            Vec::new()
        };
        for (position, (interval_index, record)) in active.iter().enumerate() {
            let selected = [
                (
                    RepresentativeKind::Cycle,
                    (request.selection != RepresentativeSelection::Cocycles).then_some(record.1),
                ),
                (RepresentativeKind::Cocycle, cocycles.get(position)),
            ];
            for (kind, column) in selected {
                if let Some(column) = column {
                    let terms = terms(column, &simplices, budget)?;
                    representatives.try_reserve(1).map_err(|_| allocation())?;
                    representatives.push(Representative {
                        request_index,
                        interval_index: *interval_index,
                        dimension: request.dimension,
                        characteristic: field.characteristic(),
                        scale: request.scale,
                        kind,
                        terms,
                    });
                }
            }
        }
    }
    budget.check()?;
    Ok((diagram, representatives))
}
fn terms(
    column: &Column<usize>,
    simplices: &[crate::complex::Simplex],
    budget: &mut WorkBudget<'_>,
) -> Result<Vec<RepresentativeTerm>> {
    let mut terms = Vec::new();
    for (&simplex, &coefficient) in column.entries() {
        budget.step()?;
        terms.try_reserve(1).map_err(|_| allocation())?;
        terms.push(RepresentativeTerm {
            vertices: simplices[simplex].vertices.clone(),
            coefficient,
        });
    }
    terms.sort_unstable_by(|a, b| a.vertices.cmp(&b.vertices));
    Ok(terms)
}
fn allocation() -> Error {
    Error::AllocationFailed {
        context: "owned representatives",
    }
}
