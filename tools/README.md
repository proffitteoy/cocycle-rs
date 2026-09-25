# Developer tools

These tools are outside the Rust library and crate package. See
[contribution rules](../CONTRIBUTING.md) and [benchmark navigation](../benches/README.md).

## Native cross-library benchmarks

Use [benchmark_native.py](benchmark_native.py) for Cocycle, GUDHI C++ and upstream
Ripser C++. Python's standard library orchestrates separate native executables;
no Python TDA binding is imported into the measured computation. Source setup,
commands and artifacts are documented in the [native guide](../benches/native/README.md),
with execution rules in the [H0/H1 protocol](../benches/protocol.md). The separate
[Rips pipeline suite](#rips-workflow-resources) measures broader public workflows.
Both follow the shared [reporting rules](../benches/reporting.md).

[build_native.py](build_native.py) verifies pinned sources and compiles workers.
[benchmark_inputs.py](benchmark_inputs.py) owns deterministic fixture generation,
shared with historical controllers without sharing their Python worker path.

## Native Rips correctness checks

[compare_rips.py](compare_rips.py) compiles the workers under `tools/reference/`
and compares exact threshold edges and prime-field interval multisets through H4. GUDHI
also checks complete expanded simplex/value sets against Rust and independent
subset enumeration; Ripser contributes implicit persistence, not explicit complex
export. Dense Rust fixtures compare matrix, sparse and explicit paths. All Rust
fixtures also check that representative-enabled computation preserves the diagram.
Inputs cover
matrices, supplied graphs, isolates, zero edges, ties, cutoffs and random nonmetric
weights. Rust dense/threshold paths are also compared with each other. No Python
TDA wrapper is used.

```sh
python3 tools/compare_rips.py --output target/rips-reference
```

Supply `--boost-include target/native-sources/boost/usr/include` for the local Boost
setup. Output must be new. This shares [source pins](../benches/native/sources.json)
and verification/output instrumentation helpers with native benchmarks, but is a
correctness suite, not timing evidence. Artifacts include fixtures, source and
consumed-header hashes, binary hashes, build logs, outputs and a completion summary.
Failed comparisons leave their outputs in `results.json` and return nonzero.
Ripser's tiny-input adapter exclusions and float64-only fixtures are explicit.
Ripser is built with `USE_COEFFICIENTS`; its pinned 8-bit coefficient storage
supports primes through 251. GUDHI's pinned `Field_Zp` caps primes at 46337.
Larger-field cases record per-reference exclusions and are not counted as native
agreement. Rust still validates those fixtures against their structural contracts.
Field-sensitive fixtures include the flag subdivision of RP2. Representative
closure/basis checks live in Rust tests, not in these C++ output adapters.

The text header is `mode n max_homology_dimension cutoff count characteristic`;
readers accept older five-field headers as F2. Field values are never narrowed
silently. Actual executable protocol checks reject invalid/composite/overflowing
moduli, verify five-field compatibility and exercise reference limits before
comparing mathematical fixtures. Outputs and fixture records include the actual
characteristic; `environment.json` retains the protocol-check outcomes.
Its raw unpaired endpoints are compared along with separately checked Rust coverage.

## Documentation and tool verification

```sh
python3 tools/check_source.py
python3 tools/check_artifacts.py
python3 tools/check_docs.py
python3 -m unittest discover -s tools -p 'test_*.py'
rustfmt --edition 2024 --check tools/diagram_dump.rs tools/benchmark_driver.rs benches/native/cocycle.rs tools/reference/rips_cocycle.rs tools/reference/sparse_cocycle.rs benches/pipeline/cocycle.rs
```

Run the artifact check after staging. It rejects generated paths and log/archive/
binary files in the Git index, including historical or forcibly added outputs.

Source checks cover maintained text encoding/whitespace and parse Python without
importing tools. They exclude raw evidence, downloaded sources, and build output.
Documentation checks recurse through `docs/`, benchmark reports, and native setup
pages while excluding raw results. They validate inline and reference local links
and headings, flag duplicate/missing explicit reference labels, and scan for CJK
text. External URLs and English prose quality need manual review. Supported
Markdown conventions are in the [code conventions](../docs/development/conventions.md#tests-and-documentation).
Tool unit tests need only the Python standard
library. Native compilation and diagram comparisons require the C++ prerequisites
listed in the native guide. No tests assert machine-dependent speed thresholds.

## Algorithm diagnostics

[profile_rips.py](profile_rips.py) invokes private H1 optimization stages and
checks each against the independent explicit reducer. It varies single/two-pass
initialization independently of virtual apparent-pair reconstruction; the
[testing guide](../docs/development/testing.md#independent-invariants-and-properties)
lists the stages. Use it to inspect algorithm work, not to rank public API
performance.

[profile_scaling.py](profile_scaling.py) reads the historical scaling artifact
schema and requires matching source hashes and validated diagrams. It is not an
adapter for `cocycle-native-v1` results. Its counts are entries and operations,
not allocated bytes. Detailed commands and counter definitions are retained in
[diagnostic instructions](legacy-benchmarks.md#private-h1-optimization-profiling).

## Optional wrapper tools

The [legacy tools guide](legacy-benchmarks.md) documents `compare_ripser.py`,
`benchmark_gudhi.py`, `benchmark_scaling.py`, their pinned Python environments,
and optional correctness checks. Generate fresh inputs under `target/`; historical
outputs are not shipped. Current native tools include `benchmark_native.py`, `compare_rips.py`,
`compare_sparse_rips.py` and `benchmark_rips_pipeline.py`, with separate
correctness and timing responsibilities. Do not mix old wrapper samples into
native rankings.

## Native sparse Rips checks

```sh
python3 tools/compare_sparse_rips.py --output target/sparse-rips-reference
```

Uses the same pinned GUDHI source checkout and optional `--boost-include` argument
as exact comparisons. The output directory must be new. No Python TDA package is
used: Python builds processes, writes metric fixtures and checks their outputs.

The adapter generates a copy of the pinned sparse header under the output build
directory, replacing exactly one constructor sampling call. A separate exhaustive
C++ sampler fixes the start and smallest-ID tie policy without reading Rust
outputs. On unique positive-radius choices it also checks the unmodified upstream
metric sampler. GUDHI sparse edges, higher-simplex blockers and persistent
cohomology remain unchanged. For explicit dimension zero the adapter prunes the
initial graph to vertices to match Cocycle's documented expansion contract.

The suite compares full simplex/value sets and interval multisets over shared
fields, as well as deterministic sampling and an independent subset enumeration.
It retains fixture, source, dependency/header, compiler and binary hashes, plus
instrumentation details. `environment.json`, `results.json`, `summary.json` and
`build/build.log` are correctness artifacts, not timing/memory measurements.
Upstream Ripser is explicitly excluded because it lacks this sparse approximation
constructor; feeding only the modified graph to its flag engine would omit the
higher-simplex blockers. Continue running the exact native suite separately.

## Rips workflow resources

`benchmark_rips_pipeline.py` measures validation/conversion, construction,
expansion, public computation and export separately, plus end-to-end time and
per-process peak RSS. See the [pipeline protocol](../benches/pipeline/README.md)
for the full scopes, native comparability limits, failure retention and commands.
The existing H0/H1 timing protocol remains separate. Generated results and logs
are stored outside Git under the shared reporting policy.
