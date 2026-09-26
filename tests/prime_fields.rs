//! Independent prime-field ranks, torsion and representative basis contracts.
use cocycle::{
    Error,
    algebra::PrimeField,
    complex::{WeightedEdge, WeightedGraph},
    diagram::{IntervalEnd, PersistenceResult, RepresentativeKind},
    filtration::{FlagFiltration, threshold_rips_from_distances},
    geometry::{DissimilarityMatrixView, MatrixLayout, PointCloudView},
    persistence::{
        ExecutionLimits, PersistenceOptions, RepresentativeRequest, RepresentativeSelection,
        compute_expanded_rips_with_representatives, compute_flag,
        compute_flag_with_representatives, compute_rips_from_distances,
        compute_rips_from_distances_with_representatives,
        compute_rips_from_points_with_representatives, compute_threshold_rips,
        compute_threshold_rips_with_representatives,
    },
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

// Oracle arithmetic intentionally does not call production field operations.
fn mul(a: u32, b: u32, p: u32) -> u32 {
    (u128::from(a) * u128::from(b) % u128::from(p)) as u32
}
fn sub(a: u32, b: u32, p: u32) -> u32 {
    ((u64::from(a) + u64::from(p) - u64::from(b)) % u64::from(p)) as u32
}
fn inv(a: u32, p: u32) -> u32 {
    let (mut old_r, mut r) = (i128::from(p), i128::from(a));
    let (mut old_s, mut s) = (0, 1);
    while r != 0 {
        let q = old_r / r;
        (old_r, r) = (r, old_r - q * r);
        (old_s, s) = (s, old_s - q * s);
    }
    assert_eq!(old_r, 1);
    old_s.rem_euclid(i128::from(p)) as u32
}
fn rank(mut rows: Vec<Vec<u32>>, width: usize, p: u32) -> usize {
    let mut rank = 0;
    for column in 0..width {
        let Some(pivot) = (rank..rows.len()).find(|&r| rows[r][column] != 0) else {
            continue;
        };
        rows.swap(rank, pivot);
        let inverse = inv(rows[rank][column], p);
        for entry in &mut rows[rank] {
            *entry = mul(*entry, inverse, p);
        }
        for row in rank + 1..rows.len() {
            let factor = rows[row][column];
            let (earlier, current) = rows.split_at_mut(row);
            for (target, &source) in current[0].iter_mut().zip(&earlier[rank]).skip(column) {
                *target = sub(*target, mul(factor, source, p), p);
            }
        }
        rank += 1;
    }
    rank
}
fn simplices(graph: &WeightedGraph, q: usize, scale: f64) -> Vec<Vec<usize>> {
    // Enumerate vertex combinations independently of production clique access.
    fn extend(
        graph: &WeightedGraph,
        size: usize,
        scale: f64,
        prefix: &mut Vec<usize>,
        out: &mut Vec<Vec<usize>>,
    ) {
        if prefix.len() == size {
            out.push(prefix.clone());
            return;
        }
        let start = prefix.last().map_or(0, |v| v + 1);
        for v in start..graph.vertex_count() {
            if prefix
                .iter()
                .all(|&u| graph.edge_value(u, v).is_some_and(|w| w <= scale))
            {
                prefix.push(v);
                extend(graph, size, scale, prefix, out);
                prefix.pop();
            }
        }
    }
    let mut result = Vec::new();
    extend(graph, q + 1, scale, &mut Vec::new(), &mut result);
    result
}
fn boundaries(faces: &[Vec<usize>], cofaces: &[Vec<usize>], p: u32) -> Vec<Vec<u32>> {
    cofaces
        .iter()
        .map(|s| {
            let mut row = vec![0; faces.len()];
            if s.len() > 1 {
                for omitted in 0..s.len() {
                    let mut face = s.clone();
                    face.remove(omitted);
                    let i = faces.iter().position(|v| v == &face).unwrap();
                    row[i] = if omitted % 2 == 0 { 1 } else { p - 1 };
                }
            }
            row
        })
        .collect()
}
fn betti(graph: &WeightedGraph, q: usize, scale: f64, p: u32) -> usize {
    let cells = simplices(graph, q, scale);
    let lower = if q == 0 {
        Vec::new()
    } else {
        simplices(graph, q - 1, scale)
    };
    let upper = simplices(graph, q + 1, scale);
    cells.len()
        - rank(boundaries(&lower, &cells, p), lower.len(), p)
        - rank(boundaries(&cells, &upper, p), cells.len(), p)
}
fn active(end: IntervalEnd, scale: f64) -> bool {
    match end {
        IntervalEnd::Finite(d) => scale < d,
        IntervalEnd::Essential => true,
        IntervalEnd::RightCensored { through } => scale <= through,
    }
}
fn verify_basis(
    graph: &WeightedGraph,
    result: &PersistenceResult,
    requests: &[RepresentativeRequest],
) {
    let p = result.context().characteristic();
    let representatives = result.representatives().unwrap();
    for (request_index, request) in requests.iter().enumerate() {
        let q = request.dimension();
        let scale = request.scale();
        let cells = simplices(graph, q, scale);
        let upper = simplices(graph, q + 1, scale);
        let lower = if q == 0 {
            Vec::new()
        } else {
            simplices(graph, q - 1, scale)
        };
        let boundary = boundaries(&cells, &upper, p);
        let downward = boundaries(&lower, &cells, p);
        let expected: BTreeSet<_> = result
            .diagram()
            .intervals()
            .iter()
            .enumerate()
            .filter(|(_, i)| i.dimension() == q && i.birth() <= scale && active(i.end(), scale))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(expected.len(), betti(graph, q, scale, p));
        let mut cycles = BTreeMap::new();
        let mut cocycles = BTreeMap::new();
        for rep in representatives
            .iter()
            .filter(|r| r.request_index() == request_index)
        {
            assert_eq!(rep.dimension(), q);
            assert_eq!(rep.scale(), scale);
            assert_eq!(rep.characteristic(), p);
            assert!(expected.contains(&rep.interval_index()));
            let mut vector = vec![0; cells.len()];
            for term in rep.terms() {
                assert!(term.coefficient() > 0 && term.coefficient() < p);
                assert_eq!(term.vertices().len(), q + 1);
                assert!(term.vertices().windows(2).all(|v| v[0] < v[1]));
                assert!(term.vertices().iter().all(|&v| v < graph.vertex_count()));
                let i = cells.iter().position(|v| v == term.vertices()).unwrap();
                assert_eq!(vector[i], 0);
                vector[i] = term.coefficient();
            }
            match rep.kind() {
                RepresentativeKind::Cycle => {
                    for (face, _) in lower.iter().enumerate() {
                        let sum = vector.iter().enumerate().fold(0_u64, |sum, (i, &c)| {
                            (sum + u64::from(mul(c, downward[i][face], p))) % u64::from(p)
                        });
                        assert_eq!(sum, 0, "cycle not closed");
                    }
                    let interval = result.diagram().intervals()[rep.interval_index()];
                    for term in rep.terms() {
                        for &a in term.vertices() {
                            for &b in term.vertices() {
                                if a != b {
                                    assert!(graph.edge_value(a, b).unwrap() <= interval.birth());
                                }
                            }
                        }
                    }
                    if let IntervalEnd::Finite(death) = interval.end() {
                        let at_death = simplices(graph, q, death);
                        let mut relations =
                            boundaries(&at_death, &simplices(graph, q + 1, death), p);
                        let before = rank(relations.clone(), at_death.len(), p);
                        let mut cycle = vec![0; at_death.len()];
                        for term in rep.terms() {
                            cycle[at_death.iter().position(|v| v == term.vertices()).unwrap()] =
                                term.coefficient();
                        }
                        relations.push(cycle);
                        assert_eq!(
                            rank(relations, at_death.len(), p),
                            before,
                            "cycle does not die at its associated death"
                        );
                    }
                    assert!(cycles.insert(rep.interval_index(), vector).is_none());
                }
                RepresentativeKind::Cocycle => {
                    for relation in &boundary {
                        let sum = vector.iter().zip(relation).fold(0_u64, |sum, (&a, &b)| {
                            (sum + u64::from(mul(a, b, p))) % u64::from(p)
                        });
                        assert_eq!(sum, 0, "cocycle not closed");
                    }
                    assert!(cocycles.insert(rep.interval_index(), vector).is_none());
                }
            }
        }
        if request.selection() != RepresentativeSelection::Cocycles {
            assert_eq!(cycles.keys().copied().collect::<BTreeSet<_>>(), expected);
            let old_rank = rank(boundary.clone(), cells.len(), p);
            let mut extended = boundary.clone();
            extended.extend(cycles.values().cloned());
            assert_eq!(
                rank(extended, cells.len(), p),
                old_rank + expected.len(),
                "cycles dependent modulo boundaries"
            );
        }
        if request.selection() != RepresentativeSelection::Cycles {
            assert_eq!(cocycles.keys().copied().collect::<BTreeSet<_>>(), expected);
            // Coboundaries are transposes of the downward boundary operator.
            let mut coboundaries: Vec<_> = (0..lower.len())
                .map(|i| downward.iter().map(|c| c[i]).collect())
                .collect();
            let old_rank = rank(coboundaries.clone(), cells.len(), p);
            coboundaries.extend(cocycles.values().cloned());
            assert_eq!(
                rank(coboundaries, cells.len(), p),
                old_rank + expected.len()
            );
        }
        if request.selection() == RepresentativeSelection::Both {
            for (&i, cycle) in &cycles {
                for (&j, cocycle) in &cocycles {
                    let sum = cycle.iter().zip(cocycle).fold(0_u64, |sum, (&a, &b)| {
                        (sum + u64::from(mul(a, b, p))) % u64::from(p)
                    });
                    assert_eq!(sum, u64::from(i == j), "dual pairing");
                }
            }
        }
    }
}
fn options(q: usize, p: u32, cutoff: Option<f64>) -> PersistenceOptions {
    PersistenceOptions::new(q, cutoff)
        .unwrap()
        .with_field(PrimeField::new(p).unwrap())
}
fn requests(q: usize, scales: &[f64]) -> Vec<RepresentativeRequest> {
    (0..=q)
        .flat_map(|dim| {
            scales.iter().map(move |&scale| {
                RepresentativeRequest::new(dim, scale, RepresentativeSelection::Both).unwrap()
            })
        })
        .collect()
}
fn graph_from_values(n: usize, values: &[f64]) -> WeightedGraph {
    WeightedGraph::new(
        n,
        (0..n)
            .flat_map(|b| {
                (0..b).map(move |a| WeightedEdge {
                    vertices: [a, b],
                    value: values[b * (b - 1) / 2 + a],
                })
            })
            .collect(),
    )
    .unwrap()
}
fn rp2() -> WeightedGraph {
    // Barycentric subdivision of the six-vertex RP2 triangulation in the bibliography.
    let facets = [
        [0, 1, 2],
        [0, 1, 3],
        [0, 2, 4],
        [0, 3, 5],
        [0, 4, 5],
        [1, 2, 5],
        [1, 3, 4],
        [1, 4, 5],
        [2, 3, 4],
        [2, 3, 5],
    ];
    let mut faces = BTreeSet::new();
    for facet in facets {
        for mask in 1..8 {
            faces.insert(
                facet
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(_, v)| *v)
                    .collect::<Vec<_>>(),
            );
        }
    }
    let faces: Vec<_> = faces.into_iter().collect();
    assert_eq!(faces.len(), 31);
    let mut edges = Vec::new();
    for b in 0..faces.len() {
        for a in 0..b {
            if faces[a].iter().all(|v| faces[b].contains(v))
                || faces[b].iter().all(|v| faces[a].contains(v))
            {
                edges.push(WeightedEdge {
                    vertices: [a, b],
                    value: 1.,
                });
            }
        }
    }
    WeightedGraph::new(faces.len(), edges).unwrap()
}
#[test]
fn prime_validation_and_full_u32_arithmetic() {
    for bad in [0, 1, 4, 9, 25, 341, 561, 1105, 65535, u32::MAX] {
        assert!(PrimeField::new(bad).is_err());
    }
    for p in [2, 3, 5, 251, 65537, 2147483647, 4294967291] {
        let field = PrimeField::new(p).unwrap();
        for a in [0, 1, 2, p - 1, p, u32::MAX] {
            for b in [0, 1, 2, p - 1, p, u32::MAX] {
                assert_eq!(field.multiply(a, b), mul(a, b, p));
                assert_eq!(
                    u128::from(field.add(a, b)),
                    (u128::from(a) + u128::from(b)) % u128::from(p)
                );
                assert_eq!(field.subtract(a, b), sub(a % p, b % p, p));
            }
            if a % p == 0 {
                assert!(field.inverse(a).is_err());
            } else {
                assert_eq!(field.inverse(a).unwrap(), inv(a % p, p));
            }
        }
    }
}
#[test]
fn torsion_changes_the_diagram_and_representatives() {
    let graph = rp2();
    assert_eq!(simplices(&graph, 2, 1.).len(), 60);
    assert!(simplices(&graph, 3, 1.).is_empty());
    let filtration = FlagFiltration::new(graph.clone());
    let queries = requests(2, &[0., 1., 2.]);
    for p in [2, 3, 5, 4294967291] {
        let options = options(2, p, None);
        let result = compute_flag_with_representatives(
            &filtration,
            &options,
            &queries,
            &ExecutionLimits::default(),
        )
        .unwrap();
        assert_eq!(
            result.diagram(),
            compute_flag(&filtration, &options, &ExecutionLimits::default())
                .unwrap()
                .diagram()
        );
        for q in [1, 2] {
            assert_eq!(betti(&graph, q, 1., p), usize::from(p == 2));
            assert_eq!(
                result.diagram().intervals_in_dimension(q).unwrap().count(),
                usize::from(p == 2)
            );
        }
        verify_basis(&graph, &result, &queries);
    }
}
#[test]
fn finite_cycles_die_with_their_intervals_and_paths_agree() {
    for p in [2, 3, 5, 4294967291] {
        let values: Vec<_> = (0..6)
            .flat_map(|b| (0..b).map(move |a| if a / 2 == b / 2 { 2. } else { 1. }))
            .collect();
        let matrix = DissimilarityMatrixView::new(&values, 6, MatrixLayout::LowerTriangle).unwrap();
        let graph = graph_from_values(6, &values);
        let queries = requests(2, &[0., 1., 1.5, 2., 3.]);
        let options = options(2, p, None);
        let source = threshold_rips_from_distances(matrix, None).unwrap();
        let expanded = source.expand(3).unwrap();
        let limits = ExecutionLimits::default();
        let result =
            compute_rips_from_distances_with_representatives(matrix, &options, &queries, &limits)
                .unwrap();
        for other in [
            compute_threshold_rips_with_representatives(&source, &options, &queries, &limits)
                .unwrap(),
            compute_expanded_rips_with_representatives(&expanded, &options, &queries, &limits)
                .unwrap(),
        ] {
            assert_eq!(result.diagram(), other.diagram());
            assert_eq!(result.representatives(), other.representatives());
        }
        assert_eq!(
            result.diagram(),
            compute_rips_from_distances(matrix, &options, &limits)
                .unwrap()
                .diagram()
        );
        assert_eq!(
            result.diagram(),
            compute_threshold_rips(&source, &options, &limits)
                .unwrap()
                .diagram()
        );
        verify_basis(&graph, &result, &queries);
    }
}
#[test]
fn odd_prime_random_filtrations_match_independent_ranks() {
    let mut seed = 91_u64;
    for sample in 0..32 {
        let n = 4 + sample % 3;
        let values: Vec<_> = (0..n * (n - 1) / 2)
            .map(|_| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                ((seed >> 32) % 5) as f64
            })
            .collect();
        let matrix = DissimilarityMatrixView::new(&values, n, MatrixLayout::LowerTriangle).unwrap();
        let graph = graph_from_values(n, &values);
        let p = [3, 5, 7, 4294967291][sample % 4];
        let options = options(2, p, None);
        let queries = requests(2, &[0., 1., 2., 3., 4.]);
        let result = compute_rips_from_distances_with_representatives(
            matrix,
            &options,
            &queries,
            &ExecutionLimits::default(),
        )
        .unwrap();
        assert_eq!(
            result.diagram(),
            compute_rips_from_distances(matrix, &options, &ExecutionLimits::default())
                .unwrap()
                .diagram(),
            "sample={sample}"
        );
        verify_basis(&graph, &result, &queries);
    }
}
#[test]
fn request_validation_censoring_and_execution_are_explicit() {
    for bad in [f64::NAN, f64::INFINITY] {
        assert!(RepresentativeRequest::new(1, bad, RepresentativeSelection::Both).is_err());
    }
    let values = [1., 2., 1., 1., 2., 1.];
    let matrix = DissimilarityMatrixView::new(&values, 4, MatrixLayout::LowerTriangle).unwrap();
    let source = threshold_rips_from_distances(matrix, Some(1.)).unwrap();
    let options = options(1, 3, None);
    let queries = requests(1, &[0., 1.]);
    let limits = ExecutionLimits::default();
    let result =
        compute_threshold_rips_with_representatives(&source, &options, &queries, &limits).unwrap();
    verify_basis(source.graph(), &result, &queries);
    assert!(
        result
            .diagram()
            .intervals_in_dimension(1)
            .unwrap()
            .all(|i| i.end() == IntervalEnd::RightCensored { through: 1. })
    );
    let outside = [RepresentativeRequest::new(1, 2., RepresentativeSelection::Both).unwrap()];
    assert!(matches!(
        compute_threshold_rips_with_representatives(&source, &options, &outside, &limits),
        Err(Error::QueryOutsideCoverage { .. })
    ));
    let too_high = [RepresentativeRequest::new(2, 1., RepresentativeSelection::Both).unwrap()];
    assert!(matches!(
        compute_threshold_rips_with_representatives(&source, &options, &too_high, &limits),
        Err(Error::DimensionNotComputed { .. })
    ));
    for limit in [0, 1, 10, 100] {
        assert!(matches!(
            compute_threshold_rips_with_representatives(
                &source,
                &options,
                &queries,
                &ExecutionLimits::new(Some(limit), None)
            ),
            Err(Error::WorkLimitExceeded { .. })
        ));
    }
    let cancel = AtomicBool::new(true);
    assert!(matches!(
        compute_threshold_rips_with_representatives(
            &source,
            &options,
            &queries,
            &ExecutionLimits::new(None, Some(&cancel))
        ),
        Err(Error::Cancelled)
    ));
    assert_eq!(
        result,
        compute_threshold_rips_with_representatives(&source, &options, &queries, &limits).unwrap()
    );
    assert!(
        compute_threshold_rips(&source, &options, &limits)
            .unwrap()
            .representatives()
            .is_none()
    );
}
#[test]
fn duplicate_intervals_and_selections_keep_local_identity() {
    let graph = WeightedGraph::new(
        5,
        vec![
            WeightedEdge {
                vertices: [0, 1],
                value: 1.,
            },
            WeightedEdge {
                vertices: [1, 2],
                value: 1.,
            },
        ],
    )
    .unwrap();
    let filtration = FlagFiltration::new(graph.clone());
    let queries = [
        RepresentativeRequest::new(0, 0., RepresentativeSelection::Cycles).unwrap(),
        RepresentativeRequest::new(0, 0., RepresentativeSelection::Cocycles).unwrap(),
        RepresentativeRequest::new(0, 0., RepresentativeSelection::Both).unwrap(),
        RepresentativeRequest::new(0, 0., RepresentativeSelection::Both).unwrap(),
    ];
    let result = compute_flag_with_representatives(
        &filtration,
        &options(0, 5, None),
        &queries,
        &ExecutionLimits::default(),
    )
    .unwrap();
    verify_basis(&graph, &result, &queries);
    assert_eq!(result.representatives().unwrap().len(), 30);
    let repeated = result
        .representatives()
        .unwrap()
        .iter()
        .filter(|r| r.request_index() == 2)
        .map(|r| (r.interval_index(), r.kind(), r.terms()))
        .collect::<Vec<_>>();
    assert_eq!(
        repeated,
        result
            .representatives()
            .unwrap()
            .iter()
            .filter(|r| r.request_index() == 3)
            .map(|r| (r.interval_index(), r.kind(), r.terms()))
            .collect::<Vec<_>>()
    );
}
#[test]
fn empty_requests_and_point_inputs_preserve_ownership() {
    let queries = requests(1, &[0., 1.]);
    for n in 0..=1 {
        let graph = FlagFiltration::new(WeightedGraph::new(n, vec![]).unwrap());
        let result = compute_flag_with_representatives(
            &graph,
            &options(1, 3, None),
            &queries,
            &ExecutionLimits::default(),
        )
        .unwrap();
        verify_basis(graph.graph(), &result, &queries);
        if n == 0 {
            assert_eq!(result.representatives(), Some([].as_slice()));
        }
    }
    let points = vec![0., 0., 1., 0., 1., 1., 0., 1.];
    let view = PointCloudView::new(&points, 4, 2).unwrap();
    let query = [RepresentativeRequest::new(1, 1., RepresentativeSelection::Both).unwrap()];
    let result = compute_rips_from_points_with_representatives(
        view,
        &options(1, 3, Some(1.)),
        &query,
        &ExecutionLimits::default(),
    )
    .unwrap();
    drop(points);
    assert_eq!(result.representatives().unwrap().len(), 2);
    assert_eq!(result.context().characteristic(), 3);
}
