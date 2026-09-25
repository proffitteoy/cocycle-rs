# Benchmarks

Cross-library measurements execute Cocycle Rust and native C++ references:
GUDHI/Ripser for persistence, Topp for diagram distances. Python's standard
library prepares fixtures, starts executables and validates outputs. Isolated
GUDHI/POT Python workers supply additional distance correctness checks; their
times are excluded from native rankings.

Start with the [reporting rules](reporting.md) to define a comparable experiment,
then select its execution protocol below. Use the [report template](report-template.md)
for retained conclusions. The native timing suites have different preparation,
warmup, export and memory boundaries; their samples must not be pooled.

## Choose a suite

| Question | Tool and contract | Evidence |
| --- | --- | --- |
| How does precomputed F2 H0/H1 compare, including GUDHI edge collapse? | [Native setup](native/README.md), [H0/H1 protocol](protocol.md); `tools/benchmark_native.py` | Time to owned intervals and process memory |
| What do complete public Rips workflows cost? | [Pipeline protocol](pipeline/README.md); `tools/benchmark_rips_pipeline.py` | Validation/construction/expansion/compute/export, end-to-end time and process memory |
| Do bottleneck/W1/W2 agree, and which distance optimizations help? | [Distance protocol](distances/README.md); `tools/compare_distances.py` and `tools/benchmark_distances.py` | Independent matching checks, Rust/Topp timing and memory ablations |
| Do exact construction, fields and higher dimensions agree? | [Exact correctness](../tools/README.md#native-rips-correctness-checks); `tools/compare_rips.py` | Topology and persistence validation, not performance |
| Do sparse sampling, blockers and persistence agree? | [Sparse correctness](../tools/README.md#native-sparse-rips-checks); `tools/compare_sparse_rips.py` | Approximation validation, not performance |
| What does the Rust API cost without external comparisons? | [rips.rs](rips.rs), `cargo bench --locked --bench rips` | Rust-only timings, not a cross-library baseline |

The pipeline covers matrix/graph/point construction, explicit complexes, prime
fields, representatives and approximation. Protocol v2 defaults to 12 measured
rounds with seeded, position-balanced backend order and optional CPU pinning. Native references doing less work are correctness-only;
Ripser approximation is explicitly unsupported. See the protocol for row-level
comparison scopes and the reporting rules for stronger comparative studies.

## Layout

| Location | Responsibility |
| --- | --- |
| [reporting.md](reporting.md) | Shared comparability, sampling, claim and evidence rules |
| [report-template.md](report-template.md) | Template for maintained comparisons with explicit measured revisions |
| [native/](native/README.md) | H0/H1 workers, shared upstream source pins and setup |
| [protocol.md](protocol.md) | `cocycle-native-v1` H0/H1 execution contract |
| [pipeline/](pipeline/README.md) | Complete-workflow workers and their execution contract |
| [distances/](distances/README.md) | `cocycle-distance-v1` workers, reference pins and controlled ablations |
| [reports/](reports/README.md) | Stable comparison pages; measured commits and evidence status in each report |
| `target/` (repository root) | Ignored local run outputs; CI uploads selected results as artifacts |

Fixture and build helpers are shared where their contracts agree; correctness
instrumentation and timing adapters remain separate. In particular, the sparse
correctness sampler is not the sampler used by the timed GUDHI workflow.

## Compared implementations

| Path | Native execution |
| --- | --- |
| Cocycle | Rust public APIs, specialized F2 H1 or generic prime-field computation; requested construction/bases depend on the workflow |
| GUDHI direct | C++ Rips/flag construction, `Simplex_tree` expansion and CAM persistence |
| GUDHI collapse | One native flag edge-collapse call before expansion/CAM; available in the H0/H1 suite |
| GUDHI sparse approximation | Original metric greedy sampler with documented start/accessor instrumentation, sparse construction, blocker expansion and CAM; pipeline only |
| Upstream Ripser | C++ implicit exact dense or sparse-threshold persistence; no equivalent approximate constructor |
| Topp | C++ diagram matching; bottleneck/L-infinity, W1/L-infinity and W2/Euclidean; distance suite only |

Backend IDs are suite-specific and retained in the raw results. GUDHI's integrated
Ripser is not the upstream Ripser backend. The adapters do not cover every engine
or feature of either library. Native execution removes Python wrapper overhead,
but does not remove required native conversions, allocations or copies.

## Evidence lifecycle

Run experiments in a fresh directory under the repository's ignored `target/`.
Validate complete outcomes before using any timing. Native CI comparisons upload
selected outputs as GitHub Actions artifacts with a 14-day retention period;
archives, logs, generated fixtures and raw results never enter source Git.

The [Rips comparison](reports/rips-comparison.md) summarizes current measured
capabilities and selected results. Reruns update that same page; Git history keeps
previous versions. Every update names the actual measured commits, protocols and
evidence status. Local-only evidence may support a clearly labeled summary;
publicly auditable results need external artifact URLs, checksums and retention
information. No durable experiment store is configured yet. See the
[storage policy](reporting.md#storage-and-evidence-lifecycle) and
[report index](reports/README.md).

Benchmarks are not CI speed gates. Smoke checks validate the harness; shared-runner
timings do not establish a local performance baseline. After staging changes, run
`python3 tools/check_artifacts.py` to catch accidental generated-file additions.
