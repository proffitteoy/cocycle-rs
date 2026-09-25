# Performance reporting rules

[Benchmarks](README.md)

This page owns the rules for new performance claims and retained reports. Each
suite owns its execution contract: [H0/H1 native](protocol.md),
[Rips pipeline](pipeline/README.md) or [diagram distances](distances/README.md).
Use the [report template](report-template.md)
for a new experiment. The source repository stores code and concise reports;
all generated run data lives outside Git.

## Classify the evidence

| Evidence class | Supports | Does not establish |
| --- | --- | --- |
| Correctness / smoke validation | Agreement on the exercised contracts; working adapters | Performance rankings or scaling claims |
| Resource snapshot | Observed time, phase costs and process memory on specified workloads | Stable rankings, universal speedups or asymptotic bounds |
| Comparative performance study | Scoped cross-library or before/after conclusions with repeated, comparable measurements | Behavior on unmeasured inputs, environments or revisions |

State the class and question before the numbers. New external performance
measurements use native GUDHI C++ and upstream Ripser C++ for persistence, and
Topp C++ for diagram distances. Python may orchestrate processes; isolated
GUDHI/POT distance workers supply correctness evidence only and do not qualify
as native timing comparisons.
Name the actual engine and adapter, not just the library: GUDHI direct expansion,
GUDHI edge collapse and GUDHI's integrated Ripser are different paths.

## Bind reports to changes and measured commits

A report has a stable topic-based path, such as `reports/rips-comparison.md`.
Update that file after a rerun; Git history preserves previous reports. Do not
create date-, PR- or commit-named copies, append a growing run diary, or archive
old reports elsewhere in the repository. The report body identifies the actual
measured code with an immutable full commit SHA and, when applicable, a PR.
A filename, report commit, branch name or moving PR head is not a measured revision.
Record these fields explicitly:

| Identity | Required meaning |
| --- | --- |
| PR | Repository and PR URL/number, when applicable; otherwise state no associated PR |
| Measured candidate | Full Rust commit SHA and source fingerprint; name whether it is the PR head or a tested merge commit |
| Measured baseline | Full SHA for a before/after study; for cross-library-only evidence state not applicable and retain upstream pins |
| Harness | Worker/controller/protocol revision and fingerprint when distinct from the measured Rust revision |
| Run | Suite and unique attempt ID; UTC start/end are metadata, not the report key |

Keep one maintained report per comparison topic. Related suites may share a
report, but their contracts and tables must remain separate. Reruns replace the
current results, measured identities, environment, coverage and conclusions
together. Retain unfavorable outcomes; if a rerun fails, show that outcome and
label any retained successful baseline with its original revision. When only
part of a comparison is rerun, label each section's revision and attempt; never
present mixed revisions as one fresh measurement. Explain invalid attempts for
the current comparison briefly, without retaining a historical results appendix.

Only artifact directories identify individual revisions and attempts. Use
`target/benchmarks/commit-<sha12>/<suite>/run-<NNN>/` locally and the same identity
in external storage. These directories are never committed. New attempts get
fresh directories; updating the Markdown report must not overwrite raw runs.

For a before/after report, record both candidate and baseline explicitly and link
each run's artifacts under its own measured revision. The PR target branch is not
a baseline identity. Rebases, squashes and merges produce new commit identities;
results for the previous head remain evidence for that head. Do not relabel them
as a measurement of the resulting merge commit.

Commit the implementation and harness before a formal measurement, verify that
the measured inputs are clean, then update the report in a subsequent
commit. Raw artifacts stay outside Git. The report's own commit is not the measured
commit. A documentation-only follow-up can cite the prior measured revision
without rerunning it, but must not claim that the new revision was measured.

Uncommitted exploratory runs remain local under `target/`, identified by their
source fingerprint and dirty state. They are not version-bound performance
reports and are not committed as draft archives. A report can cite a run only
after verifying the measured inputs against a commit or making a fresh committed
run. A later commit must not be assigned merely because it contains similar work.

Report only identities actually recorded. Controllers that do not emit all
required fields need additional run metadata in the external artifact. The
commit containing the report is distinct from the measured commit.

## Establish comparability before measuring

Record the following contract for each workload family. Split rows when any
requested work differs; sharing a final diagram is not sufficient.

| Contract | Required information |
| --- | --- |
| Input | Fixture hash, generator/seed, vertex and edge counts, point dimension or matrix layout, metric/nonmetric assumptions |
| Filtration | Exact Rips, supplied flag graph or sparse approximation; edge-length convention, cutoff inclusivity and missing-edge meaning |
| Algebra and coverage | Homology dimensions, construction dimension, coefficient field, zero-length pair policy, essential/censored endpoints |
| Approximation | Epsilon, minimum radius, initial vertex/tie policy, sampling provenance, blockers, and checked/assumed hypotheses |
| Requested output | Diagram, retained explicit complex/incidence, cycles/cocycles and query scales, exported payload |
| Representation | Scalar widths, input conversions/copies, retained buffers and native structures |

Generate shared fixtures once. When a reference needs float32 inputs, quantize
once for all backends and retain the exact values; do not loosen tolerances to
hide filtration changes. Point-distance or numerical comparisons needing a
tolerance must document its independent justification.

Validate every measured output under the suite's contract, including interval
multiplicity, dimensions and endpoint meaning. Requested topology or bases need
additional structural/algebraic validation. A diagram-only native reference
cannot validate representative vectors or compete with basis extraction timings.
The same restriction applies to precomputed matrices versus point evaluation,
unchecked versus exhaustively checked metrics, and implicit diagrams versus
requested explicit complexes. Mark these references `correctness_reference_only`
where the suite supports that label; leave their comparison cells empty with an
explanation. Unsupported capabilities are explicit exclusions, never zero times.

## Identify the execution protocol

Link the exact protocol and record the worker/controller source fingerprint.
Use the machine-emitted protocol identity when available (`cocycle-native-v1`
for H0/H1, `cocycle-rips-pipeline-v2` for the pipeline, and
`cocycle-distance-v1` for diagram distances). Preserve the description
and source hash too. Earlier unversioned pipeline runs used a fixed backend order.
A prose label alone must not imply a new protocol was executed.

The table below compares the two persistence protocols. The
[distance protocol](distances/README.md#timing-and-sampling) separately owns its
f64 endpoint format and validation/preparation/solve/cleanup boundary. It uses
fresh serial workers with a discarded warmup, records process RSS and capacity
counters separately, and excludes GUDHI/POT times from rankings. Distance
ablation workers compile counters explicitly; they do not measure the ordinary
uninstrumented public API. Preserve that distinction in every report.

| Boundary | H0/H1 native | Rips pipeline |
| --- | --- | --- |
| Work | Precomputed ordinary F2 H0/H1 | Matrix/graph/point, explicit, prime-field, representative and approximate workflows |
| Start | After validation and native input preparation | Before measured validation/conversion and construction |
| Ripser dense f64-to-f32 conversion | Before timer | Construction phase inside timer |
| End | Owned normalized intervals and algorithm cleanup, before JSON formatting | Interval payload export and workflow bookkeeping; final metrics transport excluded |
| Samples | Fresh processes; no warmup; shuffled backend order | Fresh processes; one discarded warmup; seeded, position-balanced rounds |
| Last memory reading | Before JSON serialization | After computation/export |

Both exclude fixture I/O and process startup from their internal times. Worker
wall-time limits cover more than that internal window. Record result lifetime
and intermediate destruction boundaries, not just a column called "runtime".
Do not pool samples or compute speedups across these protocols. A future change
to timing, precision, output or preparation requires a distinct protocol revision
and fresh measurements for all compared implementations.

End-to-end time answers a workflow question. Phase times answer narrower
questions only when their boundaries match. Public computation may include result
assembly; it is not automatically reducer-only time. Do not subtract independent
medians to estimate a missing phase or assume phase medians sum to the total.
Native execution avoids wrapper overhead; it does not imply zero copying.

## Plan samples and report uncertainty

Record workload selection, repetition count, warmup policy, backend order and
resource limits before the run. Include easy and difficult families, cutoffs,
fields and outputs relevant to the stated claim. For a before/after study, keep
the fixtures, native references, toolchain, hardware and protocol fixed and record
both Rust source identities. Retain regressions as well as improvements.

Resource snapshots may use fewer samples with an explicit limitation. For a new
comparative performance study, use at least ten independent measured processes
per compared cell as a project minimum, and increase repetitions or narrow the
claim when variation remains large. Ten is a reporting floor, not a statistical
guarantee. Warmups are retained but excluded from statistics. Publish all measured
samples, the median and a spread measure (at least minimum/maximum). Quantiles or
confidence intervals must state their calculation method and sample count.

Run workers serially without concurrent compilation or testing. Record CPU,
OS, compiler/build flags, affinity and any frequency/load controls; explicitly
state uncontrolled factors. Pipeline v2 defaults to 12 measured rounds with
seeded, position-balanced backend order and optional `--cpu` affinity. It records
both selected and inherited affinity; frequency and host load are not controlled
by the harness. More repetitions alone do not remove these sources of variation.
CI smoke timings are not performance baselines.

Do not discard outliers or retry until a favorable run appears. Preserve all
attempts and explain an invalidated run before replacing it. A timeout is a
censored observation under a limit, not the limit itself as a measured runtime.
Crashes, mismatches and incomplete sample groups cannot be ranked. Show planned,
successful, excluded and failed coverage, with the reasons and original outcomes.

Label a ratio's direction: `reference_ms / cocycle_ms` is greater than one when
Cocycle is faster on that comparable row. Prefer per-workload results. An aggregate
needs a predeclared workload set, formula/weights and coverage; never silently drop
hard cases or combine incompatible protocols. A selected table must link the
complete matrix and explain the selection. Report enough digits to inspect the
data without implying precision beyond its variability.

## Report memory as measured

Keep prepared-input RSS, prepared-input high-water mark, absolute process peak,
and high-water growth distinct. State sampling boundaries and units (KiB/MiB).
High-water growth can be zero despite allocations. Process RSS includes runtime,
parser/input buffers, allocator retention, temporaries and retained outputs; it is
not live algorithm allocation. Different f32/f64 widths, trees and stored incidence
must accompany memory comparisons. An algorithm-memory claim needs a separate
allocation measurement with equivalent ownership boundaries.

Distinguish process timeout and `RLIMIT_AS` address-space caps from the library's
cooperative cancellation/work limits. Neither process peak RSS nor successful
completion of a fixture establishes a hard library memory budget.

## Storage and evidence lifecycle

| Location | What belongs there | Retention |
| --- | --- | --- |
| Source Git repository | Library code, tests, benchmark workers/generators, protocols and concise PR/commit-bound reports | Maintained source history |
| Small test fixtures | Hand-maintained inputs required by a specific regression test | Reviewed as source; no generated benchmark corpus |
| `target/` | Local measurements, generated fixtures, environments, raw outputs, logs, profiles and archives | Disposable local work; never committed |
| GitHub Actions artifacts | CI comparison outputs, failure diagnostics and run metadata | Currently 14 days in this repository |
| Dedicated external artifact storage | Selected reproducible performance experiments needed beyond CI retention | Explicit durable URL, checksum and retention policy |

Do not commit raw results, generated fixture collections, build/test logs, full
machine/header inventories, binaries, caches or archives. Compressing them into
one file does not make them source. `benches/results/` is not an artifact store;
old data and reports have been removed, with no historical exemption.

GitHub's [workflow artifacts](https://docs.github.com/en/actions/tutorials/store-and-share-data)
are designed to store run outputs separately from source, with configurable
retention. They are not permanent evidence: record run ID, attempt, measured SHA,
artifact URL and expiry. A durable performance claim needs externally preserved
raw samples, fixtures, validation outcomes, environment/build metadata and hashes.
Publish an immutable external archive with a member checksum manifest if needed;
put only its identity, link, checksum and selected conclusions in the report.
No durable external experiment store is configured yet. Local-only or expired
evidence cannot be described as publicly reproducible.

The [Rust compiler performance project](https://github.com/rust-lang/rustc-perf)
separates per-commit collection and performance presentation into dedicated tools.
[airspeed velocity](https://asv.readthedocs.io/en/stable/using.html) likewise warns
that result data can grow large and needs an explicit storage plan. These are
examples of separating benchmark code from data management, not requirements to
add a database or service to this Rust library.

For every selected experiment:

1. Measure a known source revision into a fresh ignored local directory or CI run.
2. Validate all outcomes, including failures, exclusions and unfavorable cases.
3. Preserve the full run in external storage when a lasting report needs it.
4. Commit a concise report with the exact revisions, protocol, outcome, comparison
   scope and evidence location/status; update its stable path instead of adding
   another report. Do not copy the raw run into Git.

Use the [report template](report-template.md). Prefer the PR description and CI
job output for routine checks; a successful test run does not need a new document.
Only retain a repository report when it explains an enduring result or decision.
A maintained comparison may summarize local measurements when it records their
exact identities, reproduction commands and explicitly local-only evidence
status. An external upload is not a prerequisite for that summary. It must not
claim that the original evidence is publicly retrievable. Keep selected tables
and interpretation in the report; full matrices, samples and logs stay outside
Git. Do not add dead links into ignored `target/`; give local artifact paths as
code and external URLs only when actually available.

After staging, run `python3 tools/check_artifacts.py`. CI repeats this check. It
rejects tracked result directories, cache/build paths, log files and archive/
binary suffixes; forced additions are checked too. The check does not infer
whether arbitrary JSON or prose is generated, so review still owns that boundary.
There are no grandfathered artifact blobs. `.gitignore` helps keep local outputs
out of the index but is not the enforcement mechanism.

Storage changes do not change a measurement's source identity. Documentation-only
changes need documentation checks, not a benchmark rerun. Missing evidence stays
explicit. Keep correctness, local package checks, hosted CI for an exact commit
and publication status distinct. Removing artifacts from a branch does not erase
objects already present in Git history; history rewriting is a separate operation.
