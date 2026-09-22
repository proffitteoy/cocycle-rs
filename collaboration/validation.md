# Validation and reference comparisons

[Collaboration workspace](README.md) / [Phase 1](phase-1.md)

This page adds our collaboration evidence requirements. Existing commands and
ownership remain in [CONTRIBUTING.md](../CONTRIBUTING.md), the
[testing guide](../docs/development/testing.md), [tools guide](../tools/README.md)
and [native setup](../benches/native/README.md). A test that was not run is not a
pass. Missing prerequisites, timeouts, failures and unsupported cases must remain
visible in the PR, with exact scope and reason.

## Required comparison matrix

| Change | Independent expectation | Required external comparison |
| --- | --- | --- |
| Exact VR construction/persistence | Hand calculation, property or independent explicit reducer | GUDHI and Ripser on their shared supported inputs |
| Sparse Rips approximation | Independent sampling/subset checks and blocker regression | GUDHI sparse constructor; document Ripser exclusion and run exact-Rips regressions separately |
| Diagram distances | Hand calculations and tiny exhaustive matching independent of production | GUDHI and Topp with matching metric/essential-point semantics |
| Shared diagram/context conversion | Multiplicity, dimensions, coverage and endpoint cases | Apply both VR and distance rows when both contracts are affected |
| Native comparison adapters or protocol | Controller/protocol tests with real small worker inputs | Relevant native smoke and correctness suite |
| Documentation/templates only | Source, local-link and template consistency checks | No invented algorithm comparison or performance result |

GUDHI is required for mathematical changes. VR changes additionally require
Ripser; distance changes additionally require Topp. A reference's unsupported
case is an explicit exclusion, never a passing comparison. Cover that case with
an independent expectation and document the evidence limit. A missing required
reference for a supported case leaves acceptance pending.

## Source identity and local discovery

These locations were observed on 2026-09-22; they are not portable configuration
or instructions to change another checkout. Recheck revision and dirty state
before using them, and preserve all local work.

| Reference | Local observation |
| --- | --- |
| GUDHI | `F:\GUDHI\gudhi-devel`, HEAD `4ec34ac55e6d2e8cfd7c322e84c1b0a56d516d51`; untracked `.vs/` and `out/` observed |
| Topp | `F:\bottleneck`, HEAD `ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234`, version 1.0.1 |
| Ripser assessment source | `target/topp-assessment/ripser-pinned.cpp`, revision `01add51ff64aaf40889483260cc5c3b7d0f2a1e7` |

The assessment Ripser file's SHA-256 is
`6ef9c828944316974ed62408a55b351cba896c8e548eb7193af733d005585bf0`.
The existing native suite's authoritative GUDHI revision is instead
`cba915e3ab8e1f5b1fe26eb44b407285f7af4e78`, in
[sources.json](../benches/native/sources.json). The local GUDHI checkout is not
that pinned baseline. Do not bypass verification or silently substitute it.

Use fresh, dedicated checkouts under ignored `target/native-sources/`, following
the native setup, with exact revisions and clean source state. Never reset,
checkout over, clean, or rewrite the user's GUDHI/Topp repositories to satisfy
the benchmark builder. A separately justified newer-source experiment must be
labeled separately and does not replace the pinned comparison.

## Existing VR commands

After completing the native setup, run from the repository root. Every output
directory must be new; use a new attempt suffix if a path already exists.
These are existing commands, not proof they have run for the current PR:

```sh
python3 tools/compare_rips.py --output target/rips-reference-run-001
python3 tools/compare_sparse_rips.py --output target/sparse-rips-reference-run-001
python3 tools/benchmark_native.py --quick --samples 1 --output target/native-smoke-run-001
python3 tools/benchmark_rips_pipeline.py --quick --samples 1 --output target/rips-pipeline-smoke-run-001
```

Select suites by changed contract under CONTRIBUTING; sparse changes require
both their sparse suite and exact regressions. Add `--boost-include` only for an
actual header directory. Source options `--gudhi-source` and, for exact Rips,
`--ripser-source` change locations, not the required revisions. Requirements are
Rust 1.91+, Python 3.10+, C++17, Git and Boost headers. Native resource/timing
measurement currently requires Linux; use Linux CI/WSL rather than interpreting
an unsupported Windows resource measurement as a successful run.

Ripser's pinned kernel uses float32. Shared exact comparisons need values
representable under the declared common precision; record any single common
quantization before all backends run. Keep float64-only cases separate.
Coefficient support also differs: pinned Ripser supports primes through 251,
pinned GUDHI through 46337. Larger Rust fields require explicit exclusions and
independent validation. Neither exporter proves representative-vector equality;
check closure, rank and duality independently in Rust.

Ripser cannot reproduce the sparse approximation constructor's higher-simplex
blockers. Feeding only its modified graph to Ripser changes the complex and
cannot establish agreement. The existing sparse GUDHI adapter instruments only
the declared sampling choice; retain that instrumentation and its checks.

## Distance comparison contract

No maintained distance comparison command ships in this initialization. The
first distance implementation must add a runnable bounded harness, with GUDHI,
Topp and independent exhaustive small-diagram expectations, before acceptance.
Do not relabel a prior exploratory Python bridge as that harness.

Specify bottleneck with L-infinity, W1 with L-infinity, or W2 with L2 explicitly.
Record Wasserstein order and internal norm separately. Select the GUDHI backend
and exact settings explicitly (including zero bottleneck approximation where
supported); an approximate API default is not exact agreement. If its backend
uses a Python binding or an external solver, record that version and dependency.
Such correctness evidence does not qualify as a native timing comparison.

Include empty, diagonal, repeated, near-diagonal, unequal-size and extreme-value
diagrams; compatible/incompatible essential counts; invalid endpoints; and
rejection of censored or uncomputed input. Check symmetry, self-distance and
hand-derived diagonal costs. Retain multiplicity and separate homology degrees.
State absolute/relative tolerances with an independent numeric justification;
never enlarge them after a mismatch merely to obtain a pass.

## Rust, tool and documentation checks

For Rust changes, run the full relevant verification commands in CONTRIBUTING:
formatting (including affected standalone workers), Clippy with warnings denied,
debug/release tests, examples, strict rustdoc and Markdown doctests, plus Rust
1.91 MSRV tests/checks. Report unavailable toolchains as unavailable.
Source/documentation checks are required; Python controller changes also require
all `test_*.py` tests and a small exercise of the changed command.
Run `tools/check_artifacts.py` after staging because it checks the Git index.
For documentation-only changes, use source/documentation checks and affected
examples; do not add artificial tests or run large benchmarks.

## Evidence retained with every comparison

Record candidate and baseline full SHAs, dirty state/source fingerprints,
harness revision, upstream revisions and source/header/binary hashes, toolchain
versions, OS/CPU, command, seed and fixture hash, field and dimensions, cutoff,
precision, scale convention, multiplicity/zero-length policy, coverage and
essential/censored handling. Record approximation provenance when applicable.
Retain every backend's raw output, mismatch, timeout, exclusion and exit status.
Separate correctness, resource snapshots and comparable performance evidence
according to the [reporting rules](../benches/reporting.md).

Keep generated inputs, downloaded sources, logs and raw results under ignored
`target/`; never commit them or compressed copies. Commit concise reports in
this collaboration directory and link real CI artifacts or durable external
evidence. Cloud review has no `F:` drive: provide reproducible pinned-source
setup and accessible evidence, not machine-local paths as proof. GitHub artifacts
expire; record retention and preserve required evidence in durable storage before
expiry. Do not invent artifact URLs or describe pending cloud checks as passed.
