"""Regression tests for native distance experiment design and selection guards."""

from collections import Counter
import unittest

from benchmark_distances import DEVELOPMENT_SEED, FAMILIES, HOLDOUT_SEED, exercised_sparse_layout, fixtures, schedule, select, summarize, variants


class DistanceBenchmarkTests(unittest.TestCase):
    def test_rounds_are_seeded_and_position_balanced(self):
        workers = ['a', 'b', 'c', 'd']
        entries = schedule(workers, 12, 42)
        self.assertEqual(entries, schedule(workers, 12, 42))
        self.assertNotEqual(entries, schedule(workers, 12, 7))
        self.assertEqual(sum(entry['warmup'] for entry in entries), 4)
        for worker in workers:
            counts = Counter(entry['position'] for entry in entries
                             if entry['worker'] == worker and not entry['warmup'])
            self.assertEqual(counts, {0: 3, 1: 3, 2: 3, 3: 3})

    def test_tuning_and_holdout_use_disjoint_reproducible_seeds(self):
        cases = list(fixtures(FAMILIES, [8, 32, 128, 512], False))
        self.assertEqual(cases, list(fixtures(FAMILIES, [8, 32, 128, 512], False)))
        tuning = {case['seed'] for case in cases if case['split'] == 'tuning'}
        holdout = {case['seed'] for case in cases if case['split'] == 'holdout'}
        self.assertFalse(tuning & holdout)
        self.assertEqual(tuning, {20260922})
        self.assertEqual(holdout, {20260923})
        pairs = {}
        for case in cases:
            pairs.setdefault((case['family'], case['size']), {})[case['split']] = (
                case['first'], case['second'])
        for key, pair in pairs.items():
            with self.subTest(family=key[0], size=key[1]):
                self.assertNotEqual(pair['tuning'], pair['holdout'])

    def test_incomplete_failed_or_mismatched_group_has_no_statistics(self):
        sample = {'status': 'completed', 'warmup': False, 'elapsed_ms': 2,
                  'peak_rss_kib': 100, 'hwm_before_kib': 80}
        self.assertIsNone(summarize([sample], 2))
        self.assertIsNone(summarize([sample, {**sample, 'status': 'timeout'}], 2))
        self.assertIsNone(summarize([sample, {**sample, 'comparison_error': 'mismatch'}], 2))
        self.assertEqual(summarize([sample, sample], 2)['max_hwm_growth_kib'], 20)

    @staticmethod
    def rows(ratios):
        rows = []
        for index, (time_ratio, memory_ratio) in enumerate(ratios):
            for worker, time, memory in [('cocycle:baseline', 100, 100),
                                         ('cocycle:arena', 100 * time_ratio, 100 * memory_ratio)]:
                rows.append({'case': f'case-{index}', 'family': f'family-{index}', 'split': 'holdout',
                             'worker': worker, 'median_ms': time, 'median_peak_rss_kib': memory})
        return rows

    def test_mixed_selection_accepts_benefit_but_not_hidden_family_regression(self):
        self.assertEqual(select(self.rows([(0.8, 1.02)]), 'cocycle:arena')['status'], 'eligible_pending_independent_repeat')
        self.assertEqual(select(self.rows([(0.98, 0.98)]), 'cocycle:arena')['status'], 'retain_baseline')
        self.assertEqual(select(self.rows([(0.5, 0.5), (1.11, 1.0)]), 'cocycle:arena')['status'], 'retain_baseline')

    def test_selection_ignores_tuning_and_rejects_missing_pairs(self):
        rows = self.rows([(0.8, 1.0)])
        rows.extend({**row, 'case': 'tuning', 'split': 'tuning', 'median_ms': 10 ** 10} for row in list(rows))
        self.assertEqual(select(rows, 'cocycle:arena')['status'], 'eligible_pending_independent_repeat')
        self.assertEqual(select(rows[:1], 'cocycle:arena')['status'], 'incomplete')

    def test_local_ablation_and_adaptive_candidate_are_separate(self):
        pairs = variants('bottleneck', ['search'])
        for backend in ('cocycle', 'topp'):
            self.assertIn((backend, 'quickselect'), pairs)
            self.assertIn((backend, 'binary'), pairs)
        wasserstein = variants('w1', ['arena'])
        self.assertIn(('cocycle', 'adaptive_arena'), wasserstein)
        self.assertNotIn(('topp', 'adaptive_arena'), wasserstein)

    def test_empty_sparse_call_does_not_exercise_residual_layout(self):
        # Separated diagrams enter sparse::solve, then return before Network::new.
        empty_call = {'sparse_solves': 1, 'positive_edges': 0,
                      'peak_residual_storage_bytes': 0, 'direct_cost_fallbacks': 0}
        self.assertFalse(exercised_sparse_layout(empty_call))
        self.assertFalse(exercised_sparse_layout({**empty_call, 'positive_edges': 1}))
        active = {**empty_call, 'positive_edges': 1, 'peak_residual_storage_bytes': 128}
        self.assertTrue(exercised_sparse_layout(active))
        self.assertFalse(exercised_sparse_layout({**active, 'direct_cost_fallbacks': 1}))


if __name__ == '__main__':
    unittest.main()
