"""Measure pinned native distance strategies in serial, fresh Linux processes."""

import argparse
from datetime import datetime, timezone
import math
import os
import platform
import random
import statistics
import subprocess
import sys

from distance_common import (METRICS, PROTOCOL, WORKERS, add_build_arguments, agrees,
                             build_from_args, identity, invoke, number, phase_timings, save,
                             sha256, tiny_oracle, unpack_number, write_fixture)

FAMILIES = ('uniform', 'clustered', 'near_diagonal', 'duplicates', 'imbalanced',
            'separated', 'threshold_shell', 'dense', 'sparse')
GROUPS = ('baseline', 'search', 'scratch', 'matching', 'clipping', 'arena')
DEVELOPMENT_SEED = 20260922
HOLDOUT_SEED = 20260923


def schedule(backends, samples, seed):
    """One discarded warmup, then shuffled cyclic blocks with balanced positions."""
    if samples < 1 or not backends:
        raise ValueError('positive samples and at least one backend required')
    rng = random.Random(seed)
    order = list(backends)
    rng.shuffle(order)
    rounds = [list(order)]
    while len(rounds) <= samples:
        rng.shuffle(order)
        rotations = list(range(len(order)))
        rng.shuffle(rotations)
        rounds.extend(order[index:] + order[:index] for index in rotations)
    return [{'worker': worker, 'round': round_index, 'warmup': round_index == 0, 'position': position}
            for round_index, order in enumerate(rounds[:samples + 1])
            for position, worker in enumerate(order)]


def fixtures(families, sizes, quick):
    for split, seeds in (('tuning', (DEVELOPMENT_SEED,)), ('holdout', (HOLDOUT_SEED,))):
        for seed in seeds:
            for family in families:
                for size in sizes:
                    rng = random.Random(seed * 1_000_003 + size)
                    # Regular families must still produce independent held-out
                    # instances; changing only the seed label is insufficient.
                    duplicate_templates = []
                    if family == 'duplicates':
                        duplicate_templates = [(group + rng.randrange(5) / 16,
                                                1.5 + rng.randrange(9) / 8,
                                                rng.randrange(1, 9) / 64)
                                               for group in range(8)]
                    threshold_offset = rng.randrange(1, 9) / 4096 if family == 'threshold_shell' else 0
                    diagrams = []
                    for side in range(2):
                        points = []
                        count = max(1, size // 8) if family == 'imbalanced' and side else size
                        for index in range(count):
                            birth = rng.randrange(-4096, 4096) / 256
                            lifetime = rng.randrange(1, 1025) / 256
                            if family == 'clustered':
                                birth = (index % 4) * 16 + rng.randrange(16) / 256
                            elif family == 'near_diagonal':
                                lifetime = rng.randrange(1, 9) / 4096
                            elif family == 'duplicates':
                                center, lifetime, offset = duplicate_templates[index % 8]
                                birth = center + side * offset
                            elif family == 'separated':
                                birth += side * 1024
                            elif family == 'dense':
                                birth /= 16
                                lifetime += 32
                            elif family == 'sparse':
                                birth = index * 8 + side / 8 + rng.randrange(-4, 5) / 64
                                lifetime = rng.randrange(4, 9) / 8
                            elif family == 'threshold_shell':
                                birth = index / (2 ** 20) + rng.randrange(16) / (2 ** 26)
                                lifetime = 2.0 + side * threshold_offset - birth
                            points.append((birth, birth + lifetime))
                        diagrams.append(points)
                    yield {'name': f'{split}-{family}-{size}-{seed}', 'family': family,
                           'size': size, 'seed': seed, 'split': split,
                           'first': diagrams[0], 'second': diagrams[1], 'exact_grid': True}


def variants(metric, groups=GROUPS):
    """Each local contrast freezes the rest; adaptive baselines stay separate."""
    result = [('cocycle', 'baseline'), ('topp', 'baseline')]
    if metric == 'bottleneck':
        if 'search' in groups:
            result.extend((backend, name) for backend in ('cocycle', 'topp')
                          for name in ('quickselect', 'binary'))
        if 'scratch' in groups:
            result.extend(('cocycle', name) for name in ('refinement', 'refinement_no_scratch_reuse'))
        if 'matching' in groups:
            result.extend(('cocycle', name) for name in ('refinement', 'refinement_no_matching_reuse'))
        if 'clipping' in groups:
            result.extend((backend, name) for backend in ('cocycle', 'topp')
                          for name in ('quickselect', 'quickselect_no_clip'))
    elif 'arena' in groups:
        result.extend((backend, name) for backend in ('cocycle', 'topp') for name in ('vectors', 'arena'))
        result.append(('cocycle', 'adaptive_arena'))
    return list(dict.fromkeys(result))


def exercised_sparse_layout(stats):
    """An empty sparse call returns before constructing the residual layout."""
    return (stats.get('sparse_solves', 0) > 0 and stats.get('positive_edges', 0) > 0
            and stats.get('peak_residual_storage_bytes', 0) > 0
            and stats.get('direct_cost_fallbacks', 0) == 0)


def summarize(samples, expected_count):
    measured = [sample for sample in samples if not sample['warmup']]
    if len(measured) != expected_count or any(sample['status'] != 'completed' or
            sample.get('comparison_error') for sample in samples):
        return None
    return {'samples': len(measured),
            'median_ms': statistics.median(sample['elapsed_ms'] for sample in measured),
            'min_ms': min(sample['elapsed_ms'] for sample in measured),
            'max_ms': max(sample['elapsed_ms'] for sample in measured),
            'median_peak_rss_kib': statistics.median(sample['peak_rss_kib'] for sample in measured),
            'max_peak_rss_kib': max(sample['peak_rss_kib'] for sample in measured),
            'max_hwm_growth_kib': max(sample['peak_rss_kib'] - sample['hwm_before_kib'] for sample in measured)}


def select(rows, candidate, baseline='cocycle:baseline'):
    """Predeclared mixed objective on equally weighted held-out families only."""
    pairs = {}
    for row in rows:
        if row['split'] == 'holdout' and row['worker'] in (candidate, baseline):
            pairs.setdefault(row['case'], {})[row['worker']] = row
    ratios = {}
    for pair in pairs.values():
        if set(pair) != {candidate, baseline}:
            return {'status': 'incomplete', 'candidate': candidate}
        reference, current = pair[baseline], pair[candidate]
        if reference['median_ms'] <= 0 or reference['median_peak_rss_kib'] <= 0:
            return {'status': 'unmeasurable', 'candidate': candidate}
        ratios.setdefault(reference['family'], []).append((
            current['median_ms'] / reference['median_ms'],
            current['median_peak_rss_kib'] / reference['median_peak_rss_kib']))
    if not ratios:
        return {'status': 'incomplete', 'candidate': candidate}
    family_ratios = {name: [statistics.geometric_mean(pair[index] for pair in values)
                           for index in (0, 1)] for name, values in ratios.items()}
    time_ratio, memory_ratio = [statistics.geometric_mean(pair[index] for pair in family_ratios.values())
                                for index in (0, 1)]
    mixed_score = math.sqrt(time_ratio * memory_ratio)
    eligible = mixed_score <= 0.95 and all(max(pair) <= 1.10 for pair in family_ratios.values())
    return {'status': 'eligible_pending_independent_repeat' if eligible else 'retain_baseline', 'candidate': candidate,
            'baseline': baseline, 'time_ratio': time_ratio, 'memory_ratio': memory_ratio,
            'mixed_score': mixed_score, 'family_ratios': family_ratios,
            'independent_repeat_required': True,
            'interpretation': 'candidate/baseline; smaller is better; process RSS is not algorithm allocation'}


def run(args):
    if platform.system() != 'Linux':
        raise ValueError('Linux required for per-process RSS/HWM, address-space caps and affinity')
    if args.samples < 1 or args.address_space_mib < 1:
        raise ValueError('samples and address-space limit must be positive')
    if not args.quick and args.samples < 12:
        raise ValueError('formal comparisons require at least 12 measured processes per cell')
    initial = identity()
    if initial['dirty'] and not args.exploratory:
        raise ValueError('commit measured sources first, or explicitly use --exploratory')
    started = datetime.now(timezone.utc).isoformat()
    commands, build = build_from_args(args)
    gudhi = [args.gudhi_python, str(WORKERS / 'gudhi_worker.py')]
    fixtures_dir = args.output / 'fixtures'
    fixtures_dir.mkdir()
    environment = {**initial, 'protocol_id': PROTOCOL,
                   'evidence_class': ('kernel_diagnostics' if args.profile_rust else
                                      'resource_snapshot' if args.quick or args.exploratory else
                                      'comparative_performance'),
                   'started_utc': started, 'build': build, 'platform': platform.platform(),
                   'processor': platform.processor(), 'python': sys.version,
                   'cpu_info': open('/proc/cpuinfo', encoding='utf-8').read(),
                   'selected_cpu': args.cpu, 'inherited_affinity': sorted(os.sched_getaffinity(0)),
                   'frequency_and_host_load_controlled': False, 'parallel_workers': False,
                   'timeout': args.timeout, 'address_space_mib': args.address_space_mib,
                   'samples_requested': args.samples, 'order_seed': args.order_seed,
                   'families': args.families, 'sizes': args.sizes, 'metrics': args.metrics, 'groups': args.groups,
                   'development_seed': DEVELOPMENT_SEED, 'holdout_seed': HOLDOUT_SEED,
                   'command': sys.argv, 'warmups': 1,
                   'selection': 'holdout family/size-equal geometric ratios; sqrt(T*M)<=0.95, each family T<=1.10 and M<=1.10; independent second run required'}
    save(args.output / 'environment.json', environment)
    records, rows = [], []
    tuning_hashes = {}
    for case_index, case in enumerate(fixtures(args.families, args.sizes, args.quick)):
        path = fixtures_dir / (case['name'] + '.bin')
        write_fixture(path, case['first'], case['second'])
        fixture_hash = sha256(path)
        fixture_key = (case['family'], case['size'])
        if case['split'] == 'tuning':
            tuning_hashes[fixture_key] = fixture_hash
        elif tuning_hashes.get(fixture_key) == fixture_hash:
            raise ValueError(f"holdout fixture repeats tuning geometry: {case['name']}")
        for metric in args.metrics:
            active = variants(metric, args.groups)
            keys = [f'{backend}:{variant}' for backend, variant in active]
            count = args.samples if args.quick else math.ceil(args.samples / len(keys)) * len(keys)
            record = {key: value for key, value in case.items() if key not in ('first', 'second')}
            record.update(metric=metric, fixture_sha256=fixture_hash, planned_samples=count,
                          workers={key: [] for key in keys}, validation='passed')
            references = {
                'public': invoke(commands['cocycle'], path, metric, 'public', args.timeout),
                'topp': invoke(commands['topp'], path, metric, 'default', args.timeout),
                'gudhi': invoke(commands['gudhi_bottleneck'] if metric == 'bottleneck' else gudhi,
                                path, metric, 'baseline', args.timeout),
            }
            record['references'] = references
            expected = None
            if all(value['status'] == 'completed' for value in references.values()):
                expected = unpack_number(references['topp'])
                if max(len(case['first']), len(case['second'])) <= 8:
                    expected = tiny_oracle(case['first'], case['second'], metric)
                    record['tiny_expected'] = number(expected)
                for reference in references.values():
                    if not agrees(unpack_number(reference), expected, case, metric):
                        reference['comparison_error'] = 'reference mismatch'
                        record['validation'] = 'failed'
            else:
                record['validation'] = 'failed'
            record['schedule'] = schedule(keys, count, args.order_seed + case_index)
            stopped = set()
            for entry in record['schedule']:
                key = entry['worker']
                backend, variant = key.split(':')
                if key in stopped:
                    sample = {'status': 'not_run', 'reason': 'earlier attempt for this worker failed'}
                else:
                    sample = invoke(commands[backend], path, metric, variant, args.timeout,
                                    args.address_space_mib, args.cpu)
                    if sample['status'] == 'completed':
                        if backend == 'topp' and sample.get('cpp_threads') != 1:
                            sample['comparison_error'] = 'native performance requires the serial C++ configuration'
                        elif any(not isinstance(sample.get(name), int) or sample[name] < 0
                               for name in ('rss_before_kib', 'hwm_before_kib', 'peak_rss_kib')):
                            sample['comparison_error'] = 'missing Linux memory counters'
                        elif expected is None or not agrees(unpack_number(sample), expected, case, metric):
                            sample['comparison_error'] = 'unverified or unequal distance'
                    if args.profile_rust and backend == 'cocycle' and sample['status'] == 'completed':
                        try:
                            sample['phases'] = phase_timings(sample['stderr'])
                            total = 'bottleneck.total' if metric == 'bottleneck' else 'wasserstein.total'
                            if total not in sample['phases']:
                                raise ValueError('missing Rust total phase')
                        except ValueError as error:
                            sample['comparison_error'] = str(error)
                    if sample['status'] != 'completed' or sample.get('comparison_error'):
                        stopped.add(key)
                        record['validation'] = 'failed'
                record['workers'][key].append({**sample, **entry})
            record['applicability'] = {}
            for key, samples in record['workers'].items():
                valid = [sample for sample in samples if sample['status'] == 'completed']
                if metric != 'bottleneck' and key.startswith('cocycle:'):
                    record['applicability'][key] = {
                        'sparse_layout_exercised': bool(valid) and all(
                            exercised_sparse_layout(sample['stats']) for sample in valid),
                        'note': 'requires positive edges and residual storage; empty calls and direct-cost fallbacks do not test arena storage',
                    }
                elif metric == 'bottleneck' and key.startswith('cocycle:'):
                    record['applicability'][key] = {
                        'routes': sorted({sample['stats'].get('route', 'unknown') for sample in valid}),
                        'matching_reuse_observed': any(sample['stats'].get('matching_reuses', 0) > 0 for sample in valid),
                        'scratch_reuse_observed': any(sample['stats'].get('scratch_reuses', 0) > 0 for sample in valid),
                    }
            if record['validation'] == 'passed':
                for key, samples in record['workers'].items():
                    summary = summarize(samples, count)
                    if summary is not None:
                        rows.append({'case': case['name'], 'family': case['family'], 'split': case['split'],
                                     'metric': metric, 'worker': key, **summary})
            records.append(record)
            save(args.output / 'results.json', records)
    final = identity()
    unchanged = final['source_sha256'] == initial['source_sha256'] and final['commit'] == initial['commit']
    passed = unchanged and all(record['validation'] == 'passed' for record in records)
    selections = []
    if passed and environment['evidence_class'] == 'comparative_performance':
        for metric in args.metrics:
            metric_rows = [row for row in rows if row['metric'] == metric]
            # Local forced-solver ablations cannot decide the adaptive default.
            candidates = ('cocycle:quickselect', 'cocycle:binary') if metric == 'bottleneck' else ('cocycle:adaptive_arena',)
            for candidate in candidates:
                selections.append({'metric': metric, **select(metric_rows, candidate)})
    summary = {'protocol_id': PROTOCOL, 'status': 'passed' if passed else 'failed',
               'evidence_class': environment['evidence_class'], 'sources_unchanged': unchanged,
               'planned_cases': len(records), 'passed_cases': sum(r['validation'] == 'passed' for r in records),
               'selection': selections, 'finished_utc': datetime.now(timezone.utc).isoformat()}
    save(args.output / 'measurements.json', rows if passed else [])
    save(args.output / 'summary.json', summary)
    print(f"Distance benchmark: {summary['status']} ({summary['passed_cases']}/{len(records)} cases)")
    return 0 if passed else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    add_build_arguments(parser)
    parser.add_argument('--quick', action='store_true', help='small resource snapshot; never select an algorithm')
    parser.add_argument('--exploratory', action='store_true', help='allow dirty exploratory runs; never select an algorithm')
    parser.add_argument('--profile-rust', action='store_true',
                        help='generated safe Rust phase scopes; diagnostics only, never selection')
    parser.add_argument('--samples', type=int, default=12)
    parser.add_argument('--cpu', type=int)
    parser.add_argument('--order-seed', type=int, default=0)
    parser.add_argument('--address-space-mib', type=int, default=2048)
    parser.add_argument('--families', nargs='+', choices=FAMILIES, default=list(FAMILIES))
    parser.add_argument('--sizes', nargs='+', type=int)
    parser.add_argument('--metrics', nargs='+', choices=METRICS, default=list(METRICS))
    parser.add_argument('--groups', nargs='+', choices=GROUPS, default=list(GROUPS),
                        help='explicit ablation groups; baseline is always measured')
    args = parser.parse_args()
    args.sizes = args.sizes or ([8] if args.quick else [8, 32, 128, 512])
    if any(size < 1 for size in args.sizes):
        parser.error('sizes must be positive')
    for name in ('families', 'sizes', 'metrics', 'groups'):
        values = getattr(args, name)
        if len(values) != len(set(values)):
            parser.error(f'{name} must not contain duplicate entries')
    try:
        return run(args)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
