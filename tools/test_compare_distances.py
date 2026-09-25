"""Distance transport, independent expectations and failure-retention regressions."""

import contextlib
import io
import json
import math
from pathlib import Path
import runpy
import struct
import sys
import tempfile
import types
import unittest
from unittest import mock
import warnings

from distance_common import (MAGIC, PHASE_HOOKS, ROOT, agrees, instrument_phase, invoke,
                             phase_timings, profile_worker, read_fixture, sha256, tiny_oracle,
                             tolerance, unpack_number, write_fixture)


class DistanceProtocolTests(unittest.TestCase):
    def test_gudhi_iteration_limit_cannot_become_a_completed_reference(self):
        import distance_common

        def unconverged(*args, **kwargs):
            warnings.warn('numItermax reached before optimality. Try to increase numItermax.', UserWarning)
            return 123.0

        numpy = types.ModuleType('numpy')
        numpy.float64 = float
        numpy.asarray = lambda points, dtype: types.SimpleNamespace(reshape=lambda shape: points)
        gudhi = types.ModuleType('gudhi')
        wasserstein = types.ModuleType('gudhi.wasserstein')
        wasserstein.wasserstein_distance = unconverged
        with mock.patch.object(sys, 'path', sys.path.copy()):
            worker = runpy.run_path(str(distance_common.WORKERS / 'gudhi_worker.py'))
        output = io.StringIO()
        with (
            mock.patch.dict(sys.modules, {'numpy': numpy, 'gudhi': gudhi,
                                         'gudhi.wasserstein': wasserstein}),
            mock.patch('importlib.metadata.version', side_effect=distance_common.PINS['gudhi_python'].__getitem__),
            mock.patch.object(sys, 'argv', ['worker', 'unused.bin', 'w2', 'baseline']),
            mock.patch.dict(worker['main'].__globals__, {'read_fixture': lambda _: ([], [])}),
            mock.patch.dict('os.environ'),
            contextlib.redirect_stdout(output),
        ):
            worker['main']()
        result = json.loads(output.getvalue())
        self.assertEqual(result['status'], 'unavailable')
        self.assertIn('did not converge', result['error'])
        self.assertEqual(result['settings']['numItermax'], 2000000)
        self.assertNotIn('value', result)

    def test_profile_generation_preserves_bodies_and_fails_on_marker_drift(self):
        labels = [label for hooks in PHASE_HOOKS.values() for _, label in hooks]
        self.assertEqual(len(labels), len(set(labels)))
        with tempfile.TemporaryDirectory() as directory:
            worker, metadata = profile_worker(directory)
            self.assertIn('#![forbid(unsafe_code)]', worker.read_text())
            self.assertFalse(metadata['algorithm_selection_eligible'])
            for name, expected in metadata['original_source_sha256'].items():
                self.assertEqual(sha256(ROOT / name), expected)
                if not name.startswith('src/'):
                    continue
                original = (ROOT / name).read_text()
                generated = (Path(directory) / 'profile-source' / name).read_text()
                restored = '\n'.join(line for line in generated.split('\n')
                                     if 'let _distance_phase = crate::DistancePhase::new(' not in line)
                self.assertEqual(restored, original)
            self.assertEqual(sum((Path(directory) / 'profile-source' / name).read_text().count(
                'let _distance_phase = crate::DistancePhase::new(')
                for name in metadata['generated_source_sha256']), len(labels))
        for source in ('fn renamed() {}', 'fn target() {} fn target() {}'):
            with self.assertRaises(ValueError):
                instrument_phase(source, 'fn target(', 'bottleneck.total')

    def test_phase_records_preserve_nanoseconds_and_reject_ambiguity(self):
        record = 'COCYCLE_DISTANCE_PHASE\tbottleneck.total\t1\t9007199254740993'
        self.assertEqual(phase_timings('unrelated stderr\n' + record), {
            'bottleneck.total': {'calls': 1, 'elapsed_ns': 9007199254740993}})
        self.assertEqual(phase_timings(''), {})
        for malformed in (record + '\n' + record, record.replace('total', 'unknown'),
                          record + '\textra', record.replace('\t1\t', '\t0\t'),
                          record.replace('\t1\t', '\t-1\t'),
                          record.rsplit('\t', 1)[0] + '\t-1',
                          record.rsplit('\t', 1)[0] + '\t1.0'):
            with self.assertRaises(ValueError):
                phase_timings(malformed)

    def test_round_trip_preserves_multiplicity_and_essential_points(self):
        first = [(-0.0, 1.0), (0.0, 1.0), (2.0, math.inf)]
        second = []
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'fixture.bin'
            write_fixture(path, first, second)
            actual, other = read_fixture(path)
            self.assertEqual((actual, other), (first, second))
            self.assertEqual(math.copysign(1.0, actual[0][0]), -1.0)
            path.write_bytes(path.read_bytes() + b'\0')
            with self.assertRaisesRegex(ValueError, 'length'):
                read_fixture(path)

    def test_huge_claimed_length_is_rejected_before_allocation(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'fixture.bin'
            path.write_bytes(MAGIC + struct.pack('<QQ', 2 ** 64 - 1, 0))
            with self.assertRaises(ValueError):
                read_fixture(path)

    def test_hand_derived_diagonal_and_pair_costs(self):
        for metric, diagonal, pair in [('bottleneck', 1.0, 1.0), ('w1', 1.0, 1.0),
                                       ('w2', math.sqrt(2), math.sqrt(2))]:
            self.assertEqual(tiny_oracle([(0.0, 2.0)], [], metric), diagonal)
            self.assertEqual(tiny_oracle([(0.0, 2.0)], [(1.0, 3.0)], metric), pair)

    def test_partial_matching_and_duplicate_multiplicity(self):
        for metric, expected in [('bottleneck', 1.0), ('w1', 2.0), ('w2', 2.0)]:
            self.assertEqual(tiny_oracle([(0.0, 2.0)] * 3, [(0.0, 2.0)], metric), expected)
        self.assertEqual(tiny_oracle([(0.0, 0.0)], [], 'w1'), 0.0)

    def test_essential_counts_and_order(self):
        self.assertEqual(tiny_oracle([(0.0, math.inf)], [], 'w1'), math.inf)
        first = [(3.0, math.inf), (0.0, math.inf), (0.0, 2.0)]
        second = [(1.0, math.inf), (5.0, math.inf)]
        self.assertEqual(tiny_oracle(first, second, 'w1'), 4.0)
        self.assertEqual(tiny_oracle(first, second, 'w2'), math.sqrt(7.0))

    def test_adjacent_float_stress_cannot_be_hidden_by_tolerance(self):
        case = {'first': [(1.0, math.nextafter(1.0, math.inf))], 'second': [], 'exact_grid': True}
        for metric in ('bottleneck', 'w1', 'w2'):
            expected = tiny_oracle(case['first'], case['second'], metric)
            self.assertFalse(agrees(expected * 2, expected, case, metric))
            self.assertLess(tolerance(case, metric, expected), expected)

    def test_invalid_input_and_scalar_are_rejected(self):
        for points in ([(math.nan, 1.0)], [(2.0, 1.0)], [(-math.inf, 1.0)]):
            with self.assertRaises(ValueError):
                tiny_oracle(points, [], 'w1')
        for value in ({'value_kind': 'finite', 'value': math.nan},
                      {'value_kind': 'finite', 'value': True},
                      {'value_kind': 'infinite', 'value': 1}):
            with self.assertRaises(ValueError):
                unpack_number(value)

    def test_process_error_protocol_error_and_timeout_remain_distinct(self):
        cases = [(['-c', 'import sys; print("failure"); sys.exit(3)'], 'process_error', 5),
                 (['-c', 'print("not JSON")'], 'protocol_error', 5),
                 (['-c', 'import time; time.sleep(10)'], 'timeout', 0.02)]
        for arguments, expected, timeout in cases:
            sample = invoke([sys.executable, *arguments], 'unused.bin', 'w1', timeout=timeout)
            self.assertEqual(sample['status'], expected)


if __name__ == '__main__':
    unittest.main()
