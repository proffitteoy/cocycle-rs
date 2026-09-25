use super::*;

// This independent oracle enumerates partial point-to-point bijections and
// charges remaining points directly to the diagonal, without a saving graph.
fn exhaustive(first: &[[f64; 2]], second: &[[f64; 2]], metric: Metric) -> f64 {
    fn diagonal(point: [f64; 2], metric: Metric) -> f64 {
        let lifetime = point[1] - point[0];
        if metric == Metric::W1 {
            lifetime / 2.0
        } else {
            lifetime * lifetime / 2.0
        }
    }
    fn recurse(
        first: &[[f64; 2]],
        second: &[[f64; 2]],
        metric: Metric,
        row: usize,
        mask: usize,
        total: f64,
    ) -> f64 {
        if row == first.len() {
            return total
                + second
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) == 0)
                    .map(|(_, &point)| diagonal(point, metric))
                    .sum::<f64>();
        }
        let mut best = recurse(
            first,
            second,
            metric,
            row + 1,
            mask,
            total + diagonal(first[row], metric),
        );
        for (column, point) in second.iter().enumerate() {
            if mask & (1 << column) != 0 {
                continue;
            }
            let x = (first[row][0] - point[0]).abs();
            let y = (first[row][1] - point[1]).abs();
            let cost = if metric == Metric::W1 {
                x.max(y)
            } else {
                x * x + y * y
            };
            best = best.min(recurse(
                first,
                second,
                metric,
                row + 1,
                mask | (1 << column),
                total + cost,
            ));
        }
        best
    }
    let cost = recurse(first, second, metric, 0, 0, 0.0);
    if metric == Metric::W2 {
        cost.sqrt()
    } else {
        cost
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2e-12 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}

fn random(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *seed >> 32
}

#[test]
fn all_matchers_agree_with_independent_small_oracle() {
    let mut seed = 5701;
    for case in 0..160 {
        let first: Vec<_> = (0..case % 6)
            .map(|_| {
                let b = (random(&mut seed) % 25) as f64 / 4.0 - 3.0;
                [b, b + (1 + random(&mut seed) % 16) as f64 / 4.0]
            })
            .collect();
        let second: Vec<_> = (0..(case * 7 + 3) % 6)
            .map(|_| {
                let b = (random(&mut seed) % 25) as f64 / 4.0 - 3.0;
                [b, b + (1 + random(&mut seed) % 16) as f64 / 4.0]
            })
            .collect();
        for metric in [Metric::W1, Metric::W2] {
            let expected = exhaustive(&first, &second, metric);
            assert_close(distance(&first, &second, metric).unwrap(), expected);
            for sparse in [SparseLayout::Vectors, SparseLayout::Arena] {
                let actual = distance_with_options(
                    &first,
                    &second,
                    metric,
                    Options {
                        sparse,
                        force_sparse: true,
                    },
                    &mut Stats::default(),
                )
                .unwrap();
                assert_close(actual, expected);
                assert_close(
                    distance_with_options(
                        &second,
                        &first,
                        metric,
                        Options {
                            sparse,
                            force_sparse: true,
                        },
                        &mut Stats::default(),
                    )
                    .unwrap(),
                    expected,
                );
            }
            let a = prepare(&first, 1.0).unwrap();
            let b = prepare(&second, 1.0).unwrap();
            let graph = generate(&a, &b, metric, &mut Stats::default()).unwrap();
            for row_reduction in [false, true] {
                let matching = dense_sap(&graph, row_reduction, &mut Stats::default()).unwrap();
                let flows: Vec<_> = matching
                    .into_iter()
                    .enumerate()
                    .filter_map(|(r, c)| c.map(|c| (r, c, 1)))
                    .collect();
                assert_close(
                    from_flows(&a, &b, &vec![1; a.len()], &vec![1; b.len()], &flows, metric)
                        .unwrap(),
                    expected,
                );
            }
        }
    }
}

#[test]
fn duplicates_use_capacities_without_losing_multiplicity() {
    let first: Vec<_> = (0..60)
        .map(|i| if i < 36 { [0.0, 4.0] } else { [10.0, 12.0] })
        .collect();
    let second: Vec<_> = (0..70)
        .map(|i| if i < 21 { [1.0, 3.0] } else { [10.25, 12.25] })
        .collect();
    for metric in [Metric::W1, Metric::W2] {
        let mut stats = Stats::default();
        let compressed =
            distance_with_options(&first, &second, metric, Options::default(), &mut stats).unwrap();
        assert_eq!(stats.duplicate_groups, 4);
        assert_eq!(stats.candidate_pairs, 4);
        let uncompressed = distance_with_options(
            &first,
            &second,
            metric,
            Options {
                force_sparse: true,
                ..Options::default()
            },
            &mut Stats::default(),
        )
        .unwrap();
        assert_close(compressed, uncompressed);
    }
}

#[test]
fn sweep_and_disconnected_tiny_components_keep_the_full_cost() {
    let first: Vec<_> = (0..80)
        .map(|i| [i as f64 * 10.0, i as f64 * 10.0 + 2.0])
        .collect();
    let second: Vec<_> = first.iter().map(|p| [p[0] + 0.25, p[1] + 0.25]).collect();
    for metric in [Metric::W1, Metric::W2] {
        let mut stats = Stats::default();
        let actual =
            distance_with_options(&first, &second, metric, Options::default(), &mut stats).unwrap();
        assert_close(
            actual,
            if metric == Metric::W1 {
                20.0
            } else {
                10.0_f64.sqrt()
            },
        );
        assert_eq!(stats.components, 80);
        assert_eq!(stats.tiny_components, 80);
        assert!(stats.candidate_pairs < 160);
    }
}

#[test]
fn arena_reuses_search_scratch_and_retains_vector_answer() {
    let first = [[0.0, 5.0], [0.5, 5.5], [1.0, 4.0], [2.0, 6.0]];
    let second = [[0.2, 4.8], [0.6, 5.6], [0.9, 4.5], [2.2, 6.0]];
    for metric in [Metric::W1, Metric::W2] {
        let mut vectors = Stats::default();
        let mut arena = Stats::default();
        let a = distance_with_options(
            &first,
            &second,
            metric,
            Options {
                sparse: SparseLayout::Vectors,
                force_sparse: true,
            },
            &mut vectors,
        )
        .unwrap();
        let b = distance_with_options(
            &first,
            &second,
            metric,
            Options {
                sparse: SparseLayout::Arena,
                force_sparse: true,
            },
            &mut arena,
        )
        .unwrap();
        assert_eq!(a, b);
        assert_eq!(vectors.augmentations, arena.augmentations);
        assert_eq!(vectors.scratch_reuses, 0);
        assert!(arena.scratch_reuses >= 3);
    }
}

#[test]
fn tiny_multivertex_components_are_enumerated() {
    let mut first = Vec::new();
    let mut second = Vec::new();
    for block in 0..20 {
        let base = block as f64 * 30.0;
        first.extend([[base, base + 5.0], [base + 0.5, base + 4.0]]);
        second.extend([[base + 0.1, base + 5.1], [base + 0.6, base + 4.1]]);
    }
    let mut stats = Stats::default();
    let actual =
        distance_with_options(&first, &second, Metric::W2, Options::default(), &mut stats).unwrap();
    assert_close(actual, 0.8_f64.sqrt());
    assert_eq!(stats.tiny_components, 20);
}

#[test]
fn original_cost_reconstruction_avoids_w2_saving_cancellation() {
    let epsilon = 2.0_f64.powi(-40);
    let first = [[0.0, 2.0]];
    let second = [[epsilon, 2.0 + epsilon]];
    let actual = distance(&first, &second, Metric::W2).unwrap();
    assert_eq!(actual, epsilon * 2.0_f64.sqrt());
}

#[test]
fn adjacent_floats_have_the_mathematical_diagonal_cost() {
    let death = f64::from_bits(1.0_f64.to_bits() + 1);
    assert_eq!(
        distance(&[[1.0, death]], &[], Metric::W1).unwrap(),
        f64::EPSILON / 2.0
    );
    assert!(matches!(
        distance(&[[0.0, f64::from_bits(1)]], &[], Metric::W1),
        Err(Error::NumericalFailure { .. })
    ));
}

#[test]
fn direct_cost_guard_preserves_tiny_w1_permutations_and_duplicates() {
    let epsilon = 2.0_f64.powi(-60);
    let first = [[0.0, 2.0], [epsilon, 2.0]];
    let second = [first[1], first[0]];
    for metric in [Metric::W1, Metric::W2] {
        for copies in [1, 20] {
            let a = first.repeat(copies);
            let b = second.repeat(copies);
            for options in [
                Options::default(),
                Options {
                    sparse: SparseLayout::Vectors,
                    force_sparse: true,
                },
                Options {
                    sparse: SparseLayout::Arena,
                    force_sparse: true,
                },
            ] {
                let mut stats = Stats::default();
                assert_eq!(
                    distance_with_options(&a, &b, metric, options, &mut stats).unwrap(),
                    0.0
                );
                assert_eq!(stats.direct_cost_fallbacks, 1);
            }
        }
    }
}

#[test]
fn near_identical_multisets_match_independent_original_cost_oracle() {
    let epsilon = 2.0_f64.powi(-40);
    let mut seed = 329;
    for _ in 0..60 {
        let a: Vec<_> = (0..4)
            .map(|_| {
                [
                    (random(&mut seed) % 16) as f64 * epsilon,
                    2.0 + (random(&mut seed) % 16) as f64 * epsilon,
                ]
            })
            .collect();
        let b: Vec<_> = (0..4)
            .map(|_| {
                [
                    (random(&mut seed) % 16) as f64 * epsilon,
                    2.0 + (random(&mut seed) % 16) as f64 * epsilon,
                ]
            })
            .collect();
        for metric in [Metric::W1, Metric::W2] {
            let expected = exhaustive(&a, &b, metric);
            let actual = distance(&a, &b, metric).unwrap();
            assert!(
                (actual - expected).abs() <= 16.0 * f64::EPSILON * expected,
                "{metric:?}: {actual} != {expected}; {a:?}, {b:?}"
            );
        }
    }
}

#[test]
fn guarded_sweep_preserves_ulp_sized_blocks_at_large_offsets() {
    let mut seed = 45;
    for base in [1.0_f64, 1e16] {
        let ulp = base.next_up() - base;
        let mut a = Vec::new();
        let mut b = Vec::new();
        let mut expected = [0.0, 0.0];
        for block in 0..12 {
            let origin = base + block as f64 * 128.0 * ulp;
            let first: Vec<_> = (0..3)
                .map(|_| {
                    let birth = origin + (random(&mut seed) % 5) as f64 * ulp;
                    [birth, birth + (2 + random(&mut seed) % 7) as f64 * ulp]
                })
                .collect();
            let second: Vec<_> = (0..3)
                .map(|_| {
                    let birth = origin + (random(&mut seed) % 5) as f64 * ulp;
                    [birth, birth + (2 + random(&mut seed) % 7) as f64 * ulp]
                })
                .collect();
            expected[0] += exhaustive(&first, &second, Metric::W1);
            expected[1] += exhaustive(&first, &second, Metric::W2).powi(2);
            a.extend(first);
            b.extend(second);
        }
        for (metric, expected) in [(Metric::W1, expected[0]), (Metric::W2, expected[1].sqrt())] {
            let actual = distance(&a, &b, metric).unwrap();
            assert!(
                (actual - expected).abs() <= 16.0 * f64::EPSILON * expected,
                "{metric:?}: {actual} != {expected} at base {base}"
            );
        }
    }
}

#[test]
fn scalar_and_sweep_routes_agree_with_forced_residual_solvers() {
    let mut seed = 1391;
    for sample in 0..24 {
        let spread = if sample % 2 == 0 { 100.0 } else { 1.0 };
        let first: Vec<_> = (0..40)
            .map(|_| {
                let b = (random(&mut seed) % 10000) as f64 / 10000.0 * spread;
                [b, b + 1.0 + (random(&mut seed) % 1000) as f64 / 1000.0]
            })
            .collect();
        let second: Vec<_> = (0..42)
            .map(|_| {
                let b = (random(&mut seed) % 10000) as f64 / 10000.0 * spread;
                [b, b + 1.0 + (random(&mut seed) % 1000) as f64 / 1000.0]
            })
            .collect();
        for metric in [Metric::W1, Metric::W2] {
            let expected = distance(&first, &second, metric).unwrap();
            for sparse in [SparseLayout::Vectors, SparseLayout::Arena] {
                assert_close(
                    distance_with_options(
                        &first,
                        &second,
                        metric,
                        Options {
                            sparse,
                            force_sparse: true,
                        },
                        &mut Stats::default(),
                    )
                    .unwrap(),
                    expected,
                );
            }
        }
    }
}

#[test]
fn power_of_two_scaling_preserves_extreme_representable_answers() {
    let maximum = f64::MAX;
    assert_eq!(
        distance(&[[-maximum, maximum]], &[], Metric::W1).unwrap(),
        maximum
    );
    assert!(matches!(
        distance(&[[-maximum, maximum]], &[], Metric::W2),
        Err(Error::NumericalFailure { .. })
    ));
    for lifetime in [1e200, 1e-200, f64::MIN_POSITIVE] {
        let actual = distance(&[[0.0, lifetime]], &[], Metric::W2).unwrap();
        let expected = lifetime / 2.0_f64.sqrt();
        assert!(actual > 0.0);
        assert!((actual / expected - 1.0).abs() <= 4.0 * f64::EPSILON);
    }
    assert!(matches!(
        distance(&[[0.0, 1e-300], [0.0, 1e300]], &[], Metric::W2),
        Err(Error::NumericalFailure { .. })
    ));
}
