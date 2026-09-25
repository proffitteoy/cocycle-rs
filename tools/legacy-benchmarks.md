# Optional Python-wrapper and diagnostic tools

These development tools are outside the Rust library and crate package. Current
cross-library performance reports use [native C++ workers](../benches/README.md).
No historical datasets, logs or measured reports are shipped with this repository.
Generate fresh inputs under ignored `target/`; do not commit their outputs.

## Independent reference comparison

`compare_ripser.py` compares Cocycle with Ripser.py on identical distance matrices.
It compiles the crate and a Rust adapter in a temporary directory. With `uv`, Rust
and Python 3.12 available, run from the repository root:

```sh
uv venv --python 3.12 /tmp/cocycle-reference-env
uv pip install --python /tmp/cocycle-reference-env/bin/python -r tools/requirements-reference.txt
/tmp/cocycle-reference-env/bin/python tools/compare_ripser.py
```

This optional F2 H0/H1 differential check supplements independent Rust properties.
It does not measure the upstream C++ API. Quarter-integer inputs make the shared
values exact in float32 and float64; coverage determines essential versus censored
endpoints. Python is not required for `cargo test`.

## Wrapper experiments

`benchmark_gudhi.py` and `benchmark_scaling.py` remain optional tools for exercising
Python binding interfaces. Their numbers must not enter native rankings because
conversion and wrapper costs are included. Install their pinned environment:

```sh
uv venv --python 3.12 /tmp/cocycle-benchmark-env
uv pip install --python /tmp/cocycle-benchmark-env/bin/python -r tools/requirements-benchmark.txt
/tmp/cocycle-benchmark-env/bin/python tools/benchmark_gudhi.py --include-ripser --samples 1 --output target/wrapper-reference
/tmp/cocycle-benchmark-env/bin/python tools/benchmark_scaling.py --quick --samples 1 --output target/wrapper-scaling
```

Output directories must be new. With `--include-ripser`, shared fixtures are
quantized once to float32-exact values; Cocycle/GUDHI retain f64 kernels. Without
it, the GUDHI-only path retains f64 inputs. Unsupported cases, errors and timeouts
are distinct outcomes. Whole-worker timeouts include startup, warmup and all
samples; address-space caps are not RSS budgets. Inspect the recorded protocol
and environment before interpreting any result.

## Private H1 optimization profiling

`profile_rips.py` builds release library tests and runs the private H1 stages
listed in the [testing guide](../docs/development/testing.md#independent-invariants-and-properties).
They include independent single/two-pass initialization and stored/virtual
apparent-owner configurations. Each stage uses a fresh process, one warmup and
five measured samples, with reference validation after timing.
Use newly generated wrapper fixtures from the command above:

```sh
python3 tools/profile_rips.py \
  target/wrapper-reference/fixtures/uniform_h1_128.bin \
  target/wrapper-reference/fixtures/circle_h1_128.bin \
  target/wrapper-reference/fixtures/equal_h1_128.bin \
  --output target/h1-ablation.json
```

These instrumented timings describe algorithm work, not public API performance.
Use a CPU allowed on the host if setting `--cpu`; do not copy machine-specific
CPU choices from another experiment.

## Scaling work counters

`profile_scaling.py` consumes the wrapper scaling schema, not native worker
artifacts. It requires matching source hashes and validated diagrams:

```sh
python3 tools/profile_scaling.py --benchmark target/wrapper-scaling \
  --cases uniform_h1_16 bipartite_h1_32_cutoff --output target/h1-work.json
```

Counters record accepted cofacets, column additions, retained transformation
entries and heap entries. Accepted cofacets exclude rejected candidate vertices;
heap entries include duplicates awaiting parity cancellation. Entry counts are
not allocated bytes, and a zero cofacet count does not mean zero enumeration work.
The independent reference remains separate from production computation.

Record full commands, fingerprints and outcomes in external run artifacts when
using diagnostics to support a performance decision. Follow the
[storage policy](../benches/reporting.md#storage-and-evidence-lifecycle).
