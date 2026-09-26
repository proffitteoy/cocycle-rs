//! Public filtered-cell adapters and non-flag simplicial regression cases.
use cocycle::{
    Error, Result,
    algebra::PrimeField,
    complex::{FilteredComplex, Simplex, SimplicialComplex},
    diagram::IntervalEnd,
    execution::Execution,
    filtration::{Coverage, FiltrationScale, FiltrationSource, RipsBuilder},
    geometry::PointCloudView,
    persistence::{
        PersistenceBuilder, PersistenceExt, RepresentativeRequest, RepresentativeSelection,
    },
};

fn simplex(vertices: &[usize], value: f64) -> Simplex {
    Simplex::new(vertices.to_vec(), value).unwrap()
}
fn alpha_triangle() -> SimplicialComplex {
    SimplicialComplex::new(vec![
        simplex(&[10, 30, 90], 1. / 3.),
        simplex(&[10, 30], 0.25),
        simplex(&[10, 90], 0.25),
        simplex(&[30, 90], 0.25),
        simplex(&[10], 0.),
        simplex(&[30], 0.),
        simplex(&[90], 0.),
    ])
    .unwrap()
}
#[test]
fn delayed_triangle_uses_all_simplex_values_and_preserves_certified_ranges() -> Result<()> {
    let complex = alpha_triangle();
    assert_eq!(complex.max_filtration_value(), Some(1. / 3.));
    let requests = [RepresentativeRequest::new(
        1,
        0.3,
        RepresentativeSelection::Both,
    )?];
    for prime in [2, 3, 251, 65537] {
        let field = PrimeField::new(prime)?;
        let result = complex.persistence().field(field).compute()?;
        let generic = PersistenceBuilder::from_complex(&complex)
            .field(field)
            .compute()?;
        let bases = complex
            .persistence()
            .field(field)
            .representatives(&requests)
            .compute()?;
        assert_eq!(result.diagram(), generic.diagram());
        assert_eq!(result.diagram(), bases.diagram());
        let interval = result.diagram().intervals_in_dimension(1)?.next().unwrap();
        assert_eq!(
            (interval.birth(), interval.end()),
            (0.25, IntervalEnd::Finite(1. / 3.))
        );
        assert_eq!(bases.representatives().unwrap().len(), 2);
        assert_eq!(
            result.context().filtration().scale(),
            FiltrationScale::Unspecified
        );
        assert!(matches!(
            result.context().filtration().source(),
            FiltrationSource::SuppliedSimplicial
        ));
    }
    let capped = complex.persistence().max_filtration_value(0.3).compute()?;
    assert_eq!(capped.diagram().coverage(), Coverage::Through(0.3));
    assert_eq!(
        capped
            .diagram()
            .intervals_in_dimension(1)?
            .next()
            .unwrap()
            .end(),
        IntervalEnd::RightCensored { through: 0.3 }
    );
    let early = complex.persistence().max_filtration_value(-1.).compute()?;
    assert!(early.diagram().intervals().is_empty());
    assert_eq!(early.diagram().coverage(), Coverage::Through(-1.));
    // A bare supplied skeleton is its own complete complex. A Rips certificate
    // must still reject the same insufficient skeleton relative to its source.
    let coordinates = [0., 1., 2.];
    let rips =
        RipsBuilder::from_points(PointCloudView::new(&coordinates, 3, 1)?).build_complex(1)?;
    assert!(matches!(
        rips.persistence().compute(),
        Err(Error::InsufficientSkeleton { .. })
    ));
    assert_eq!(
        rips.complex()
            .persistence()
            .compute()?
            .diagram()
            .intervals_in_dimension(1)?
            .next()
            .unwrap()
            .end(),
        IntervalEnd::Essential
    );
    Ok(())
}
#[test]
fn signed_vertex_births_and_interleaved_cells_agree_with_representatives() -> Result<()> {
    let complex = SimplicialComplex::new(vec![
        simplex(&[100], -4.),
        simplex(&[20], -3.),
        simplex(&[20, 100], -2.),
        simplex(&[999], 1.),
        simplex(&[100, 999], 2.),
    ])?;
    for cap in [None, Some(-5.), Some(-3.5), Some(-1.), Some(1.5), Some(2.)] {
        let mut query = complex.persistence().max_homology_dimension(0);
        if let Some(t) = cap {
            query = query.max_filtration_value(t);
        }
        let plain = query.compute()?;
        let requests = [RepresentativeRequest::new(
            0,
            cap.unwrap_or(-2.5),
            RepresentativeSelection::Both,
        )?];
        let mut query = complex
            .persistence()
            .max_homology_dimension(0)
            .representatives(&requests);
        if let Some(t) = cap {
            query = query.max_filtration_value(t);
        }
        let bases = query.compute()?;
        assert_eq!(plain.diagram(), bases.diagram());
        let mut generic = PersistenceBuilder::from_complex(&complex).max_homology_dimension(0);
        if let Some(t) = cap {
            generic = generic.max_filtration_value(t);
        }
        assert_eq!(plain.diagram(), generic.compute()?.diagram());
        for rep in bases.representatives().unwrap() {
            for term in rep.terms() {
                let id = complex.find(term.vertices()).unwrap();
                assert!(complex.simplex(id).unwrap().value() <= rep.scale());
            }
        }
    }
    let diagram = complex.persistence().max_homology_dimension(0).compute()?;
    assert_eq!(
        cocycle::descriptors::betti_curve(diagram.diagram(), 0, &[-5., -4., -3., -2., 0., 1., 2.])?,
        [0, 1, 2, 1, 1, 2, 1]
    );
    let intervals = diagram.diagram().intervals();
    assert_eq!(
        (intervals[0].birth(), intervals[0].end()),
        (-4., IntervalEnd::Essential)
    );
    assert_eq!(
        (intervals[1].birth(), intervals[1].end()),
        (-3., IntervalEnd::Finite(-2.))
    );
    assert_eq!(
        (intervals[2].birth(), intervals[2].end()),
        (1., IntervalEnd::Finite(2.))
    );
    Ok(())
}

#[test]
fn low_degree_selection_reuses_frozen_incidence_and_range() -> Result<()> {
    // Every nonempty face of a 9-simplex, with delayed positive-dimensional
    // simplices. The full range includes values beyond the H0 skeleton.
    let complex = SimplicialComplex::new(
        (1_u32..1 << 10)
            .map(|mask| {
                let labels = (0..10)
                    .filter(|v| mask & (1 << v) != 0)
                    .map(|v| v * 7 + 10)
                    .collect();
                Simplex::new(labels, f64::from(mask.count_ones()) - 2.).unwrap()
            })
            .collect(),
    )?;
    assert_eq!(complex.vertex_count(), 10);
    assert_eq!(complex.dimension(), Some(9));
    for prime in [2, 3, 65537] {
        let field = PrimeField::new(prime)?;
        for cutoff in [-2., -1., 0., 8.] {
            let requests = [RepresentativeRequest::new(
                0,
                cutoff,
                RepresentativeSelection::Both,
            )?];
            let direct = complex
                .persistence()
                .max_homology_dimension(0)
                .max_filtration_value(cutoff)
                .field(field)
                .compute()?;
            let bases = complex
                .persistence()
                .max_homology_dimension(0)
                .max_filtration_value(cutoff)
                .field(field)
                .representatives(&requests)
                .compute()?;
            let generic = PersistenceBuilder::from_complex(&complex)
                .max_homology_dimension(0)
                .max_filtration_value(cutoff)
                .field(field)
                .compute()?;
            assert_eq!(direct.diagram(), generic.diagram());
            assert_eq!(bases.diagram(), generic.diagram());
            assert_eq!(direct.context().vertex_count(), 10);
            assert_eq!(
                direct.diagram().coverage(),
                if cutoff < 8. {
                    Coverage::Through(cutoff)
                } else {
                    Coverage::Complete
                }
            );
        }
    }
    // A cutoff before the first vertex needs neither a full metadata traversal
    // nor boundary validation. Keep margin for reduction/control bookkeeping.
    let early = complex
        .persistence()
        .max_filtration_value(-2.)
        .compute_with(&Execution::default().max_work(16))?;
    assert!(early.diagram().intervals().is_empty());
    let empty = SimplicialComplex::new(vec![])?;
    assert_eq!(empty.vertex_count(), 0);
    assert_eq!(empty.dimension(), None);
    assert_eq!(
        empty.persistence().compute()?.diagram().coverage(),
        Coverage::Complete
    );
    Ok(())
}

#[test]
fn zero_born_non_flag_cofaces_agree_with_boundary_reduction() -> Result<()> {
    // A tetrahedron boundary, with edges at 1, three faces at 2 and the last
    // face at 3. H1 has three [1, 2) bars; H2 has one [3, infinity) bar.
    // No tetrahedron exists, despite its complete graph. Labels are not indices.
    let labels = [10, 30, 90, 200];
    let simplices = (1_u32..15)
        .map(|mask| {
            let vertices = labels
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| (mask & (1 << i) != 0).then_some(v))
                .collect();
            let value = match mask.count_ones() {
                1 => 0.,
                2 => 1.,
                _ if mask == 14 => 3.,
                _ => 2.,
            };
            Simplex::new(vertices, value).unwrap()
        })
        .collect();
    let complex = SimplicialComplex::new(simplices)?;
    for prime in [2, 3, 65537] {
        for cutoff in [-1., 0., 1., 2., 3.] {
            let field = PrimeField::new(prime)?;
            let actual = complex
                .persistence()
                .max_homology_dimension(2)
                .max_filtration_value(cutoff)
                .field(field)
                .compute()?;
            let boundary = PersistenceBuilder::from_complex(&complex)
                .max_homology_dimension(2)
                .max_filtration_value(cutoff)
                .field(field)
                .compute()?;
            assert_eq!(actual.diagram(), boundary.diagram());
            if cutoff == 3. {
                let h1: Vec<_> = actual.diagram().intervals_in_dimension(1)?.collect();
                assert_eq!(h1.len(), 3);
                assert!(
                    h1.iter()
                        .all(|i| i.birth() == 1. && i.end() == IntervalEnd::Finite(2.))
                );
                let h2: Vec<_> = actual.diagram().intervals_in_dimension(2)?.collect();
                assert_eq!(h2.len(), 1);
                assert_eq!((h2[0].birth(), h2[0].end()), (3., IntervalEnd::Essential));
            }
        }
    }
    Ok(())
}

// A minimal cell adapter: the 2-cell attaches twice around a loop. This is
// not a simplicial container, and detects lost integer incidence coefficients.
struct AttachedDisk;
impl FilteredComplex for AttachedDisk {
    type CellId = &'static str;
    fn cells(&self) -> impl Iterator<Item = Self::CellId> + '_ {
        ["point", "loop", "disk"].into_iter()
    }
    fn dimension(&self, cell: Self::CellId) -> usize {
        match cell {
            "point" => 0,
            "loop" => 1,
            _ => 2,
        }
    }
    fn filtration_value(&self, cell: Self::CellId) -> f64 {
        match cell {
            "point" => -4.,
            "loop" => -2.,
            _ => 1.,
        }
    }
    fn boundary(&self, cell: Self::CellId) -> impl Iterator<Item = (Self::CellId, i32)> + '_ {
        (cell == "disk").then_some(("loop", -2)).into_iter()
    }
}
#[test]
fn external_cell_contract_handles_field_dependent_homology() -> Result<()> {
    let source = AttachedDisk;
    let mod2 = PersistenceBuilder::from_complex(&source)
        .max_homology_dimension(2)
        .compute()?;
    assert_eq!(
        mod2.diagram()
            .intervals_in_dimension(1)?
            .next()
            .unwrap()
            .end(),
        IntervalEnd::Essential
    );
    assert_eq!(
        mod2.diagram()
            .intervals_in_dimension(2)?
            .next()
            .unwrap()
            .end(),
        IntervalEnd::Essential
    );
    let mod3 = PersistenceBuilder::from_complex(&source)
        .max_homology_dimension(2)
        .field(PrimeField::new(3)?)
        .compute()?;
    let interval = mod3.diagram().intervals_in_dimension(1)?.next().unwrap();
    assert_eq!(
        (interval.birth(), interval.end()),
        (-2., IntervalEnd::Finite(1.))
    );
    assert_eq!(mod3.diagram().intervals_in_dimension(2)?.count(), 0);
    let requests = [RepresentativeRequest::new(
        1,
        0.,
        RepresentativeSelection::Both,
    )?];
    assert!(matches!(
        PersistenceBuilder::from_complex(&source)
            .representatives(&requests)
            .compute(),
        Err(Error::InvalidParameter {
            parameter: "representatives",
            ..
        })
    ));
    Ok(())
}
#[test]
fn construction_rejects_invalid_topology_and_filtration() {
    for vertices in [vec![], vec![0, 0], vec![2, 1]] {
        assert!(Simplex::new(vertices, 0.).is_err());
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(Simplex::new(vec![0], value).is_err());
    }
    assert!(matches!(
        SimplicialComplex::new(vec![simplex(&[0], 0.), simplex(&[0], 1.)]),
        Err(Error::InvalidComplex { .. })
    ));
    assert!(matches!(
        SimplicialComplex::new(vec![simplex(&[0], 0.), simplex(&[0, 1], 1.)]),
        Err(Error::InvalidComplex { .. })
    ));
    assert!(matches!(
        SimplicialComplex::new(vec![
            simplex(&[0], 2.),
            simplex(&[1], 0.),
            simplex(&[0, 1], 1.)
        ]),
        Err(Error::InvalidComplex { .. })
    ));
    let empty = SimplicialComplex::new(vec![]).unwrap();
    assert_eq!(empty.max_filtration_value(), None);
    assert!(
        empty
            .persistence()
            .max_filtration_value(-3.)
            .compute()
            .unwrap()
            .diagram()
            .intervals()
            .is_empty()
    );
    let legacy: cocycle::complex::FilteredSimplicialComplex = alpha_triangle();
    assert_eq!(legacy.len(), 7);
}

struct Broken {
    order: Vec<usize>,
    dimensions: Vec<usize>,
    values: Vec<f64>,
    boundaries: Vec<Vec<(usize, i32)>>,
}
impl FilteredComplex for Broken {
    type CellId = usize;
    fn cells(&self) -> impl Iterator<Item = usize> + '_ {
        self.order.iter().copied()
    }
    fn dimension(&self, c: usize) -> usize {
        self.dimensions[c]
    }
    fn filtration_value(&self, c: usize) -> f64 {
        self.values[c]
    }
    fn boundary(&self, c: usize) -> impl Iterator<Item = (usize, i32)> + '_ {
        self.boundaries[c].iter().copied()
    }
}
#[test]
fn generic_path_checks_order_incidence_and_boundary_square() {
    let cases = [
        Broken {
            order: vec![0, 0],
            dimensions: vec![0],
            values: vec![0.],
            boundaries: vec![vec![]],
        },
        Broken {
            order: vec![0, 1],
            dimensions: vec![0, 0],
            values: vec![1., 0.],
            boundaries: vec![vec![], vec![]],
        },
        Broken {
            order: vec![0],
            dimensions: vec![0],
            values: vec![f64::NAN],
            boundaries: vec![vec![]],
        },
        Broken {
            order: vec![0, 1],
            dimensions: vec![0, 1],
            values: vec![0., 1.],
            boundaries: vec![vec![], vec![(5, 1)]],
        },
        Broken {
            order: vec![0, 1],
            dimensions: vec![0, 2],
            values: vec![0., 1.],
            boundaries: vec![vec![], vec![(0, 1)]],
        },
        Broken {
            order: vec![0, 1, 2],
            dimensions: vec![0, 1, 2],
            values: vec![0., 1., 2.],
            boundaries: vec![vec![], vec![(0, 1)], vec![(1, 1)]],
        },
    ];
    for source in cases {
        assert!(matches!(
            PersistenceBuilder::from_complex(&source)
                .max_homology_dimension(2)
                .compute(),
            Err(Error::InvalidComplex { .. })
        ));
    }
}
#[test]
fn generic_work_budget_cancellation_and_recovery() -> Result<()> {
    use std::sync::atomic::AtomicBool;
    let complex = alpha_triangle();
    assert!(matches!(
        complex
            .persistence()
            .compute_with(&Execution::default().max_work(0)),
        Err(Error::WorkLimitExceeded { .. })
    ));
    let flag = AtomicBool::new(true);
    assert!(matches!(
        PersistenceBuilder::from_complex(&complex)
            .compute_with(&Execution::default().cancellation(&flag)),
        Err(Error::Cancelled)
    ));
    let mut required = 0;
    for n in 1..5000 {
        match complex
            .persistence()
            .compute_with(&Execution::default().max_work(n))
        {
            Ok(_) => {
                required = n;
                break;
            }
            Err(Error::WorkLimitExceeded { .. }) => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(required > 1);
    assert!(
        complex
            .persistence()
            .compute_with(&Execution::default().max_work(required - 1))
            .is_err()
    );
    let baseline = complex.persistence().compute()?;
    assert_eq!(
        baseline,
        complex
            .persistence()
            .compute_with(&Execution::default().max_work(required))?
    );
    Ok(())
}
