use super::*;
use crate::diagram::{Coverage, IntervalEnd};
use crate::persistence::reference::FilteredBoundary;
use crate::persistence::rips::resolve_rips_range;
use crate::persistence::{RipsOptions, assemble_diagram, reference};

fn compare_all(values: &[f64], n: usize, cutoff: Option<f64>) {
    let input = DissimilarityView::new(values, n).unwrap();
    let options = RipsOptions::new(1, cutoff).unwrap();
    let expected = reference::compute(input, &options).unwrap();
    let (cutoff, coverage) = resolve_rips_range(input, &options);
    macro_rules! check {
        ($implicit:literal, $clear:literal, $cone:literal, $short:ident) => {
            check!($implicit, $clear, $cone, $short, false)
        };
        ($implicit:literal, $clear:literal, $cone:literal, $short:ident, $two_pass:literal) => {{
            let mut stats = Stats {
                two_pass_initialization: $two_pass,
                ..Stats::default()
            };
            let raw = run::<$implicit, $clear, $cone, $short>(input, cutoff, &mut stats).unwrap();
            assert_eq!(
                assemble_diagram(1, coverage, raw).unwrap(),
                expected,
                "n={n} cutoff={cutoff} implicit={} clear={} cone={} shortcuts={} two_pass={} values={values:?}",
                $implicit,
                $clear,
                $cone,
                $short,
                $two_pass
            );
        }};
    }
    check!(false, false, false, NO_SHORTCUTS);
    check!(false, true, false, NO_SHORTCUTS);
    check!(true, true, false, NO_SHORTCUTS);
    check!(true, true, true, NO_SHORTCUTS);
    check!(true, true, true, APPARENT);
    check!(true, true, true, EMERGENT);
    check!(true, true, true, APPARENT_EMERGENT);
    check!(true, true, true, APPARENT_EMERGENT, true);
    check!(true, true, true, VIRTUAL_APPARENT);
    check!(true, true, true, ALL_SHORTCUTS);
    check!(true, true, true, ALL_SHORTCUTS, true);
    check!(false, true, false, ALL_SHORTCUTS);
}

#[test]
fn every_three_level_four_vertex_filtration_matches_reference_with_each_optimization() {
    for code in 0..3_usize.pow(6) {
        let mut remaining = code;
        let values: Vec<_> = (0..6)
            .map(|_| {
                let value = (remaining % 3) as f64;
                remaining /= 3;
                value
            })
            .collect();
        for cutoff in [None, Some(0.0), Some(1.0), Some(1.5)] {
            compare_all(&values, 4, cutoff);
        }
    }
}

#[test]
fn randomized_f64_nonmetric_filtrations_and_ties_match_each_optimization() {
    let mut state = 173_u64;
    for n in 0_usize..=12 {
        for sample in 0..24 {
            let values: Vec<_> = (0..n * n.saturating_sub(1) / 2)
                .map(|_| {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    if sample % 2 == 0 {
                        ((state >> 32) % 9) as f64 / 4.0
                    } else {
                        ((state >> 11) as f64) / ((1_u64 << 52) as f64)
                    }
                })
                .collect();
            for cutoff in [None, Some(0.0), Some(0.75), Some(1.0), Some(1.8)] {
                compare_all(&values, n, cutoff);
            }
        }
    }
}

#[test]
fn extreme_scales_and_adjacent_f64_values_do_not_quantize_or_overflow() {
    for base in [f64::from_bits(1), 1.0, 1.0e300] {
        let next = f64::from_bits(base.to_bits() + 1);
        let values = [base, next, base, base, next, base];
        for cutoff in [None, Some(base), Some(next)] {
            compare_all(&values, 4, cutoff);
        }
    }
    compare_all(&[-0.0, f64::MAX, 0.0, f64::MAX, f64::MAX, -0.0], 4, None);
}

#[test]
fn cone_stopping_preserves_user_coverage_and_closed_boundary() {
    // Vertex zero is a cone point at 1, while the full input diameter is 5.
    let values = [1., 1., 5., 1., 5., 5.];
    let input = DissimilarityView::new(&values, 4).unwrap();
    assert_eq!(cone_radius(input.into(), &mut || Ok(())).unwrap(), 1.0);
    for cutoff in [0.5, 1.0, 2.0, 5.0] {
        compare_all(&values, 4, Some(cutoff));
        let options = RipsOptions::new(1, Some(cutoff)).unwrap();
        let result = crate::persistence::rips_from_dissimilarities(input, &options).unwrap();
        if (1.0..5.0).contains(&cutoff) {
            assert_eq!(result.coverage(), Coverage::Through(cutoff));
            assert_eq!(result.intervals().len(), 4);
            assert!(
                result
                    .intervals()
                    .iter()
                    .any(|bar| bar.end() == IntervalEnd::RightCensored { through: cutoff })
            );
        }
    }
}

#[test]
fn large_cycles_and_disconnected_components_survive_truncation() {
    for n in [16, 24] {
        let values: Vec<_> = (0..n)
            .flat_map(|b| {
                (0..b).map(move |a| {
                    // Two disjoint cycle graphs until scale 3; a complete graph at 4.
                    let half = n / 2;
                    if a / half != b / half {
                        4.0
                    } else if b - a == 1 || b - a == half - 1 {
                        1.0
                    } else {
                        3.0
                    }
                })
            })
            .collect();
        for cutoff in [None, Some(0.0), Some(1.0), Some(2.0), Some(3.0)] {
            compare_all(&values, n, cutoff);
        }
    }
}

#[test]
fn heap_entries_cancel_by_parity_instead_of_set_deduplication() {
    let mut heap = BinaryHeap::from(vec![5, 5, 4, 4, 4, 2, 2]);
    assert_eq!(
        pop_parity(
            &mut heap,
            &mut WorkBudget::new(&crate::persistence::ExecutionLimits::default()).unwrap()
        )
        .unwrap(),
        Some(4)
    );
    assert_eq!(
        pop_parity(
            &mut heap,
            &mut WorkBudget::new(&crate::persistence::ExecutionLimits::default()).unwrap()
        )
        .unwrap(),
        None
    );
}

#[test]
fn original_column_initialization_reuses_storage_and_checks_only_the_first_equal_cofacet() {
    // Edge 01 has cofacets 014 at value 2, then 013 and 012 at value 1.
    let values = [1., 1., 1., 1., 1., 2., 2., 2., 2., 2.];
    let input = DissimilarityView::new(&values, 5).unwrap();
    let access = DenseFlag::new(input.into(), 2.).unwrap();
    let edge = SimplexEntry { id: 0, value: 1. };
    let first_equal = SimplexEntry { id: 1, value: 1. };
    let mut working = Coboundary::with_capacity(16);
    working.push(Reverse(SimplexEntry { id: 99, value: 9. }));
    let capacity = working.capacity();
    let mut stats = Stats::default();
    let mut budget = WorkBudget::new(&crate::persistence::ExecutionLimits::default()).unwrap();
    assert_eq!(
        initialize_coboundary::<APPARENT_EMERGENT>(
            &access,
            edge,
            &HashMap::new(),
            &mut working,
            &mut stats,
            &mut budget,
        )
        .unwrap(),
        (Some(first_equal), false)
    );
    assert!(working.is_empty());
    assert_eq!(working.capacity(), capacity);
    assert_eq!(stats.initial_candidates, 2);
    assert_eq!(stats.cofacets, 2);

    // The next equal cofacet is unowned, but cannot replace an occupied pivot.
    let owners = HashMap::from([(first_equal.id, ColumnPosition(0))]);
    let mut stats = Stats::default();
    assert_eq!(
        initialize_coboundary::<APPARENT_EMERGENT>(
            &access,
            edge,
            &owners,
            &mut working,
            &mut stats,
            &mut budget,
        )
        .unwrap(),
        (None, false)
    );
    assert_eq!(stats.initial_candidates, 5);
    assert_eq!(stats.cofacets, 3);
    assert_eq!(working.pop(), Some(Reverse(first_equal)));
    assert_eq!(
        working.pop(),
        Some(Reverse(SimplexEntry { id: 0, value: 1. }))
    );
    assert_eq!(
        working.pop(),
        Some(Reverse(SimplexEntry { id: 4, value: 2. }))
    );
    assert!(working.is_empty());
}

#[test]
fn original_column_fallback_handles_empty_no_equal_and_apparent_only_rejection() {
    for (values, cutoff, edge, expected_rows) in [
        (vec![1., 2., 2.], 1., SimplexEntry { id: 0, value: 1. }, 0),
        (vec![1., 2., 2.], 2., SimplexEntry { id: 0, value: 1. }, 1),
        (vec![1., 1., 1.], 1., SimplexEntry { id: 1, value: 1. }, 1),
    ] {
        let input = DissimilarityView::new(&values, 3).unwrap();
        let access = DenseFlag::new(input.into(), cutoff).unwrap();
        let mut working = Coboundary::new();
        let mut stats = Stats::default();
        let mut budget = WorkBudget::new(&crate::persistence::ExecutionLimits::default()).unwrap();
        assert_eq!(
            initialize_coboundary::<APPARENT>(
                &access,
                edge,
                &HashMap::new(),
                &mut working,
                &mut stats,
                &mut budget,
            )
            .unwrap(),
            (None, false)
        );
        assert_eq!(working.len(), expected_rows);
        assert_eq!(stats.initial_candidates, 3);
    }
}

#[test]
fn single_pass_visits_failed_prefixes_once_and_reuses_alternating_buffers() {
    let mut working = Coboundary::with_capacity(128);
    let capacity = working.capacity();
    let mut large = vec![2.; 64 * 63 / 2];
    large[0] = 1.;
    // Alternate a large column, a rejected and a successful shortcut after a
    // high prefix, an empty column, and a column without an equal cofacet.
    for (n, values, cutoff, occupied, visits) in [
        (64, large, 2., false, [128, 64]),
        (
            5,
            vec![1., 1., 1., 1., 1., 2., 2., 2., 2., 2.],
            2.,
            true,
            [7, 5],
        ),
        (
            5,
            vec![1., 1., 1., 1., 1., 2., 2., 2., 2., 2.],
            2.,
            false,
            [2, 2],
        ),
        (3, vec![1., 2., 2.], 1., false, [6, 3]),
        (3, vec![1., 2., 2.], 2., false, [6, 3]),
    ] {
        let input = DissimilarityView::new(&values, n).unwrap();
        let access = DenseFlag::new(input.into(), cutoff).unwrap();
        let mut owners = HashMap::new();
        if occupied {
            owners.insert(1, ColumnPosition(0));
        }
        let mut outcomes = Vec::new();
        for (two_pass, expected_visits) in [true, false].into_iter().zip(visits) {
            let mut stats = Stats {
                two_pass_initialization: two_pass,
                ..Stats::default()
            };
            let mut budget =
                WorkBudget::new(&crate::persistence::ExecutionLimits::default()).unwrap();
            let result = initialize_coboundary::<APPARENT_EMERGENT>(
                &access,
                SimplexEntry { id: 0, value: 1. },
                &owners,
                &mut working,
                &mut stats,
                &mut budget,
            )
            .unwrap();
            assert_eq!(stats.initial_candidates, expected_visits);
            assert_eq!(working.capacity(), capacity);
            outcomes.push((result, working.clone().into_sorted_vec()));
        }
        assert_eq!(outcomes[0], outcomes[1]);
    }
}

#[test]
fn initial_scan_work_failure_does_not_poison_reused_input_or_heap() {
    let values = [1., 2., 2.];
    let input = DissimilarityView::new(&values, 3).unwrap();
    let access = DenseFlag::new(input.into(), 2.).unwrap();
    let edge = SimplexEntry { id: 0, value: 1. };
    let mut working = Coboundary::new();
    let mut budget =
        WorkBudget::new(&crate::persistence::ExecutionLimits::new(Some(1), None)).unwrap();
    assert_eq!(
        initialize_coboundary::<APPARENT_EMERGENT>(
            &access,
            edge,
            &HashMap::new(),
            &mut working,
            &mut Stats::default(),
            &mut budget,
        )
        .unwrap_err(),
        Error::WorkLimitExceeded { limit: 1 }
    );
    let mut budget = WorkBudget::new(&crate::persistence::ExecutionLimits::default()).unwrap();
    assert_eq!(
        initialize_coboundary::<APPARENT_EMERGENT>(
            &access,
            edge,
            &HashMap::new(),
            &mut working,
            &mut Stats::default(),
            &mut budget,
        )
        .unwrap(),
        (None, false)
    );
    assert_eq!(
        working.pop(),
        Some(Reverse(SimplexEntry { id: 0, value: 2. }))
    );
}

#[test]
fn raw_h1_output_omits_zero_bars_without_omitting_repeated_positive_bars() {
    let equal = [1.; 15];
    let input = DissimilarityView::new(&equal, 6).unwrap();
    let mut stats = Stats::default();
    let raw = run::<true, true, false, APPARENT_EMERGENT>(input, 1., &mut stats).unwrap();
    assert!(stats.shortcuts > 0);
    assert!(raw.iter().all(|&(dimension, _, _)| dimension == 0));

    // K(3,3) has four independent cycles at 1, all dying at 2.
    let values: Vec<_> = (0..6)
        .flat_map(|b| (0..b).map(move |a| if a / 3 == b / 3 { 2. } else { 1. }))
        .collect();
    let input = DissimilarityView::new(&values, 6).unwrap();
    let raw =
        run::<true, true, false, APPARENT_EMERGENT>(input, 2., &mut Stats::default()).unwrap();
    assert_eq!(
        raw.iter().filter(|&&bar| bar == (1, 1., Some(2.))).count(),
        4
    );
    assert!(
        raw.iter()
            .all(|&(dimension, birth, death)| dimension == 0 || death != Some(birth))
    );
}

fn check_virtual_access(
    access: &impl FlagAccess,
    coverage: Coverage,
    expected: &crate::diagram::PersistenceDiagram,
) -> Stats {
    let mut single_pass = Stats::default();
    for two_pass in [true, false] {
        let mut ordinary = Stats {
            two_pass_initialization: two_pass,
            verify_transforms: true,
            ..Stats::default()
        };
        let mut optimized = Stats {
            two_pass_initialization: two_pass,
            verify_transforms: true,
            ..Stats::default()
        };
        for (shortcuts, stats) in [
            (APPARENT_EMERGENT, &mut ordinary),
            (ALL_SHORTCUTS, &mut optimized),
        ] {
            let mut budget =
                WorkBudget::new(&crate::persistence::ExecutionLimits::default()).unwrap();
            let raw = if shortcuts == APPARENT_EMERGENT {
                run_access::<true, true, APPARENT_EMERGENT>(access, stats, &mut budget)
            } else {
                run_access::<true, true, ALL_SHORTCUTS>(access, stats, &mut budget)
            }
            .unwrap();
            assert_eq!(&assemble_diagram(1, coverage, raw).unwrap(), expected);
        }
        assert!(optimized.stored_columns <= ordinary.stored_columns);
        if optimized.skipped_apparent > 0 {
            assert!(optimized.stored_columns < ordinary.stored_columns);
        }
        single_pass = optimized;
    }
    single_pass
}

/// Check R = C V with independent set XOR rather than the production heap.
/// This catches missing virtual terms even when re-eliminating their pivots
/// happens to leave the final interval multiset unchanged.
pub(super) fn check_transform(
    access: &impl FlagAccess,
    edges: &[SimplexEntry],
    edge: SimplexEntry,
    additions: &[EdgePosition],
    working: &Coboundary,
    pivot: SimplexEntry,
) {
    let mut expected = std::collections::BTreeSet::new();
    for source in std::iter::once(edge).chain(additions.iter().map(|position| edges[position.0])) {
        access
            .visit_cofacets(source, &mut || Ok(()), |row| {
                if !expected.insert(row) {
                    expected.remove(&row);
                }
                Ok(true)
            })
            .unwrap();
    }
    let mut actual = std::collections::BTreeSet::new();
    for row in working
        .iter()
        .map(|row| row.0)
        .chain(std::iter::once(pivot))
    {
        if !actual.insert(row) {
            actual.remove(&row);
        }
    }
    assert_eq!(
        actual, expected,
        "stored transformation must reconstruct its full column"
    );
}

#[test]
fn virtual_pairs_cancel_and_reconstruct_on_dense_and_sparse_tied_cycles() {
    use crate::filtration::flag::SparseFlag;
    use crate::filtration::threshold_rips_from_distances;

    let n = 12;
    // Integer geodesic distances on a cycle provide ties without rounding.
    let values: Vec<_> = (0..n)
        .flat_map(|b| (0..b).map(move |a| (b - a).min(n - b + a) as f64))
        .collect();
    let input = DissimilarityView::new(&values, n).unwrap();
    let mut virtual_additions = 0;
    let mut reconstructions = 0;
    let mut checked_transforms = 0;
    for cutoff in [1., 2., 4., 6.] {
        let options = RipsOptions::new(1, Some(cutoff)).unwrap();
        let expected = reference::compute(input, &options).unwrap();
        let (_, coverage) = resolve_rips_range(input, &options);
        let dense = DenseFlag::new(input.into(), cutoff).unwrap();
        let graph = threshold_rips_from_distances(input.into(), Some(cutoff)).unwrap();
        let sparse = SparseFlag::new(graph.graph(), cutoff).unwrap();
        for stats in [
            check_virtual_access(&dense, coverage, &expected),
            check_virtual_access(&sparse, coverage, &expected),
        ] {
            virtual_additions += stats.virtual_additions;
            reconstructions += stats.column_additions;
            checked_transforms += stats.checked_transforms;
        }
    }
    assert!(
        virtual_additions > 1,
        "virtual additions={virtual_additions}"
    );
    assert!(
        reconstructions > 0,
        "ordinary reconstructions={reconstructions}"
    );
    assert!(checked_transforms > 0);
}

struct CancelAccess<'a, A> {
    inner: &'a A,
    flag: &'a std::sync::atomic::AtomicBool,
    cancel_at: usize,
    candidates: std::cell::Cell<usize>,
}

impl<A: FlagAccess> FlagAccess for CancelAccess<'_, A> {
    fn vertex_count(&self) -> usize {
        self.inner.vertex_count()
    }
    fn edges(&self, checkpoint: &mut impl FnMut() -> Result<()>) -> Result<Vec<SimplexEntry>> {
        self.inner.edges(checkpoint)
    }
    fn edge_vertices(&self, id: usize) -> [usize; 2] {
        self.inner.edge_vertices(id)
    }
    fn latest_facet(&self, triangle: SimplexEntry) -> SimplexEntry {
        self.inner.latest_facet(triangle)
    }
    fn visit_cofacets(
        &self,
        edge: SimplexEntry,
        checkpoint: &mut impl FnMut() -> Result<()>,
        visitor: impl FnMut(SimplexEntry) -> Result<bool>,
    ) -> Result<()> {
        self.inner.visit_cofacets(
            edge,
            &mut || {
                let count = self.candidates.get() + 1;
                self.candidates.set(count);
                if count == self.cancel_at {
                    self.flag.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                checkpoint()
            },
            visitor,
        )
    }
}

#[test]
fn cancellation_at_each_cofacet_checkpoint_preserves_input_and_reentrancy() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let n = 8;
    let values: Vec<_> = (0..n)
        .flat_map(|b| (0..b).map(move |a| (b - a).min(n - b + a) as f64))
        .collect();
    let input = DissimilarityView::new(&values, n).unwrap();
    let dense = DenseFlag::new(input.into(), input.diameter()).unwrap();
    let expected = reference::compute(input, &RipsOptions::default()).unwrap();
    let flag = AtomicBool::new(false);
    let limits = crate::persistence::ExecutionLimits::new(None, Some(&flag));
    let mut access = CancelAccess {
        inner: &dense,
        flag: &flag,
        cancel_at: usize::MAX,
        candidates: std::cell::Cell::new(0),
    };
    let mut stats = Stats::default();
    run_access::<true, true, ALL_SHORTCUTS>(
        &access,
        &mut stats,
        &mut WorkBudget::new(&limits).unwrap(),
    )
    .unwrap();
    assert!(stats.apparent_candidates > 0 && stats.virtual_additions > 0);
    let total = access.candidates.get();
    for cancel_at in 1..=total {
        access.cancel_at = cancel_at;
        access.candidates.set(0);
        let error = run_access::<true, true, ALL_SHORTCUTS>(
            &access,
            &mut Stats::default(),
            &mut WorkBudget::new(&limits).unwrap(),
        )
        .unwrap_err();
        assert_eq!(error, Error::Cancelled, "checkpoint {cancel_at}");
        assert!(flag.load(Ordering::Relaxed));
        flag.store(false, Ordering::Relaxed);
        let raw = run_access::<true, true, ALL_SHORTCUTS>(
            &dense,
            &mut Stats::default(),
            &mut WorkBudget::new(&limits).unwrap(),
        )
        .unwrap();
        assert_eq!(
            assemble_diagram(1, Coverage::Complete, raw).unwrap(),
            expected
        );
    }
}

struct Matrix(Vec<Vec<usize>>);
impl FilteredBoundary for Matrix {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn dimension(&self, _: usize) -> usize {
        unreachable!()
    }
    fn value(&self, _: usize) -> f64 {
        unreachable!()
    }
    fn write_boundary(&self, j: usize, out: &mut Vec<usize>) -> Result<()> {
        out.clear();
        out.extend_from_slice(&self.0[j]);
        Ok(())
    }
}

#[test]
fn reversed_transpose_pairs_and_unpaired_indices_map_back_to_boundary_reduction() {
    // Enumerate subsets independently of production combinatorial indexing.
    for n in 1_usize..=6 {
        let values: Vec<_> = (0..n * (n - 1) / 2)
            .map(|i| ((i * 13) % 5) as f64)
            .collect();
        let input = DissimilarityView::new(&values, n).unwrap();
        for cutoff in [0.0, 2.0, 4.0] {
            let mut cells: Vec<_> = (1_usize..(1 << n))
                .filter(|mask| mask.count_ones() <= 3)
                .map(|mask| {
                    let v: Vec<_> = (0..n).filter(|i| mask & (1 << i) != 0).collect();
                    let value = v
                        .iter()
                        .flat_map(|&a| v.iter().map(move |&b| input.get(a, b).unwrap()))
                        .fold(0.0_f64, f64::max);
                    (mask, v.len(), value)
                })
                .filter(|x| x.2 <= cutoff)
                .collect();
            // Within a fixed dimension, mask order equals combinatorial order.
            cells.sort_by(|a, b| a.2.total_cmp(&b.2).then(a.1.cmp(&b.1)).then(b.0.cmp(&a.0)));
            let size = cells.len();
            let d = Matrix(
                cells
                    .iter()
                    .map(|&(mask, dim, _)| {
                        cells
                            .iter()
                            .enumerate()
                            .filter_map(|(i, &(face, fdim, _))| {
                                (fdim + 1 == dim && face & mask == face).then_some(i)
                            })
                            .collect()
                    })
                    .collect(),
            );
            let mut c = Matrix(vec![vec![]; size]);
            for (j, rows) in d.0.iter().enumerate() {
                for &i in rows {
                    c.0[size - 1 - i].push(size - 1 - j);
                }
            }
            for rows in &mut c.0 {
                rows.sort_unstable();
            }
            let forward = crate::persistence::reference::reduction::reduce(&d).unwrap();
            let dual = crate::persistence::reference::reduction::reduce(&c).unwrap();
            let mut expected = forward.pairs;
            let mut actual: Vec<_> = dual
                .pairs
                .into_iter()
                .map(|(i, j)| (size - 1 - j, size - 1 - i))
                .collect();
            expected.sort_unstable();
            actual.sort_unstable();
            assert_eq!(actual, expected);
            let mut actual: Vec<_> = dual.unpaired.into_iter().map(|i| size - 1 - i).collect();
            actual.sort_unstable();
            assert_eq!(actual, forward.unpaired);
        }
    }
}

#[test]
fn shortcut_and_reconstruction_paths_are_exercised() {
    let n = 24;
    let values: Vec<_> = (0..n)
        .flat_map(|b| {
            (0..b).map(move |a| {
                let angle = std::f64::consts::PI * (b - a) as f64 / n as f64;
                2.0 * angle.sin()
            })
        })
        .collect();
    let input = DissimilarityView::new(&values, n).unwrap();
    let mut stats = Stats::default();
    let raw =
        run::<true, true, true, APPARENT_EMERGENT>(input, input.diameter(), &mut stats).unwrap();
    let expected = reference::compute(input, &RipsOptions::default()).unwrap();
    assert_eq!(
        assemble_diagram(1, Coverage::Complete, raw).unwrap(),
        expected
    );
    assert!(
        stats.shortcuts > 0 && stats.column_additions > 0 && stats.stored_entries > 0,
        "{stats:?}"
    );
}
