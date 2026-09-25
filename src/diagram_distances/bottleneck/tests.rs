use super::*;

// Independent enumeration of partial injective matches; no candidate search,
// threshold graph, diagonal copies, flow, or production cost helper is shared.
fn exhaustive(first: &[[f64; 2]], second: &[[f64; 2]]) -> f64 {
    fn visit(first: &[[f64; 2]], second: &[[f64; 2]], row: usize, used: u64, cost: f64) -> f64 {
        if row == first.len() {
            let mut result = cost;
            for (index, point) in second.iter().enumerate() {
                if used & (1 << index) == 0 {
                    result = result.max((point[1] - point[0]) / 2.0);
                }
            }
            return result;
        }
        let mut best = visit(
            first,
            second,
            row + 1,
            used,
            cost.max((first[row][1] - first[row][0]) / 2.0),
        );
        for (column, point) in second.iter().enumerate() {
            if used & (1 << column) == 0 {
                let edge = (first[row][0] - point[0])
                    .abs()
                    .max((first[row][1] - point[1]).abs());
                best = best.min(visit(
                    first,
                    second,
                    row + 1,
                    used | (1 << column),
                    cost.max(edge),
                ));
            }
        }
        best
    }
    visit(first, second, 0, 0, 0.0)
}

fn options() -> [Options; 7] {
    [
        Options::default(),
        Options {
            search: Search::Binary,
            ..Options::default()
        },
        Options {
            search: Search::Quickselect,
            ..Options::default()
        },
        Options {
            search: Search::Refinement,
            ..Options::default()
        },
        Options {
            search: Search::Refinement,
            reuse_matching: false,
            ..Options::default()
        },
        Options {
            search: Search::Refinement,
            reuse_scratch: false,
            ..Options::default()
        },
        Options {
            search: Search::Quickselect,
            clip_candidates: false,
            reuse_scratch: false,
            ..Options::default()
        },
    ]
}

#[test]
fn all_searches_match_independent_small_enumeration() {
    let mut state = 0xc0c7_c1e5_u64;
    for case in 0..240 {
        let mut next = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            state >> 32
        };
        let n = next() as usize % 6;
        let m = next() as usize % 6;
        let mut first = Vec::new();
        let mut second = Vec::new();
        for (count, points) in [(n, &mut first), (m, &mut second)] {
            for _ in 0..count {
                let birth = (next() % 17) as f64 / 8.0 - 1.0;
                let death = birth + (1 + next() % 9) as f64 / 8.0;
                points.push([birth, death]);
            }
        }
        let expected = exhaustive(&first, &second);
        for config in options() {
            let result =
                distance_with_options(&first, &second, config, &mut Diagnostics::default())
                    .unwrap();
            assert_eq!(
                result, expected,
                "case={case}, config={config:?}, first={first:?}, second={second:?}"
            );
        }
    }
}

#[test]
fn default_routes_preserve_c0_gates_and_exact_results() {
    let duplicates_a = vec![[0.0, 2.0]; 64];
    let duplicates_b = vec![[0.25, 2.25]; 64];
    let dense_a: Vec<_> = (0..64)
        .map(|i| [i as f64 / 64.0, i as f64 / 64.0 + 1.0])
        .collect();
    let dense_b: Vec<_> = dense_a
        .iter()
        .map(|p| [p[0] + 0.125, p[1] + 0.125])
        .collect();
    let separated: Vec<_> = dense_a.iter().map(|p| [p[0] + 8.0, p[1] + 8.0]).collect();
    let mut sparse_a: Vec<_> = (0..128).map(|i| [i as f64, i as f64 + 0.03125]).collect();
    sparse_a[127] = [127.0, 131.0];
    let sparse_b: Vec<_> = sparse_a
        .iter()
        .map(|p| [p[0] + 0.015625, p[1] + 0.015625])
        .collect();
    for (first, second, route, expected) in [
        (&duplicates_a, &duplicates_a, Route::Identity, 0.0),
        (&duplicates_a, &duplicates_b, Route::Multiplicity, 0.25),
        (&dense_a, &dense_b, Route::Refinement, 0.125),
        (&dense_a, &separated, Route::NoCross, 0.5),
        (&sparse_a, &sparse_b, Route::MandatorySparse, 0.015625),
    ] {
        let mut diagnostics = Diagnostics::default();
        let actual =
            distance_with_options(first, second, Options::default(), &mut diagnostics).unwrap();
        assert_eq!(diagnostics.route, route);
        assert_eq!(actual, expected);
        for config in options() {
            assert_eq!(
                distance_with_options(first, second, config, &mut Diagnostics::default()).unwrap(),
                expected,
                "route={route:?}, config={config:?}"
            );
        }
    }
}

#[test]
fn mandatory_and_grouped_circulations_match_partial_enumeration() {
    let catalog = [[0.0, 0.5], [0.25, 1.0], [0.0, 2.0], [1.0, 2.0]];
    for mask in 0..256 {
        let first = [
            catalog[mask % 4],
            catalog[(mask / 4) % 4],
            catalog[(mask / 16) % 4],
        ];
        let second = [catalog[(mask / 64) % 4], catalog[(mask / 16) % 4]];
        let expected = exhaustive(&first, &second);
        let first = Prepared::new(&first).unwrap();
        let second = Prepared::new(&second).unwrap();
        let pair = Pair::new(&first, &second, false).unwrap();
        for grouped in [false, true] {
            assert!(
                flow::within(&pair, expected, grouped, &mut Diagnostics::default(), 0).unwrap()
            );
            if expected > 0.0 {
                assert!(
                    !flow::within(
                        &pair,
                        expected.next_down(),
                        grouped,
                        &mut Diagnostics::default(),
                        0
                    )
                    .unwrap()
                );
            }
        }
    }
}

#[test]
fn extreme_finite_diagonal_cost_does_not_overflow() {
    let first = [[f64::MAX * 0.5, f64::MAX]];
    let expected = f64::MAX * 0.25;
    assert_eq!(distance(&first, &[]).unwrap(), expected);
    assert_eq!(distance(&[[f64::MIN, f64::MAX]], &[]).unwrap(), f64::MAX);
    assert_eq!(distance(&first, &first).unwrap(), 0.0);
    // Overflowing cross edges cannot contaminate the finite all-diagonal bound.
    let second = [[f64::MIN, f64::MIN * 0.5]];
    assert!(distance(&first, &second).unwrap().is_finite());
}

#[test]
fn geometric_reuse_and_rebuild_agree_on_ulp_thresholds() {
    let first: Vec<_> = (0..96)
        .map(|i| [1e12 + i as f64 / 8.0, 1e12 + i as f64 / 8.0 + 2.0])
        .collect();
    let second: Vec<_> = first
        .iter()
        .map(|p| [p[0].next_up(), p[1].next_up()])
        .collect();
    let expected = second[0][0] - first[0][0];
    for config in options() {
        assert_eq!(
            distance_with_options(&first, &second, config, &mut Diagnostics::default()).unwrap(),
            expected
        );
    }
}

#[test]
fn diagonal_cost_is_half_lifetime_at_float_boundaries() {
    let next = 1.0_f64.next_up();
    let expected = (next - 1.0) * 0.5;
    for points in [[[1.0, next]], [[-next, -1.0]], [[0.0, next - 1.0]]] {
        assert_eq!(distance(&points, &[]).unwrap(), expected);
        assert_eq!(distance(&[], &points).unwrap(), expected);
    }
    let negative = [[-1.0, (-1.0_f64).next_up()]];
    assert_eq!(distance(&negative, &[]).unwrap(), 2.0_f64.powi(-54));
    assert!(matches!(
        distance(&[[0.0, f64::from_bits(1)]], &[]),
        Err(Error::NumericalFailure { .. })
    ));
}

#[test]
fn diagnostic_switches_exercise_actual_reuse() {
    let first: Vec<_> = (0..64)
        .map(|i| [i as f64 / 64.0, 1.0 + i as f64 / 64.0])
        .collect();
    let second: Vec<_> = first.iter().map(|p| [p[0] + 0.125, p[1] + 0.125]).collect();
    let mut reused = Diagnostics::default();
    let mut rebuilt = Diagnostics::default();
    let mut no_clip = Diagnostics::default();
    let result = distance_with_options(&first, &second, Options::default(), &mut reused).unwrap();
    let other = distance_with_options(
        &first,
        &second,
        Options {
            reuse_matching: false,
            reuse_scratch: false,
            ..Options::default()
        },
        &mut rebuilt,
    )
    .unwrap();
    assert_eq!(result, other);
    assert!(reused.kd_nodes_visited > 0 && reused.scratch_reuses > 0 && reused.matching_reuses > 0);
    assert_eq!(rebuilt.matching_reuses, 0);
    assert_eq!(rebuilt.scratch_reuses, 0);
    distance_with_options(
        &first,
        &second,
        Options {
            clip_candidates: false,
            ..Options::default()
        },
        &mut no_clip,
    )
    .unwrap();
    assert!(no_clip.candidate_count > reused.candidate_count);
}

// A deliberately plain dense threshold oracle for medium dyadic inputs. Its
// recursive DFS is test-only and bounded below by at most 288 vertices.
fn dense_reference(first: &[[f64; 2]], second: &[[f64; 2]]) -> f64 {
    fn feasible(first: &[[f64; 2]], second: &[[f64; 2]], threshold: f64) -> bool {
        fn augment(
            left: usize,
            adjacency: &[Vec<usize>],
            seen: &mut [bool],
            owners: &mut [usize],
        ) -> bool {
            for &right in &adjacency[left] {
                if seen[right] {
                    continue;
                }
                seen[right] = true;
                if owners[right] == usize::MAX || augment(owners[right], adjacency, seen, owners) {
                    owners[right] = left;
                    return true;
                }
            }
            false
        }
        let n = first.len();
        let m = second.len();
        let mut adjacency = vec![Vec::new(); n + m];
        for (left, &a) in first.iter().enumerate() {
            for (right, &b) in second.iter().enumerate() {
                if (a[0] - b[0]).abs().max((a[1] - b[1]).abs()) <= threshold {
                    adjacency[left].push(right);
                }
            }
            if (a[1] - a[0]) / 2.0 <= threshold {
                adjacency[left].push(m + left);
            }
        }
        for (right, &b) in second.iter().enumerate() {
            if (b[1] - b[0]) / 2.0 <= threshold {
                adjacency[n + right].push(right);
            }
            adjacency[n + right].extend(m..m + n);
        }
        let mut owners = vec![usize::MAX; n + m];
        for left in 0..n + m {
            if !augment(left, &adjacency, &mut vec![false; n + m], &mut owners) {
                return false;
            }
        }
        true
    }
    let mut candidates = vec![0.0];
    for point in first.iter().chain(second) {
        candidates.push((point[1] - point[0]) / 2.0);
    }
    for a in first {
        for b in second {
            candidates.push((a[0] - b[0]).abs().max((a[1] - b[1]).abs()));
        }
    }
    candidates.sort_unstable_by(f64::total_cmp);
    candidates.dedup();
    let mut begin = 0;
    let mut end = candidates.len() - 1;
    while begin < end {
        let mid = (begin + end) / 2;
        if feasible(first, second, candidates[mid]) {
            end = mid;
        } else {
            begin = mid + 1;
        }
    }
    candidates[begin]
}

#[test]
fn medium_adaptive_routes_and_geometry_match_dense_oracle() {
    let mut state = 0x8ac3_55af_fab9_4315_u64;
    let mut routes = [false; 3];
    for case in 0..48 {
        let mut next = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            state >> 32
        };
        let mut first = Vec::new();
        let mut second = Vec::new();
        let sizes = if case % 3 == 0 { [128, 144] } else { [48, 80] };
        for (n, target) in [(sizes[0], &mut first), (sizes[1], &mut second)] {
            for index in 0..n {
                let (birth, persistence) = match case % 3 {
                    0 => (
                        index as f64 / 4.0,
                        if index == 0 {
                            4.0
                        } else {
                            (1 + next() % 3) as f64 / 64.0
                        },
                    ),
                    1 => ((next() % 64) as f64 / 16.0, (1 + next() % 48) as f64 / 16.0),
                    _ => ((next() % 3) as f64, (1 + next() % 2) as f64),
                };
                target.push([birth, birth + persistence]);
            }
        }
        let expected = dense_reference(&first, &second);
        let mut stats = Diagnostics::default();
        let actual =
            distance_with_options(&first, &second, Options::default(), &mut stats).unwrap();
        assert_eq!(actual, expected, "case={case}, route={:?}", stats.route);
        match stats.route {
            Route::MandatorySparse => routes[0] = true,
            Route::Refinement => routes[1] = true,
            Route::Multiplicity => routes[2] = true,
            _ => {}
        }
        for reuse_matching in [false, true] {
            let options = Options {
                search: Search::Refinement,
                reuse_matching,
                ..Options::default()
            };
            assert_eq!(
                distance_with_options(&first, &second, options, &mut Diagnostics::default())
                    .unwrap(),
                expected,
                "case={case}, reuse={reuse_matching}"
            );
        }
    }
    assert!(routes.into_iter().all(|exercised| exercised));
}
