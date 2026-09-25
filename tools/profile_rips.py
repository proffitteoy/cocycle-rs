"""Profile private H1 optimization stages in separate release test processes.

Only standard-library Python is needed. Inputs are COCYCLE1 distance/H1 binary
fixtures. Two-pass initialization and virtual pairs can be measured separately
or together. Test instrumentation is enabled, so these timings are diagnostic
and must not replace the public-API cross-library benchmark.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fixtures", type=Path, nargs="+")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cpu", type=int)
    args = parser.parse_args()
    if args.output.exists():
        parser.error("output must be new")
    if args.cpu is not None:
        os.sched_setaffinity(0, {args.cpu})
    repo = Path(__file__).resolve().parents[1]
    build = subprocess.check_output(
        ["cargo", "test", "--locked", "--offline", "--release", "--lib", "--no-run",
         "--message-format=json"], cwd=repo, text=True,
    )
    artifacts = [json.loads(line) for line in build.splitlines()]
    executable, = [x["executable"] for x in artifacts
                   if x.get("reason") == "compiler-artifact" and x.get("executable")]
    digest = hashlib.sha256()
    for path in sorted((repo / "src").rglob("*.rs")):
        digest.update(str(path.relative_to(repo)).encode())
        digest.update(path.read_bytes())
    record = {
        "source_sha256": digest.hexdigest(),
        "rustc": subprocess.check_output(["rustc", "-Vv"], text=True),
        "platform": platform.platform(), "cpu": args.cpu,
        "protocol": "release lib tests; instrumentation; one warmup, five samples; fresh process per stage; reference checked after measurement",
        "results": [],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    # Failures leave a partial diagnostic record; completion is explicit.
    for fixture in args.fixtures:
        for stage in ["explicit", "clearing", "implicit", "cone", "apparent",
                      "two-pass", "emergent", "virtual-two-pass", "virtual"]:
            print(f"{fixture.stem}: {stage}", flush=True)
            env = dict(os.environ, COCYCLE_ABLATION_FIXTURE=str(fixture.resolve()),
                       COCYCLE_ABLATION_STAGE=stage)
            output = subprocess.check_output(
                [executable, "persistence::flag::cohomology::profiling::profile_stage",
                 "--exact", "--ignored", "--nocapture"],
                cwd=repo, env=env, text=True, timeout=180,
            )
            row, = [json.loads(line.split("ABLATION ", 1)[1])
                    for line in output.splitlines() if "ABLATION " in line]
            row.update(fixture=fixture.name, fixture_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest())
            row["median_ms"] = statistics.median(row["samples_ms"])
            record["results"].append(row)
            args.output.write_text(json.dumps(record, indent=2) + "\n")
    record["status"] = "passed"
    args.output.write_text(json.dumps(record, indent=2) + "\n")


if __name__ == "__main__":
    main()
