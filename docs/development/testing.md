# Testing and validation

[Documentation](../README.md) / Development

Tests protect mathematical and public API contracts. Run commands and review
requirements are in [CONTRIBUTING.md](../../CONTRIBUTING.md#verification); measurement
protocols belong in the [benchmark guide](../../benches/README.md).

## Test layers

| Location | Purpose |
| --- | --- |
| `tests/contracts.rs` | Input shapes, values, options, interval ownership and coverage |
| `tests/euclidean.rs` | Distance numerics, invariance, and point/distance parity |
| `tests/rips.rs` | Hand calculations, independent ranks, stability, truncation, bipartite multiplicity |
| `tests/matrix.rs`, `tests/graph.rs` | Layout parity, graph invariants, borrowing and invalid inputs |
| `tests/rips_construction.rs`, `tests/flag.rs` | Construction coverage, callbacks, sparse results, limits/cancellation and isolated vertices |
| `tests/rips_expansion.rs` | Oriented incidence, skeleton sufficiency, H2/H3/H4 spheres, independent high-dimensional boundary oracle and sparse resource regression |
| `tests/prime_fields.rs` | Full-u32 modular arithmetic, field-sensitive flag RP2, independent ranks, cycle/cocycle closure, nontriviality, duality and interval identity |
| `tests/rips_resources.rs` | Budget/cancellation recovery across public paths, every-work-budget F2/H1 retry, and concurrent read-only F2/H1 and prime-field calls |
| `tests/sparse_rips.rs` | Metric hypotheses, sampling provenance, blocker topology, original IDs, approximate coverage and computation parity |
| `tests/descriptors.rs` | Formula, endpoint, exclusion, overflow, and empty-result behavior |
| `tests/diagram_distances.rs` | Uninstrumented public distances, independent partial matching, essential multiplicity, coverage, context and numerical regressions |
| `src/diagram_distances/bottleneck/tests.rs`, `src/diagram_distances/wasserstein/tests.rs` | Kernel oracles, adaptive routes, forced alternatives and diagnostic counters |
| `src/persistence/reference/` | Independent explicit filtration and boundary reducer |
| `src/filtration/flag/dense.rs` tests | Indexing, overflow, and independent cofacet enumeration |
| `src/persistence/flag/cohomology/tests.rs` | Independent optimization combinations, transformation replay, duality, cancellation checkpoints and difficult numeric cases |
| `tools/test_*.py` | Source/documentation checks, external comparison, and benchmark protocol behavior |
| `tools/test_compare_distances.py`, `tools/test_benchmark_distances.py` | Distance transport, independent rational oracle, failure retention, profiling hooks and selection gates |

The two ignored profiling tests are deliberately invoked only by developer tools.
They do not represent missing ordinary regression coverage. Instrumented timings
are not production benchmarks.

When moving private modules, update the exact test names in
`tools/profile_rips.py` and `tools/profile_scaling.py`. List discovered tests and
run small diagnostic smoke cases: a successful process that matched zero tests
does not verify a diagnostic path. Keep historical experiment paths tied to the
source revision they measured.

## Hand-derived cases

Ordinary homology over F2, zero-lifetime intervals omitted. E denotes essential;
C(T) denotes alive at the cutoff, not a finite death.

| Case | Input | H0 | H1 |
| --- | --- | --- | --- |
| V01 | Empty input | Empty | Empty |
| V02 | One vertex | (0,E) | Empty |
| V03 | Two vertices, distance 2 | [0,2), (0,E) | Empty |
| V04 | Duplicate pair, distance 0 | (0,E) | Empty; zero pair omitted |
| V05 | Three vertices, all distances 1 | Two [0,1), (0,E) | Empty |
| V06 | Unit square | Three [0,1), (0,E) | [1,sqrt(2)) |
| V07 | Unit square, T=1 | Three [0,1), (0,C(1)) | (1,C(1)) |
| V08 | Unit square, T=0.5 | Four (0,C(0.5)) | Empty |
| V09 | Hand filtration: vertices 0, edges 1, face 2 | Two [0,1), (0,E) | [1,2) |
| V10 | Four equidistant vertices, 2-skeleton | Three [0,1), (0,E) | Empty; do not export artificial H2 |

V09 is a hand-built filtered complex, not a different cutoff on a three-point
Rips input. It verifies reduction independently of Rips construction. At T=1,
the square's H1 must count in a Betti query; queries above T must fail.
Complete bipartite fixtures additionally use the analytic multiplicities in
[mathematics section 10](../reference/mathematics.md#10-analytic-complete-bipartite-filtration).

## Independent invariants and properties

- Faces exist before cofaces, filtration values are monotone, and boundary squared
  is zero. Reduced nonempty columns have distinct pivots.
- Independent dense F2 elimination checks Betti numbers from boundary ranks.
  It must not call the persistence reducer to construct expected values.
- H0 union-find matches reference boundary reduction. Unpaired births are decided
  after reduction, not when an empty column is first encountered.
- Complete and truncated diagrams agree on Betti numbers within known coverage.
  Death beyond T must not be relabeled as death at T.
- Vertex permutations preserve diagram multisets. Translation and orthogonal
  transforms preserve Euclidean results; positive rescaling scales all endpoints.
- Small exhaustive matchings, including diagonal matches, check the fixed-vertex
  perturbation bound on complete diagrams.
- Input overflow, invalid numbers, unsupported dimensions, and query errors must
  remain distinguishable from valid empty results. Computed empty dimensions and
  uncomputed dimensions have different semantics.
- Descriptor tests check entropy ln(2) for two equal lifetimes, zero for one,
  `None` for none, and explicit exclusion of essential/censored intervals.

The cohomology tests compare explicit cohomology, clearing, implicit
reconstruction, cone stopping and apparent/emergent shortcuts. Two additional
independent axes compare single-pass versus two-pass initialization and stored
apparent owners versus virtual zero-apparent reconstruction. The combinations
cover all 729 four-vertex distance assignments from {0,1,2}, random f64/nonmetric
inputs, ties, adjacent floats, subnormals, huge scales and cutoff endpoints.
Reversed-transpose matrix tests check pair and unpaired-index mapping independently
of production simplex indexing.

Dense and sparse tests independently replay each stored transformation by XORing
its original edge coboundaries and compare the result with the reduced column,
checking $R=CV$ as well as diagram equality. Directed cases exercise an occupied
first equal-valued cofacet, alternating heap capacities, parity cancellation and
repeated virtual-owner use. F2/H1 resource tests interrupt at every work budget
below completion and retry the same read-only input; private tests also cancel
at each cofacet checkpoint. Concurrent calls verify per-call state isolation.

The diagnostic stages in `tools/profile_rips.py` are `explicit`, `clearing`,
`implicit`, `cone`, `apparent`, `two-pass`, `emergent`, `virtual-two-pass` and
`virtual`. The historical `emergent` stage includes both apparent and emergent
shortcuts; its `two-pass` counterpart changes only initialization. The two virtual
stages add apparent-pair omission. Each stage checks the independent explicit
oracle; its instrumented timings do not rank production performance.

## External comparison

The [native comparison](../../benches/native/README.md) runs GUDHI C++ and upstream
Ripser C++ workers against the Rust API. It checks every measured diagram,
including multiplicities, dimensions, coverage, and endpoint kinds, using exact
shared filtration values. Hand-derived cases and analytic bipartite counts
provide independent expectations. Infinity from another library is interpreted
using the requested range; it is not automatically essential.

Shared precomputed values isolate persistence from distance construction. When a
reference uses lower precision, quantize the same input for every backend instead
of hiding differences behind larger tolerances. Independent point-cloud norm
comparisons use an explicit tolerance. The H0/H1 native protocol measures only
precomputed distances. The separate [pipeline suite](../../benches/pipeline/README.md)
also measures Rust point construction, but its precomputed native references are
correctness-only for those rows. The [historical tools](../../tools/legacy-benchmarks.md)
retain the optional 512-case Ripser.py correctness check and wrapper benchmarks;
their timings are not the native performance baseline.

Diagram distances have a separate [native protocol](../../benches/distances/README.md).
Independent partial-injection enumeration checks finite, diagonal and essential
matching costs; public tests exercise the ordinary uninstrumented crate, while
kernel tests exercise private counters and forced routes. The external suite
uses pinned Topp, native GUDHI bottleneck with the merged matching fix, Hera at
zero relative error, and isolated GUDHI/POT W1/W2 workers with explicit norms.
It preserves f64 endpoint bits and records numerical stress disagreements without
widening tolerances. Missing references, timeouts and POT nonconvergence remain
failures or unavailable evidence, never successful comparisons.

## CI and evidence

CI is configured to run source hygiene/Python syntax checks, all tool unit tests,
formatting, Clippy, rustdoc, local documentation checks,
debug/release tests on Linux/macOS/Windows, Rust 1.91 tests/checks, package validation,
and external comparison smoke checks. Full performance runs are manual, with no
machine-dependent speed gates. Actual hosted results are available in
[GitHub Actions](https://github.com/huangbogeng/cocycle-rs/actions/workflows/ci.yml);
check the exact commit rather than inferring success from the workflow definition.

The diagram-distance CI job runs the quick supported comparison and a small
resource smoke. Full supported acceptance uses `tools/compare_distances.py`
without `--quick`, with the pinned native Topp/fixed-GUDHI and isolated weighted
GUDHI/POT references. Record the exact tested SHA and retained summary; a quick
CI pass does not establish that the full suite ran.

Performance evidence is indexed by measured revision and associated PR under
[benchmarks](../../benches/README.md); execution dates are metadata.
The [reporting rules](../../benches/reporting.md) distinguish correctness checks,
resource snapshots and comparative studies, with separate execution protocols.
Its source hashes identify the measured implementation. A directory refactor
changes those hashes even when behavior is preserved; old measurements must not
be relabeled as a new run. The artifact storage policy replaces historical
repository-local verification records. All generated results and logs stay
outside Git; the CI artifact check enforces the tracked-file boundary.

## Exact graph and matrix paths

The private graph oracle enumerates all vertex triples using an independent edge
map and ordinary boundary reduction. Exhaustive absent/zero/unit edge choices on
four vertices plus an isolate compare both H0 and H1 through multiple cutoffs.
The dense path retains all seven independent optimization configurations. Public
regressions distinguish original Rips censoring from a supplied graph's essential
classes, and a large isolated-vertex case rejects an accidental all-pairs scan
using a small work budget.

`tools/compare_rips.py` compares constructed edge sets/values and interval
multisets using pinned GUDHI and upstream Ripser C++ workers. Dense fixtures also
compare the matrix, threshold-graph and explicit Rust paths. GUDHI additionally
checks full simplex/value sets through the constructed dimension against Rust
and independent subset enumeration. Higher-dimensional fixtures include
H2/H3/H4 spheres and deterministic random weighted graphs. Separate f64 fixtures use
GUDHI only; Ripser float32 precision and tiny-input exclusions are recorded, not
counted as successful comparisons. The suite writes fixtures, source/binary/header
hashes and results. See [tools](../../tools/README.md#native-rips-correctness-checks).

## Prime fields and representative bases

Prime validation covers composites and pseudoprimes as well as the largest u32
prime. Independent test arithmetic uses extended Euclid and u128 products;
production uses modular exponentiation and u64 products. Odd-prime random graph
checks compare implicit diagrams with the production representative path and
independent dense boundary ranks at every fixture scale.

The barycentric subdivision of a six-vertex real projective plane is a 31-vertex
flag graph. F2 has one essential H1 and one essential H2 class; tested odd primes
have neither. This fixture detects an accidental F2-only implementation hidden
behind field options. Its topology is checked by independent enumeration and ranks.

Representative checks establish closed chains/cochains, ranks modulo boundaries
or coboundaries, identity evaluation pairing between active bases, original vertex
IDs, canonical coefficients, repeated interval identity and finite cycles becoming
boundaries at their associated death. Queries test tied entry scales, death
exclusion, inclusive censoring, empty selections, invalid dimensions/scales,
cancellation and work limits. Representative-enabled diagrams must match ordinary
implicit results; equality of diagrams alone is not sufficient evidence for bases.

Native workers compile Ripser with `USE_COEFFICIENTS` and compare matching fields.
The pinned default coefficient bit width supports primes through 251, while
GUDHI's pinned `Field_Zp` supports primes through 46337. Cases beyond these bounds
are explicitly excluded per reference; large-u32 Rust validation is not counted
as cross-library agreement. The adapters compare intervals, not representative
vectors. Record precision and field exclusions alongside successful comparisons. The
native controller also runs malformed-modulus, legacy-header and field-limit
checks against all three real executables; parse failure must never select F2.

## Sparse Rips approximation

`tests/sparse_rips.rs` checks deterministic permutation/radii, zero duplicates,
positive-radius omissions, original-label mappings, conditional bound targets,
strict triangle inequalities (including an upward-rounded equality trap), empty
inputs, scale/dimension sufficiency and numerical/execution failures. A fixed
Manhattan-metric fixture contains the graph clique `[1,2,3]` at value 178 but its
smallest insertion radius is 20, below `178/8`; the triangle must be absent.
Independent subset enumeration verifies all allowed simplices. Generic implicit,
explicit and representative computations agree through H4 over several primes.

`tools/compare_sparse_rips.py` runs Rust and pinned GUDHI C++ workers on deterministic
metric fixtures. It compares complete simplex/value sets, greedy orders/radii,
and prime-field interval multisets. A reference-only exhaustive sampler fixes
start/tie semantics; unmodified GUDHI metric sampling is checked separately on
unique positive-radius choices. GUDHI edge modification, blocker expansion and
cohomology are not replaced. Dimension zero is pruned to vertices in the adapter,
because upstream first inserts the graph regardless of requested dimension.
These differences and source/header/binary hashes are recorded in each artifact.

This is correctness evidence, not an uninstrumented runtime comparison. Ripser
has no matching sparse approximation constructor and is explicitly excluded from
this suite. Exact Rips still has independent GUDHI/Ripser C++ comparisons.


## Workflow resource evidence

The [pipeline protocol](../../benches/pipeline/README.md) measures the implemented
public paths in fresh processes, including representation preparation and result
export. Native references execute C++ directly. Timed runs compare every diagram
before accepting measurements, retain failures/timeouts and distinguish missing
native approximation/representative capabilities from successful comparisons.
These measurements complement cooperative-limit regressions; they do not prove a
hard library memory ceiling or a complexity bound from finite samples.
