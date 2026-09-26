//! End-to-end public workflows, source contracts and owned outputs.
use cocycle::{
    Error, Result,
    algebra::PrimeField,
    complex::{WeightedEdge, WeightedGraph},
    diagram::IntervalEnd,
    filtration::{ApproximateRipsBuilder, Coverage, FiltrationKind, FlagFiltration, RipsBuilder},
    geometry::{DissimilarityMatrixView, MatrixLayout, MetricPolicy, PointCloudView},
    persistence::{PersistenceExt, RepresentativeRequest, RepresentativeSelection},
};
use std::cell::Cell;

fn matrix(values: &[f64], n: usize) -> DissimilarityMatrixView<'_> {
    DissimilarityMatrixView::new(values, n, MatrixLayout::LowerTriangle).unwrap()
}
#[test]
fn square_workflows_preserve_fields_bases_context_and_source_ownership() -> Result<()> {
    let result = {
        let values = [1., 2., 1., 1., 2., 1.];
        let rips = RipsBuilder::from_distance_matrix(matrix(&values, 4));
        let filtration = rips.build_complex(2)?;
        assert_eq!(filtration.complex().len(), 14);
        let triangle = filtration.complex().find(&[0, 1, 2]).unwrap();
        assert_eq!(filtration.complex().boundary(triangle).unwrap().len(), 3);
        for characteristic in [2, 3, 251, 65537] {
            let field = PrimeField::new(characteristic)?;
            let requests = [RepresentativeRequest::new(
                1,
                1.,
                RepresentativeSelection::Both,
            )?];
            let direct = rips
                .persistence()
                .field(field)
                .representatives(&requests)
                .compute()?;
            let explicit = filtration
                .persistence()
                .field(field)
                .representatives(&requests)
                .compute()?;
            assert_eq!(direct.diagram(), explicit.diagram());
            assert_eq!(direct.context(), explicit.context());
            assert_eq!(direct.representatives().unwrap().len(), 2);
            assert_eq!(explicit.representatives().unwrap().len(), 2);
            let interval = direct.diagram().intervals_in_dimension(1)?.next().unwrap();
            assert_eq!(
                (interval.birth(), interval.end()),
                (1., IntervalEnd::Finite(2.))
            );
        }
        rips.persistence().compute()?
    };
    assert_eq!(result.diagram().coverage(), Coverage::Complete);
    assert_eq!(result.diagram().intervals_in_dimension(1)?.count(), 1);
    Ok(())
}
#[test]
fn matrix_layouts_and_points_agree_without_losing_construction_caps() -> Result<()> {
    let coordinates = [0., 0., 1., 0., 1., 1., 0., 1.];
    let points = PointCloudView::new(&coordinates, 4, 2)?;
    let diagonal = 2.0_f64.sqrt();
    let lower = [1., diagonal, 1., 1., diagonal, 1.];
    let upper = [1., diagonal, 1., 1., diagonal, 1.];
    let square = [
        0., 1., diagonal, 1., 1., 0., 1., diagonal, diagonal, 1., 0., 1., 1., diagonal, 1., 0.,
    ];
    for cap in [None, Some(1.), Some(2.)] {
        let mut points_builder = RipsBuilder::from_points(points);
        if let Some(t) = cap {
            points_builder = points_builder.max_edge_length(t);
        }
        let expected = points_builder.persistence().compute()?;
        assert_eq!(expected.context().construction_cutoff(), cap);
        for (values, layout) in [
            (&lower[..], MatrixLayout::LowerTriangle),
            (&upper[..], MatrixLayout::UpperTriangle),
            (&square[..], MatrixLayout::Square),
        ] {
            let mut rips =
                RipsBuilder::from_distance_matrix(DissimilarityMatrixView::new(values, 4, layout)?);
            if let Some(t) = cap {
                rips = rips.max_edge_length(t);
            }
            let direct = rips.persistence().compute()?;
            let prepared = rips.prepare()?.persistence().compute()?;
            let explicit = rips.build_complex(2)?.persistence().compute()?;
            assert_eq!(direct.diagram(), expected.diagram());
            assert_eq!(direct.diagram(), prepared.diagram());
            assert_eq!(direct.diagram(), explicit.diagram());
            assert_eq!(direct.context(), explicit.context());
        }
    }
    let rips = RipsBuilder::from_points(points);
    let analysis_only = rips.persistence().max_filtration_value(1.).compute()?;
    assert_eq!(analysis_only.context().construction_cutoff(), None);
    assert_eq!(analysis_only.context().requested_cutoff(), Some(1.));
    Ok(())
}

#[test]
fn independent_point_caps_preserve_source_coverage_and_representatives() -> Result<()> {
    let coordinates = [0., 1., 2., 3.];
    let points = PointCloudView::new(&coordinates, 4, 1)?;
    let distances = matrix(&[1., 2., 1., 3., 2., 1.], 4);
    for construction in [None, Some(0.5), Some(1.), Some(3.), Some(4.)] {
        let mut rips = RipsBuilder::from_points(points);
        let mut dense = RipsBuilder::from_distance_matrix(distances);
        if let Some(cap) = construction {
            rips = rips.max_edge_length(cap);
            dense = dense.max_edge_length(cap);
        }
        let prepared = rips.prepare()?;
        let expanded = rips.build_complex(2)?;
        for analysis in [None, Some(0.), Some(0.5), Some(1.5), Some(3.), Some(5.)] {
            for prime in [2, 3, 65537] {
                let field = PrimeField::new(prime)?;
                let scale = analysis.or(construction).unwrap_or(5.);
                let requests = [RepresentativeRequest::new(
                    0,
                    scale,
                    RepresentativeSelection::Both,
                )?];
                for representatives in [&[][..], &requests[..]] {
                    let mut direct = rips
                        .persistence()
                        .field(field)
                        .representatives(representatives);
                    let mut stored = prepared
                        .persistence()
                        .field(field)
                        .representatives(representatives);
                    let mut explicit = expanded
                        .persistence()
                        .field(field)
                        .representatives(representatives);
                    let mut matrix = dense
                        .persistence()
                        .field(field)
                        .representatives(representatives);
                    if let Some(cap) = analysis {
                        direct = direct.max_filtration_value(cap);
                        stored = stored.max_filtration_value(cap);
                        explicit = explicit.max_filtration_value(cap);
                        matrix = matrix.max_filtration_value(cap);
                    }
                    let results = [
                        direct.compute(),
                        stored.compute(),
                        explicit.compute(),
                        matrix.compute(),
                    ];
                    if construction.is_some_and(|cap| cap < 3. && analysis.is_some_and(|t| t > cap))
                    {
                        for result in results {
                            assert!(matches!(result, Err(Error::IncompleteFiltration { .. })));
                        }
                        continue;
                    }
                    let [direct, stored, explicit, matrix] = results;
                    let direct = direct?;
                    for other in [stored?, explicit?, matrix?] {
                        assert_eq!(direct.diagram(), other.diagram());
                        assert_eq!(
                            direct.context().construction_cutoff(),
                            other.context().construction_cutoff()
                        );
                        assert_eq!(
                            direct.context().requested_cutoff(),
                            other.context().requested_cutoff()
                        );
                        assert_eq!(
                            direct.representatives().map(<[_]>::len),
                            other.representatives().map(<[_]>::len)
                        );
                    }
                    assert_eq!(direct.context().construction_cutoff(), construction);
                    assert_eq!(direct.context().requested_cutoff(), analysis);
                    let effective = match (construction, analysis) {
                        (Some(a), Some(b)) => Some(a.min(b)),
                        (a, b) => a.or(b),
                    };
                    assert_eq!(
                        direct.diagram().coverage(),
                        match effective {
                            Some(t) if t < 3. => Coverage::Through(t),
                            _ => Coverage::Complete,
                        }
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn direct_point_analysis_does_not_process_edges_above_its_cutoff() -> Result<()> {
    let coordinates: Vec<_> = (0..32).map(f64::from).collect();
    let rips =
        RipsBuilder::from_points(PointCloudView::new(&coordinates, 32, 1)?).max_edge_length(100.);
    // Enough for every pair's distance checks and an edgeless H0 computation,
    // but not another pass over all 496 edges allowed by the construction cap.
    let execution = cocycle::execution::Execution::default().max_work(1200);
    for prime in [2, 3, 65537] {
        let result = rips
            .persistence()
            .max_homology_dimension(0)
            .max_filtration_value(0.5)
            .field(PrimeField::new(prime)?)
            .compute_with(&execution)?;
        assert_eq!(result.diagram().intervals().len(), 32);
        assert_eq!(result.diagram().coverage(), Coverage::Through(0.5));
        assert_eq!(result.context().construction_cutoff(), Some(100.));
    }
    Ok(())
}
#[test]
fn scale_and_dimension_truncation_are_independent() -> Result<()> {
    let values: Vec<_> = (0..6)
        .flat_map(|b| (0..b).map(move |a| if a / 2 == b / 2 { 2. } else { 1. }))
        .collect();
    let rips = RipsBuilder::from_distance_matrix(matrix(&values, 6));
    let skeleton = rips.build_complex(2)?;
    assert!(matches!(
        skeleton.persistence().max_homology_dimension(2).compute(),
        Err(Error::InsufficientSkeleton { .. })
    ));
    let explicit = rips
        .build_complex(3)?
        .persistence()
        .max_homology_dimension(2)
        .compute()?;
    let direct = rips.persistence().max_homology_dimension(2).compute()?;
    assert_eq!(explicit.diagram(), direct.diagram());
    let h2 = direct.diagram().intervals_in_dimension(2)?.next().unwrap();
    assert_eq!((h2.birth(), h2.end()), (1., IntervalEnd::Finite(2.)));
    let truncated = rips.max_edge_length(1.);
    assert!(matches!(
        truncated.persistence().max_filtration_value(2.).compute(),
        Err(Error::IncompleteFiltration { .. })
    ));
    assert!(matches!(
        truncated
            .prepare()?
            .persistence()
            .max_filtration_value(2.)
            .compute(),
        Err(Error::IncompleteFiltration { .. })
    ));
    assert!(matches!(
        truncated
            .build_complex(3)?
            .persistence()
            .max_filtration_value(2.)
            .compute(),
        Err(Error::IncompleteFiltration { .. })
    ));
    let zero = rips.build_complex(0)?;
    assert_eq!(zero.complex().len(), 6);
    assert!(matches!(
        zero.persistence().max_homology_dimension(0).compute(),
        Err(Error::InsufficientSkeleton { .. })
    ));
    Ok(())
}
#[test]
fn supplied_flag_expansion_preserves_essentiality_and_isolates() -> Result<()> {
    let graph = WeightedGraph::new(
        5,
        [[0, 1], [1, 2], [2, 3], [0, 3]]
            .into_iter()
            .map(|vertices| WeightedEdge {
                vertices,
                value: 1.,
            })
            .collect(),
    )?;
    let flag = FlagFiltration::new(graph);
    let filtration = flag.build_complex(1)?;
    assert!(filtration.is_dimension_complete());
    assert_eq!(
        filtration.context().filtration_kind(),
        FiltrationKind::SuppliedFlag
    );
    assert_eq!(filtration.context().vertex_count(), 5);
    let direct = flag.persistence().compute()?;
    let explicit = filtration.persistence().compute()?;
    assert_eq!(direct, explicit);
    assert_eq!(
        explicit
            .diagram()
            .intervals_in_dimension(0)?
            .filter(|i| i.end() == IntervalEnd::Essential)
            .count(),
        2
    );
    assert_eq!(
        explicit
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
fn approximation_keeps_blockers_mapping_hypotheses_and_callback_samples() -> Result<()> {
    let points = [0., 0., 1., 2., 4., 7.];
    let input = PointCloudView::new(&points, 6, 1)?;
    for policy in [
        MetricPolicy::Check,
        MetricPolicy::Assume,
        MetricPolicy::Unchecked,
    ] {
        for epsilon in [0.5, 1.5] {
            let approximate = ApproximateRipsBuilder::from_points(input, epsilon, policy)
                .min_insertion_radius(1.)
                .max_filtration_value(4.);
            let direct = approximate
                .persistence()
                .max_homology_dimension(2)
                .compute()?;
            let filtration = approximate.build_complex(3)?;
            let explicit = filtration
                .persistence()
                .max_homology_dimension(2)
                .compute()?;
            assert_eq!(direct, explicit);
            let calls = Cell::new(0);
            let prepared = ApproximateRipsBuilder::from_distance_fn(
                &points,
                |a, b| {
                    calls.set(calls.get() + 1);
                    Ok((a - b).abs())
                },
                epsilon,
                policy,
            )
            .min_insertion_radius(1.)
            .max_filtration_value(4.)
            .prepare()?;
            assert_eq!(calls.get(), 15);
            assert_eq!(
                prepared
                    .persistence()
                    .max_homology_dimension(2)
                    .compute()?
                    .diagram(),
                direct.diagram()
            );
            assert_eq!(
                prepared.approximation(),
                direct.context().approximation().unwrap()
            );
            assert!(prepared.approximation().retained_vertices().len() < points.len());
            assert_eq!(calls.get(), 15);
        }
    }
    Ok(())
}
#[test]
fn empty_duplicate_and_singleton_inputs_are_owned_and_reusable() -> Result<()> {
    for n in [0, 1, 4] {
        let points = vec![0.; n];
        let rips = RipsBuilder::from_points(PointCloudView::new(&points, n, 1)?);
        let explicit = rips.build_complex(usize::MAX)?;
        let expected = usize::from(n > 0);
        assert_eq!(explicit.context().vertex_count(), n);
        assert!(explicit.is_dimension_complete());
        let result = rips.persistence().max_homology_dimension(3).compute()?;
        assert_eq!(result.diagram().intervals().len(), expected);
        assert_eq!(
            result,
            explicit.persistence().max_homology_dimension(3).compute()?
        );
        std::thread::scope(|scope| {
            let a = scope.spawn(|| rips.persistence().compute().unwrap());
            let b = scope.spawn(|| rips.persistence().compute().unwrap());
            assert_eq!(a.join().unwrap(), b.join().unwrap());
        });
    }
    Ok(())
}
#[test]
fn setters_replace_values_and_invalid_requests_fail_before_callbacks() -> Result<()> {
    let values = [1.];
    let rips = RipsBuilder::from_distance_matrix(matrix(&values, 2))
        .max_edge_length(f64::NAN)
        .full_range();
    let requests = [RepresentativeRequest::new(
        0,
        0.,
        RepresentativeSelection::Both,
    )?];
    assert!(
        rips.persistence()
            .max_filtration_value(f64::NAN)
            .available_range()
            .representatives(&requests)
            .representatives(&[])
            .compute()?
            .representatives()
            .is_none()
    );
    let calls = Cell::new(0);
    let failure = ApproximateRipsBuilder::from_distance_fn(
        &values,
        |_, _| {
            calls.set(calls.get() + 1);
            Ok(0.)
        },
        0.5,
        MetricPolicy::Check,
    )
    .start_vertex(4)
    .prepare();
    assert!(failure.is_err());
    assert_eq!(calls.get(), 0);
    let failure = RipsBuilder::from_distance_fn(&[0, 1], |_, _| Err(Error::Cancelled)).prepare();
    assert!(matches!(failure, Err(Error::Cancelled)));
    Ok(())
}

#[test]
fn representative_scales_follow_certified_coverage_not_the_requested_cap() -> Result<()> {
    let values = [1.];
    let rips = RipsBuilder::from_distance_matrix(matrix(&values, 2));
    let requests = [RepresentativeRequest::new(
        0,
        3.,
        RepresentativeSelection::Both,
    )?];
    // All changes occurred by 1: a requested cap of 2 still certifies the entire
    // filtration, so the surviving component has a valid representative at 3.
    let result = rips
        .persistence()
        .max_filtration_value(2.)
        .representatives(&requests)
        .compute()?;
    assert_eq!(result.diagram().coverage(), Coverage::Complete);
    assert_eq!(result.representatives().unwrap().len(), 2);
    let explicit = rips
        .build_complex(1)?
        .persistence()
        .max_filtration_value(2.)
        .representatives(&requests)
        .compute()?;
    assert_eq!(result, explicit);
    let points = [0., 1.];
    let input = PointCloudView::new(&points, 2, 1)?;
    let point_result = RipsBuilder::from_points(input)
        .persistence()
        .max_filtration_value(2.)
        .representatives(&requests)
        .compute()?;
    assert_eq!(point_result.diagram(), result.diagram());
    let approximate = ApproximateRipsBuilder::from_points(input, 0.5, MetricPolicy::Check);
    assert_eq!(
        approximate
            .persistence()
            .max_filtration_value(2.)
            .representatives(&requests)
            .compute()?
            .representatives()
            .unwrap()
            .len(),
        2
    );
    assert!(matches!(
        rips.persistence()
            .max_filtration_value(0.5)
            .representatives(&requests)
            .compute(),
        Err(Error::QueryOutsideCoverage { .. })
    ));
    Ok(())
}

#[test]
fn expanded_negative_cutoff_does_not_create_zero_born_components() -> Result<()> {
    let values = [1.];
    let expanded = RipsBuilder::from_distance_matrix(matrix(&values, 2)).build_complex(1)?;
    let requests = [RepresentativeRequest::new(
        0,
        -1.,
        RepresentativeSelection::Both,
    )?];
    let early = expanded.persistence().max_filtration_value(-1.).compute()?;
    assert!(early.diagram().intervals().is_empty());
    assert_eq!(early.diagram().coverage(), Coverage::Through(-1.));
    let bases = expanded
        .persistence()
        .max_filtration_value(-1.)
        .representatives(&requests)
        .compute()?;
    assert_eq!(early.diagram(), bases.diagram());
    assert!(bases.representatives().unwrap().is_empty());
    Ok(())
}
