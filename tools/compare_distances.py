"""Check real Rust distances against pinned Topp, GUDHI and tiny exhaustive values."""

import argparse
from datetime import datetime, timezone
import subprocess
import sys

from distance_common import (METRICS, PROTOCOL, WORKERS, add_build_arguments, agrees,
                             build_from_args, correctness_cases, identity, invoke,
                             number, save, sha256, tiny_oracle, tolerance,
                             unpack_number, write_fixture)


def run(args):
    started = datetime.now(timezone.utc).isoformat()
    initial = identity()
    commands, build = build_from_args(args)
    commands['gudhi'] = [args.gudhi_python, str(WORKERS / 'gudhi_worker.py')]
    fixtures = args.output / 'fixtures'
    fixtures.mkdir()
    save(args.output / 'environment.json', {
        **initial, 'protocol_id': PROTOCOL, 'evidence_class': 'correctness',
        'started_utc': started, 'python': sys.version, 'build': build,
        'gudhi_python': args.gudhi_python,
        'tolerance': 'exact small dyadic bottleneck/W1; otherwise 64*epsilon*(n+m+1)*max(cost scale,expected)',
    })
    records = []
    for case in correctness_cases(args.quick):
        stress = case['name'] in ('adjacent_float_diagonal', 'large_finite', 'small_finite')
        if (args.suite == 'supported' and stress) or (args.suite == 'stress' and not stress):
            continue
        path = fixtures / (case['name'] + '.bin')
        write_fixture(path, case['first'], case['second'])
        for metric in METRICS:
            expected = tiny_oracle(case['first'], case['second'], metric)
            record = {'case': case['name'], 'metric': metric, 'fixture_sha256': sha256(path),
                      'expected': number(expected),
                      'absolute_tolerance': tolerance(case, metric, expected) if expected != float('inf') else 0,
                      'workers': {}, 'validation': 'passed'}
            references = {name: commands[name] for name in ('cocycle', 'topp')}
            references['gudhi'] = commands['gudhi_bottleneck'] if metric == 'bottleneck' else commands['gudhi']
            if metric == 'bottleneck':
                references['hera'] = commands['gudhi']
            for backend, command in references.items():
                variant = 'public' if backend == 'cocycle' else 'default' if backend == 'topp' else 'baseline'
                sample = invoke(command, path, metric, variant, args.timeout)
                if sample['status'] == 'completed':
                    if not agrees(unpack_number(sample), expected, case, metric):
                        sample['comparison_error'] = 'independent tiny oracle mismatch'
                        record['validation'] = 'failed'
                else:
                    record['validation'] = 'failed'
                record['workers'][backend] = sample
            if (record['validation'] == 'failed' and case['name'] == 'adjacent_float_diagonal'
                    and record['workers']['cocycle']['status'] == 'completed'
                    and not record['workers']['cocycle'].get('comparison_error')):
                record['validation'] = 'reference_discrepancy'
                record['note'] = 'strict nextafter diagonal stress: retain oracle disagreement; never widen tolerance'
            records.append(record)
            save(args.output / 'results.json', records)
    final = identity()
    unchanged = final['source_sha256'] == initial['source_sha256'] and final['commit'] == initial['commit']
    summary = {'protocol_id': PROTOCOL, 'evidence_class': 'correctness',
               'status': 'passed' if unchanged and all(r['validation'] == 'passed' for r in records) else 'failed',
               'planned_cells': len(records), 'passed_cells': sum(r['validation'] == 'passed' for r in records),
               'sources_unchanged': unchanged, 'finished_utc': datetime.now(timezone.utc).isoformat(),
               'failed': [{'case': r['case'], 'metric': r['metric']} for r in records if r['validation'] != 'passed']}
    save(args.output / 'summary.json', summary)
    print(f"Distance correctness: {summary['status']} ({summary['passed_cells']}/{len(records)} cells)")
    return 0 if summary['status'] == 'passed' else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    add_build_arguments(parser)
    parser.add_argument('--quick', action='store_true')
    parser.add_argument('--suite', choices=('supported', 'stress', 'all'), default='supported',
                        help='stress retains numerical reference discrepancies and returns nonzero for disagreements')
    args = parser.parse_args()
    try:
        return run(args)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
