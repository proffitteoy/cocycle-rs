# Native diagram-distance experiments

[Benchmarks](../README.md) / [Reporting rules](../reporting.md)

Use this suite to check bottleneck, W1 and W2 against independent references,
then compare memory and search choices within Rust and C++. Protocol
`cocycle-distance-v1` preserves f64 inputs and has its own timing boundary;
do not pool its samples with either Rips suite. For library usage, start with
the [distance example](../../examples/diagram_distances.rs) and
[matching contract](../../docs/reference/mathematics.md#16-diagram-matching-distances).

## Prepare the reference environment

Requirements are Rust 1.91+, Python 3.10+, a GCC-compatible C++20 compiler, Git,
CGAL and Boost headers for the independent fixed GUDHI bottleneck oracle.
The pinned NumPy oracle environment requires Python 3.11 or later; the standard-
library controller itself supports Python 3.10.
The benchmark controller requires Linux, including for resource smoke checks.
Windows can run correctness checks with a GCC-compatible toolchain; this builder
does not support MSVC.
The library gains no dependencies. [sources.json](sources.json) pins Topp and the
exact GUDHI/POT/NumPy versions. Use a dedicated clean external source checkout:

```sh
git clone https://github.com/proffitteoy/Topp.git target/native-sources/topp
git -C target/native-sources/topp checkout --detach ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234
git clone https://github.com/GUDHI/gudhi-devel.git target/native-sources/gudhi-distance
git -C target/native-sources/gudhi-distance checkout --detach 4ec34ac55e6d2e8cfd7c322e84c1b0a56d516d51
python3 -m venv target/distance-oracle-venv
target/distance-oracle-venv/bin/python -m pip install gudhi==3.11.0 numpy==2.4.6 POT==0.9.6.post1
```

These commands use Linux virtual-environment paths; on Windows the interpreter
is `target/distance-oracle-venv/Scripts/python.exe`.
`--topp-source`, `--cargo`, `--rustc`, `--cxx`
and `--gudhi-python` accept explicit paths. The builder rejects wrong/dirty Topp
and GUDHI sources. `--gudhi-source`, `--cgal-include` and `--boost-include` locate
the fixed native oracle and non-system headers. The builder reads external
sources and never installs packages, resets, or modifies an external checkout.
It fingerprints source, native binaries, commands and toolchains.

## Check correctness and the harness

Run from the repository root. Every output directory must be new; change the
attempt suffix when repeating a command.

```sh
python3 tools/compare_distances.py --quick --gudhi-python target/distance-oracle-venv/bin/python --output target/distance-correctness-001
```

This checks the small supported suite. Success prints `Distance correctness:
passed (...)` and writes `summary.json` with `status: "passed"`. Omit `--quick`
for the full supported suite. Missing references and numerical disagreements
return a nonzero exit status. Once workers run, their failures and comparison
results are retained in `results.json`; build failures remain in `build/build.log`.

On Linux, check benchmark workers with a small resource snapshot:

```sh
python3 tools/benchmark_distances.py --quick --samples 1 --exploratory --groups baseline arena --families uniform sparse --gudhi-python target/distance-oracle-venv/bin/python --output target/distance-smoke-001
```

Success prints `Distance benchmark: passed (...)`. Both `--quick` and
`--exploratory` exclude the run from algorithm selection; `--exploratory` also
allows uncommitted sources. Numerical stress is a separate diagnostic:

```sh
python3 tools/compare_distances.py --suite stress --gudhi-python target/distance-oracle-venv/bin/python --output target/distance-stress-001
```

The pinned references have [known stress disagreements](#reference-backends-and-numerical-limits),
so a nonzero stress exit must be inspected separately from supported acceptance.

## Native build and routing controls

The builder compiles the ordinary public crate without instrumentation, then
compiles the standalone Rust worker with `--cfg cocycle_distance_bench`. This
private cfg enables the counter schemas and forced policies also used by kernel
tests. Conditional compilation removes counter statements, capacity traversals,
binary-search ablation and experimental arena storage from ordinary library
builds, including debug. The worker's `public` variant calls the separately
linked ordinary crate; native ablations call the instrumented private kernels.
Build metadata records this boundary. There is no public Cargo experiment feature.

Topp uses a portable scalar build with MSVC-only AVX2 dispatch disabled. Weighted
Topp matching uses compiler-dependent `long double`; Rust uses f64. Preserve
these representation differences when interpreting results.
The correctness `default` variant keeps Topp's unrestricted adaptive defaults.
Its `dense_parallel` candidate route can spawn threads at 262144 pairs. Measured
C++ variants instead use a serial configuration: the same pinned four-row density
test replaces only that route with `dense_blocked`. Prepared diagrams and the
extra routing check are counted inside the clock; inputs are prepared once.
Original Topp source remains unchanged. Records identify `cpp_threads: 1`, whether
the serial override applied, and a post-call observed thread count. The latter
alone is not a peak-thread measurement or the reason to assert serial behavior.

## Inputs, outputs and independent correctness

All workers read the same binary fixture: `COCDST1\0` (eight bytes), little-endian
u64 left/right counts, then left and right interleaved little-endian f64 endpoint
pairs. Exact length is checked; no f32 quantization occurs. Original bits, order
and multiplicity remain in the fixture.

Correctness compares one complete computed dimension with finite births and
ordered finite deaths or positive infinity. The Rust public adapter explicitly
removes finite diagonal points when making typed intervals to match Topp's raw
convention; this is not acceptance of diagonal points by `PersistenceInterval`.
The library tests separately exercise censored coverage, uncomputed dimensions,
invalid endpoints and provenance rejection. Performance fixtures are strictly
finite and off-diagonal.

The pinned GUDHI weighted wrapper fixes POT's network-simplex limit at 2,000,000
iterations. An iteration-limit warning is retained as an unavailable reference,
even if POT returned a scalar. Such a cell remains incomplete and is never
ranked; no automatic budget increase or tolerance relaxation is performed.

Each process receives `fixture metric variant`. Metrics are `bottleneck`
(L-infinity), `w1` (order 1, L-infinity), and `w2` (order 2, Euclidean, final square
root included). JSON identifies protocol/backend/metric/variant and retains the
scalar, time, process memory and counters. Infinity is `value_kind: "infinite"`
with `value: null`. NaN, negative results, crashes and malformed output are errors.

The tiny independent oracle enumerates every partial injection, including
unmatched points' diagonal costs, using exact rational arithmetic before the W2
square root. It shares no production search, graph or matcher code. Hand-derived,
empty, repeated, unequal-size, near-diagonal, negative-scale, tied and essential
examples supplement deterministic random inputs. Every supported backend is
checked against independent expectations, not merely Rust/Topp mutual agreement.

## Reference backends and numerical limits

The primary GUDHI bottleneck reference is the native `e=0` implementation at the
fixed head of [merged PR #1367](https://github.com/GUDHI/gudhi-devel/pull/1367),
which corrects the premature matching shortcut. It uses double-coordinate KD
trees and builds with `CGAL_DISABLE_GMP`; it is a correctness oracle only. Full
source identity and header hashes are retained. The correctness suite additionally
checks GUDHI 3.11.0 Hera bottleneck `delta=0`. Wasserstein uses explicit order/internal norm with
`keep_essential_parts=True`, no autodiff and POT's exact transport solver. The
isolated worker uses NumPy and loads only the requested metric's native backend;
Wasserstein disables optional POT GPU/autodiff imports. Hera requires removal of
diagonal points, which does not change the raw diagram distance. The pinned Linux
wheel's default `gudhi.bottleneck_distance(e=0)` returned 2 instead of the independent
oracle's 1.375 on an ordinary tiny fixture, even before POT was loaded. Its failed
attempt is retained separately; it is not counted as agreement. Hera's zero-delta
mode is independently checked on supported inputs and retains its own extreme-
value limitations in stress results. GUDHI/POT Python workers run in isolated
processes for correctness and never qualify as native timing references.
Backend identities are fixed in `sources.json`.
GUDHI 3.11.0 predates PR #1367 (merged 2026-08-27 and labeled 3.14.0); its default
backend is not the acceptance reference. Do not treat that known old-version
failure as a new Rust defect or silently call Hera the repaired implementation.
Versions/settings are retained; a missing reference makes validation fail.
Small dyadic bottleneck/W1 cases use zero tolerance. Other cases use the fixed
bound `64 * f64_epsilon * (n+m+1) * max(cost_scale, |expected|)`, with scale from
endpoint differences and diagonal costs, not the absolute coordinate origin.
Never increase tolerances after observing a mismatch.

`--suite supported` is the default. `--suite stress` checks very large/small and
adjacent-float cases; `--suite all` includes both. Pinned Topp's rounded-midpoint
diagonal projection disagrees with `(death-birth)/2` at adjacent floats. Stress
retains the real disagreement and returns nonzero, rather than labeling it as
agreement or broadening tolerance. Rust correctness and reference disagreement
remain separate evidence.

## Timing and sampling

One fresh native process computes one pair. Parsing and initial raw-pair layout
finish before timing. Validation, diagonal projection, preparation, solve and
temporary cleanup are counted; startup, fixture I/O and JSON formatting/transport
are excluded. All Rust native variants share the same finite validation/copying
boundary; the separate `public` adapter is correctness-only. Statistics counters
run inside the clock for both languages: these are instrumented time-to-result
measurements, not uninstrumented kernels. Raw inputs survive computation; solver
temporaries are destroyed before the clock stops. Copies/parser high-water marks
remain part of process memory.

Retain/discard one fresh warmup per cell; formal comparisons use at least twelve
measured processes. Sample counts round upward to a multiple of active variants.
Seeded shuffled cyclic blocks balance execution positions. Workers run serially,
single-threaded, without concurrent compilation or testing. Optional `--cpu` pins
to an allowed Linux CPU. Selected/inherited affinity and uncontrolled frequency
and host load are recorded.

Development seed is `20260922`, holdout seed is `20260923`. Families include
uniform, clustered, near-diagonal, duplicates, imbalance, separated, threshold
shell and sparse/dense adversarial cases. Default sizes are 8/32/128/512; request
2048/4096 explicitly where limits permit. `--families`, `--sizes`, `--metrics` and
`--groups` preregister a smaller study; its conclusions remain scoped to that set.

Each family varies its actual geometry across the two seeds, including repeated
templates and regular sparse/threshold cases. The controller rejects identical
tuning and holdout fixture hashes for the same family and size.

## Controlled optimization contrasts

| Group | Controlled contrast |
| --- | --- |
| `baseline` | Topp serial adaptive configuration and Rust adaptation, always retained; unrestricted default remains a correctness reference |
| `search` | Forced quickselect versus binary with other settings held fixed within each language; adaptive baseline remains a separate row |
| `scratch` | Rust forced refinement with/without scratch reuse |
| `matching` | Rust forced refinement with/without matching reuse |
| `clipping` | Forced quickselect with/without candidate clipping in both languages |
| `arena` | Forced sparse vectors versus the arena plus scratch/heap-reuse candidate in both languages, plus Rust `adaptive_arena` under its ordinary default routing |

Topp has no corresponding exposed scratch/matching-reuse controls or adaptive
arena layout switch; these missing counterparts are limitations, never invented
equivalent experiments. Local sparse variants disable duplicate/component/greedy
shortcuts and keep those settings fixed within their pair. They cannot select an
adaptive default alone: `adaptive_arena` tests the actual candidate integration.
Route/counter records show whether the intended path ran. Rust
`direct_cost_fallbacks > 0`, `sparse_solves == 0`, no positive edges, or zero
residual-storage capacity does not test arena layout. Empty sparse calls can
return before constructing a network. The weighted arena contrast includes
scratch/heap reuse across augmentations; it does not isolate that reuse from
contiguous storage. Bottleneck scratch reuse has its own separate control.
Per-language retained/disabled ratios answer whether an optimization survives
migration; cross-language absolute times answer a different question.

## Optional kernel diagnostics

The benchmark-only `--profile-rust` flag copies the private Rust distance sources
and worker into the fresh artifact build directory, then inserts safe RAII
timing scopes at uniquely checked function markers. Production files and the
ordinary worker build stay unchanged. Generated sources retain `forbid(unsafe_code)`,
and build metadata records every original/generated source hash and hook.

Each Rust sample retains calls and integer `elapsed_ns` by phase alongside raw
stderr. Scopes include their nested calls and hook overhead; they must not be
summed as disjoint phases. Early returns close their scopes through `Drop`.
Preparation, candidate generation, KD construction/decisions, weighted graph
generation/components and dense/sparse/direct-cost solves can be inspected.
Public-library correctness references remain uninstrumented. A profiling run
always has evidence class `kernel_diagnostics` and cannot select an algorithm,
even with 12 samples. C++ is a numerical control here, not a timing comparator.

## Memory metrics

Workers read pre-call `VmRSS`/`VmHWM` and final `VmHWM` after cleanup, before JSON.
Report absolute peak and high-water growth separately; zero growth does not mean
no allocations. RSS includes runtime, input, allocator retention and outputs.
Explicit container capacities are not allocator peaks.
Rust Wasserstein records separate maximum capacities for retained graph storage,
residual storage (including vector headers or arena offsets), and search scratch
(vectors and heap). These maxima exclude construction temporaries and must not
be summed into a simultaneous memory peak. Compare them only within matching
categories and representation sizes; C++ long-double edges can be larger.
`--address-space-mib`
(default 2048 MiB) caps virtual address space, and `--timeout` (default 60 seconds)
limits whole-process wall time. Neither is a library budget; timeout is censored
evidence, not a measured runtime equal to the limit.

## Selection gates and repeat procedure

`results.json` retains every sample, warmup, order, exit and mismatch. After a
worker fails, its remaining attempts are `not_run` while others continue. No
mismatched/incomplete case is ranked. Valid summaries include median/min/max and
RSS. Changed sources invalidate a run. `--quick` and `--exploratory` are resource
snapshots and never select algorithms; formal measurements reject dirty sources.

Selection compares candidate/R0 time and peak-RSS ratios, with equal size weight
within each family and equal family weight. R0 is the Rust adaptive baseline
(`cocycle:baseline`). Ratios use per-case medians, then geometric means across
sizes and families. The held-out composite
`sqrt(time_ratio * peak_rss_ratio)` must be at most 0.95; neither metric may exceed
1.10 in any default-applicable family. A first qualifying result is only
`eligible_pending_independent_repeat`: repeat the same gates in a second fresh
run on identical sources/fixtures before deciding. Noise and fixed process
overhead are inconclusive; retain the simpler safe baseline on ties. Forced
local ablations are not automatically global default candidates.

Commit implementation and harness before formal measurement, then record the
full commit SHA. On Linux, the following runs all default families, metrics and
ablation groups at sizes 8/32/128/512 with development and held-out inputs:

```sh
distance_revision=$(git rev-parse --short=12 HEAD)
python3 tools/benchmark_distances.py --samples 12 --order-seed 2401 --gudhi-python target/distance-oracle-venv/bin/python --output "target/benchmarks/commit-${distance_revision}/distance/run-001"
```

For selection, repeat on the same committed sources and fixture hashes in a new
output directory with a different order seed. Apply the gates to both rounds;
the controller reports only eligibility and never changes the library default.
Specify `--families`, `--sizes`, `--metrics` and `--groups` before execution if
limiting the study. Formal commands are reproduction instructions, not evidence
that a run has completed. Shared-host load and fixed RSS overhead can leave
differences inconclusive.

## Inspect and retain artifacts

| Artifact | Interpretation |
| --- | --- |
| `summary.json` | Overall validation, source consistency, and benchmark selection eligibility |
| `results.json` | Raw reference results, samples, warmups, execution order, failures and route counters |
| `measurements.json` | Benchmark summaries in milliseconds and KiB; empty if overall validation fails |
| `environment.json` | Commit, source fingerprint, toolchains, protocol and evidence class |
| `fixtures/` | Shared binary inputs; SHA-256 hashes are recorded in results |
| `build/build.json`, `build/build.log` | Build commands, source/binary hashes and compiler output |

Keep formal artifacts under fresh
`target/benchmarks/commit-<sha12>/distance/run-<NNN>/` directories. Preserve raw
fixtures, JSON, logs and metadata outside Git. A report identifies its measured
commit separately from its documentation commit and links any published evidence.
A local artifact path does not provide public access.

For changes to this suite, run the applicable [contribution checks](../../CONTRIBUTING.md#verification),
including the focused controller tests and worker formatting:

```sh
python3 -m unittest discover -s tools -p 'test_*distances.py'
rustfmt --edition 2024 --check benches/distances/cocycle.rs
```
