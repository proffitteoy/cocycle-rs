# Topp interoperability and Ripser implementation assessment

[Documentation](../README.md) / Research

Historical investigation: the external-bridge recommendation below predates the
collaboration's Rust-first decision. Follow the active [phase-one plan](../phase-1.md)
for implementation scope; retain this report as source and experiment evidence.

Observed on **2026-09-22**. This is a source-grounded collaboration assessment,
not a release decision, a new performance study, or a maintained integration.
Cocycle computes persistence diagrams; Topp measures distances between diagrams.
Their mathematical roles are complementary. A small external bridge is feasible
for complete results, provided it preserves the contracts below. The local
prototype below verifies that composition on actual native outputs.

## Source and collaboration boundaries

| Component | Assessed revision or state |
| --- | --- |
| Cocycle local main and upstream main | [`0cfc7b5cf280644aa60a0f92a96a6bb0f04c42bf`][cocycle-main] |
| Local `origin` | `git@github.com:proffitteoy/cocycle-rs.git`; fork main matched upstream |
| Current upstream | [Aequiludium/cocycle-rs](https://github.com/Aequiludium/cocycle-rs) |
| Historical measured Rust kernel | `6dbfd4298217158831c8df48a4caefec9730b5e9` |
| Topp local and remote HEAD | [`ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234`][topp-source], version 1.0.1 |
| Ripser source baseline | [`01add51ff64aaf40889483260cc5c3b7d0f2a1e7`][ripser-source] |

Some manifest and documentation links still name `huangbogeng/cocycle-rs`; the
observed upstream now uses `Aequiludium`. Branch names and open-PR states are dated
observations; commit links identify the assessed code. Remotes remain unchanged.

Comparing `src/`, `Cargo.toml` and `Cargo.lock` between assessed main and the
historical measured kernel produced no changes. This establishes source identity
for that scope, not a fresh timing measurement or identity of every tool file.

[Main CI run 35504704575][main-ci] completed successfully with seven jobs:
quality checks, native C++ reference comparisons, Rust tests on Windows/Linux/
macOS, package verification, and minimum-Rust verification. The job definitions
are in the [assessed workflow][ci-source]. These results belong to assessed main.

[PR #4](https://github.com/Aequiludium/cocycle-rs/pull/4) was open and unmerged
at head [`818ff7e8775299c3028cb18fe415438b4f1afac6`][builder-source]. Its
[CI run 35585001615][builder-ci] completed successfully. It proposes unified
`RipsBuilder`/`PersistenceExt` workflows and whole-operation `Execution` budgets.
Its compatibility contract retains legacy free functions, option types and
import paths. Do not describe its builder API as already present on assessed
main, or attribute main's historical performance numbers to the PR.

## Implemented scope and maturity

Cocycle is a single pre-release Rust crate, version 0.1.0, using edition 2024
and Rust 1.91 or later. The [manifest](../../Cargo.toml) has no runtime
dependencies and forbids unsafe code. The [README](../../README.md) identifies
Git-based installation and says it is not yet published on crates.io.

| Area | Implemented behavior and boundary |
| --- | --- |
| Inputs | Borrowed Euclidean points and lower/upper/square dissimilarity matrices; explicit validation |
| Construction | Exact threshold graphs, supplied weighted graphs, and frozen simplicial complexes with incidence queries |
| Persistence | Ordinary persistence through requested dimensions over prime `u32` fields; optimized F2 H0/H1 |
| Approximation | Deterministic sampling, modified edges, higher-simplex blockers and metric/provenance metadata |
| Results | Owned diagrams retain dimensions and coverage; richer `PersistenceResult` also retains field and construction context |
| Representatives | Opt-in persistent cycle bases and query-scale dual cocycles, associated with interval instances |
| Descriptors | Finite lifetimes, persistence entropy and Betti curves |
| Execution | Cooperative persistence cancellation/work budgets; no hard RSS bound or successful partial result |

The [architecture](../../docs/development/architecture.md) separates geometry, complexes,
filtrations, persistence engines, owned diagrams and descriptors. Geometry and
filtration do not depend on persistence; descriptors do not need reducer state.
The independent explicit reference reducer is test-only, not a production backend.
Legacy `RipsOptions` remains F2 H0/H1-only; dimension-generic and field selection
use `PersistenceOptions` and the richer entry points. On assessed main, standalone
construction is outside the persistence budget. Sparse retained storage does not
eliminate all-pairs distance work; explicit skeleton size and reduction fill-in
can dominate. The builder PR extends budget scope but does not provide a hard RSS cap.

This is a developed Rips implementation with multiple validation layers, not a
complete replacement for GUDHI's wider TDA toolkit. Diagram distances, public
Python bindings, and a maintained Cocycle-to-Topp adapter are absent. A frozen
complex is not a mutable `Simplex_tree`. Other filtration families and general
application workflows must not be inferred from the general-purpose direction.

API evolution, combinatorial growth and local-only historical evidence remain
considerations; the [roadmap](../../docs/design/roadmap.md) and
[acceptance audit](../../docs/design/rips-acceptance.md) distinguish priorities from release gates.

## Topp compatibility contract

Topp 1.0.1 exposes a stable Python 1.x API over a C++20 kernel; its
[release interface description][topp-readme] excludes C++ headers/ABI from that promise.
Python requires NumPy; validated wheels target Windows x64 and Linux x86_64.
This does not establish a portable Rust/C++ ABI or equivalent SIMD performance.

Topp accepts arrays convertible to `(n, 2)` float64 and computes exact bottleneck,
W1 with the internal infinity norm, and W2 with the internal Euclidean norm.
Other Wasserstein combinations are unsupported. Exact excludes algorithmic
approximation, not floating-point error; see [mathematics][topp-mathematics] and [inputs][topp-input].

| Cocycle information | Required bridge behavior |
| --- | --- |
| `Finite(death)` | Emit `(birth, death)` without rounding or deduplication |
| `Essential` | Emit `(birth, +inf)`; preserve the number of essential instances |
| `RightCensored { through }` | Reject for a full-diagram distance; neither `through` nor `+inf` is an equivalent death |
| Computed dimension | Export each requested dimension separately; reject an uncomputed dimension |
| Empty computed dimension | Produce shape `(0, 2)`, not an uncomputed-dimension success |
| Multiplicity | Retain every interval instance; array order need not carry meaning |
| Field and filtration context | Preserve outside the array and check comparison compatibility explicitly |
| Coverage and approximation | Preserve outside the array; require the intended filtration to be fully observed |

These requirements follow from [interval endpoints](../../src/diagram/interval.rs)
and [diagram validation](../../src/diagram/persistence_diagram.rs). In particular,
`Coverage::Complete` describes coverage of a filtration, not a complete graph in
the graph-theoretic sense. Supplied flag filtrations can have absent edges and
essential higher-dimensional classes even with complete coverage.

Finite intervals use `[birth, death)`; right-censoring records survival through an
inclusive cutoff. Replacing a censored endpoint by that cutoff invents a death;
replacing it by infinity invents essentiality. Rejecting every `Through` result
is a conservative full-diagram policy, including an empty observed result.
Comparing only observed finite subdiagrams is a different, explicitly named
analysis and does not recover the unknown complete-diagram distance.

Topp ignores finite diagonal points and preserves non-diagonal multiplicity.
Positive essential points match only the same type; unequal counts give infinity.
The bridge must not mix dimensions or silently remove essential H0 intervals.

Identical array shapes do not establish comparable topology. Match coefficient
fields, scale units and intended filtration semantics. A complete sparse
approximation is complete for its approximate filtration, not proof of equality
to exact Rips. Retain epsilon, metric policy, retained vertices and bound target;
do not present an exact Topp distance between approximate diagrams as an exact
distance between the corresponding exact-Rips diagrams.

Both MIT licenses permit an appropriately attributed integration. Cocycle's
[contributing rules](../../CONTRIBUTING.md) place bindings outside the crate, and
[conventions](../../docs/development/conventions.md) exclude C++ from core dependencies.
An external process/data bridge to stable Python is the smallest initial boundary.
A separate FFI crate would need ABI, ownership, error and distribution design;
it is not an existing supported option.

## How the persistence engines differ from Ripser

Both implementations use union-find for H0 and implicit coboundary reduction for
diagram-only higher homology. Both use clearing and regenerate coboundaries from
stored transformation information. Cocycle is not merely explicit boundary-matrix
reduction, nor is it a binding to Ripser. Its F2 implementation documents an
independent implementation from mathematical invariants.

[Dispatch](../../src/persistence/flag/mod.rs) selects the compact path for H0/H1
over F2. The [reducer](../../src/persistence/flag/cohomology/mod.rs) classifies
edges in forward order with union-find, skips H0 death edges, then reduces cycle
edges in reverse order. Combinatorial `usize` IDs identify edges and triangles;
`HashMap` records pivot owners, while `BinaryHeap` supports parity cancellation.
Stored transformation columns contain edge positions and sparse additions.

The production path already enables apparent and emergent pair shortcuts. It
searches the original edge column before additions, examines the first equal-value
cofacet, and accepts an unoccupied pivot. Apparent pairing additionally tests the
latest facet. A successful shortcut still records the pivot owner and a transform
column. It is inaccurate to say Cocycle has no apparent-pair optimization.

[Generic reduction](../../src/persistence/flag/cohomology/dimensions.rs) is selected
for higher requested dimensions or odd-prime coefficients. It enumerates a current
simplex level, uses vertex tuples as pivot keys, and retains sparse transformation
columns. Its [ordered columns](../../src/algebra/column/mod.rs) use `BTreeMap<K,u32>`;
oriented cofacet coefficients require modular multiplication and inversion.
Clearing carries pivot simplices to the next dimension while retaining topology
for enumeration. It does not contain the specialized H1 pair-shortcut branch.
This is still implicit cohomology, not eager storage of every reduced coboundary.

The [pinned Ripser implementation][ripser-source] uses combinatorial `int64_t`
indices, a binomial-coefficient table, priority queues, pivot hash maps, and a
compressed sparse transformation matrix. Optional coefficient support is selected
at compilation; it does not switch to Cocycle's tuple-keyed ordered-column design.
The pinned coefficient packing limits the current native adapter to primes at
most 251; Cocycle's validated field type supports prime `u32` characteristics.
The new composition experiment below exercises only F2 and F3.

Ripser filters zero apparent pairs while assembling columns, including after H0
and across higher dimensions. Its reduction loop can eliminate a pivot through
an apparent facet without a stored ordinary owner. Its emergent shortcut also
checks that the candidate lacks a zero apparent facet. Cocycle's original-column
H1 shortcut has a narrower placement and different bookkeeping; these conditions
must be compared with their surrounding invariants, not transplanted separately.

See pinned [apparent detection][ripser-apparent], [column filtering][ripser-columns],
and [emergent/reduction handling][ripser-reduce]. These suggest profiling targets;
they do not prove which operation explains an observed timing gap.

Ripser's pinned default `value_t` is `float`; Cocycle stores scales as `f64`.
The native comparisons use shared float32-exact inputs where exact filtration
parity is required, with explicit exclusions for incompatible reference cases.
Different rounding can change ties and filtration order. Topp's float64 inputs
cannot recover precision already lost when a diagram was computed in float32.

Cocycle [representative requests](../../docs/guides/rips-representatives.md) materialize
the required skeleton and retain forward boundary transformations; dual cocycles
add scale-specific solves. This costs more than ordinary diagram computation.
The pinned upstream Ripser executable/adapters do not supply the same owned
cycle-and-dual-basis contract. This observation does not describe every Ripser
fork or Python wrapper. Cross-library equality of representative vectors is not
a valid acceptance criterion when bases and tie choices differ.

Ripser's sparse distance input computes the supplied sparse flag filtration.
It is distinct from Cocycle's [sparse Rips approximation](../../docs/guides/sparse-rips.md),
which changes edges and applies higher-simplex blockers using sampling metadata.
The pinned Ripser has no corresponding approximation constructor; GUDHI is the
matching native reference. Sparsity alone does not establish equivalent complexes.

## Historical performance evidence

The maintained [Rips comparison](../../benches/reports/rips-comparison.md) owns these
historical medians in milliseconds: Linux, AMD EPYC 7H12, rustc 1.92.0, g++ 11.4.0,
12 measured processes per cell. No performance benchmark was run for this report.

| Historical workload | Cocycle | Ripser | Timing scope |
| --- | ---: | ---: | --- |
| Uniform F2 H0, n=1024 | 37.081 | 84.424 | Prepared-input native computation |
| Uniform F2 H1, n=128 | 3.832 | 2.665 | Prepared-input H0 through H1 |
| Nonmetric F3 H2, n=24 | 8.996 | 1.009 | Public workflow through H2 |

H0 was faster on the selected row; H1 and generic F3/H2 trailed Ripser there.
Prepared-input and public-workflow timings include different work and must not
be pooled. The source identity check above supports relevance to assessed main,
but does not generalize the ranking to other datasets, fields, hardware or PR #4.
Original raw evidence is explicitly local-only, without a public artifact URL or
guaranteed retention. The report summary and fingerprints do not replace those
raw files. No new speedup, universal ranking or release-readiness claim follows.

## Verification performed for this assessment

Source inspection checked dispatch, reduction, result semantics and Topp's
documented input behavior. A local Topp 1.0.1 smoke used hand-derived Cocycle-format
records and passed 19 checks, including multiplicity, essential intervals, empty
diagrams, supported distances, batching and rejection of censored/uncomputed data.
That smoke did not execute the Cocycle Rust kernel or establish a shipped adapter.

The initially absent Rust toolchain was installed in an isolated ignored
`target/topp-assessment/` directory, without changing the system PATH. Rust 1.91.0
on WSL Ubuntu compiled the assessed source. Debug and release each passed 97
ordinary tests and five doctests; two diagnostic profiling tests were ignored.
This local result is Linux evidence; the hosted OS matrix above is separate.

A fresh native-to-Topp correctness experiment then used the existing
[Cocycle worker](../../tools/reference/rips_cocycle.rs) and
[Ripser worker](../../tools/reference/rips_ripser.cpp). Ripser's downloaded source
matched the repository's pinned SHA256; the existing output instrumentation
captures numeric intervals without changing reduction. Every fixture ran in a
fresh process. Topp 1.0.1 was the installed Windows wheel, with Python 3.12.14 and
NumPy 2.5.2; its Python implementation files matched the local Topp source.
The wheel was not rebuilt or certified byte-for-byte against that Git revision.

| Fresh check | Observed result and scope |
| --- | --- |
| Native interval multisets | 62 Cocycle/Ripser comparisons passed, including multiplicities |
| Complete-result composition | 60 native results grouped into 30 original/scaled pairs and 86 per-dimension pairs |
| Topp distances | 258 agreements: bottleneck, W1-infinity and W2-Euclidean for the two backends' diagrams |
| Independent distance expectations | 24 hand-derived distance checks passed for square H0/H1, octahedral H2 and essential flag H1 |
| Incomplete coverage | Two cutoff-square results were explicitly rejected before Topp conversion |

Fixtures use F2/F3, H0 through H1/H2, square and octahedral metrics, a supplied
four-cycle flag filtration, and deterministic random dissimilarities on 4-7
vertices (seed 20260922). Original/scaled pairs use factors 1 and 1.25. All
filtration values are dyadic and exactly representable in both f32 and f64.
The existing Rust worker also checks dense/threshold, implicit/explicit and
representative-enabled diagram agreement on its applicable paths. Representative
vectors were not compared with Ripser.

The 258 distance agreements are consumption/adapter evidence: identical upstream
diagrams imply identical Topp inputs, so they are not an independent oracle for
Topp's matching algorithm. The hand-derived expectations supply the independent
checks. These small cases do not establish large-data scalability, all prime
fields, sparse-approximation composition, or production binding support.

Local evidence is under ignored `target/topp-assessment/`: `semantic_smoke.py`,
`semantic-results.json`, `run-rust-checks.sh`, `rust-debug.log`, `rust-release.log`,
`end_to_end.py`, `native-results.json`, `end-to-end-results.json`, the generated
fixtures and binary/source fingerprints. `remote-state.json` records the checked
main/PR identities. Evidence remains local-only and is not packaged or committed.
The smoke script is an experimental bridge; its development protocol is not a
new stable public API. The experiment can be rerun using the following sequence
after rebuilding the existing Rust worker as in `run-rust-checks.sh`:

```text
Linux/WSL: python3 target/topp-assessment/end_to_end.py native
Windows, with Topp 1.0.1: python target/topp-assessment/end_to_end.py topp
```

On Windows, source and documentation checks passed; 58 Python tool tests produced
53 passes, three skips, one failure and one error. The error directly invoked a
[Linux-only worker](../../tools/benchmark_rips_pipeline.py) importing `resource`;
the normal pipeline entry point already requires Linux. The failure was a
[documentation-test assertion](../../tools/test_check_docs.py) expecting `/` where
Windows emitted `\`; the checker correctly reported the missing target.
These are tool-test portability limitations, not observed Rust-kernel failures.
The Linux tools run passed all 58 tests. Hosted Windows Rust success is consistent:
the [workflow][ci-source] runs Python tools on Ubuntu and Rust tests across OSes.

## Recommended collaboration sequence

1. Agree on a versioned result contract: per-dimension intervals, endpoint kinds,
   computed dimensions, coverage, field, scale units and approximation provenance.
   Keep it outside reducer internals and do not treat a development dump as stable.
2. Build the smallest external bridge to Topp's documented Python API. Validate
   real Cocycle outputs against hand-derived cases and independent native diagram
   oracles, then independently validate distances. Preserve refusal cases and
   repeated/essential intervals; a successful self-distance alone is insufficient.
3. Fix the two Windows tool-test portability issues in a separate focused change,
   preserving the pipeline's explicit Linux execution contract.
4. Profile H1 cofacet regeneration and generic tuple/column work, then measure
   individual changes with fixed precision and equivalent outputs. Retain hard
   cases, regressions and independently reproducible evidence.

This proposes work, not an implemented adapter, ABI commitment or deployment.
Reconcile the API revision with PR #4; keep performance changes separate.

[cocycle-main]: https://github.com/Aequiludium/cocycle-rs/tree/0cfc7b5cf280644aa60a0f92a96a6bb0f04c42bf
[main-ci]: https://github.com/Aequiludium/cocycle-rs/actions/runs/35504704575
[ci-source]: https://github.com/Aequiludium/cocycle-rs/blob/0cfc7b5cf280644aa60a0f92a96a6bb0f04c42bf/.github/workflows/ci.yml
[builder-source]: https://github.com/Aequiludium/cocycle-rs/tree/818ff7e8775299c3028cb18fe415438b4f1afac6
[builder-ci]: https://github.com/Aequiludium/cocycle-rs/actions/runs/35585001615
[topp-source]: https://github.com/proffitteoy/Topp/tree/ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234
[topp-readme]: https://github.com/proffitteoy/Topp/blob/ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234/README.en.md
[topp-mathematics]: https://github.com/proffitteoy/Topp/blob/ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234/docs/en/MATHEMATICS.md
[topp-input]: https://github.com/proffitteoy/Topp/blob/ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234/docs/en/guide/input-semantics.md
[ripser-source]: https://github.com/Ripser/ripser/blob/01add51ff64aaf40889483260cc5c3b7d0f2a1e7/ripser.cpp
[ripser-apparent]: https://github.com/Ripser/ripser/blob/01add51ff64aaf40889483260cc5c3b7d0f2a1e7/ripser.cpp#L513-L552
[ripser-columns]: https://github.com/Ripser/ripser/blob/01add51ff64aaf40889483260cc5c3b7d0f2a1e7/ripser.cpp#L554-L635
[ripser-reduce]: https://github.com/Ripser/ripser/blob/01add51ff64aaf40889483260cc5c3b7d0f2a1e7/ripser.cpp#L670-L799
