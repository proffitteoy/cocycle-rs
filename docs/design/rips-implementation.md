# Rips implementation scope


Module paths below record the Rips implementation stages. The later filtered-cell
boundary moved shared access and representatives into simplicial modules; consult
[current architecture](../development/architecture.md) for current locations and
[filtered complexes](../guides/filtered-complexes.md) for the public contract.

[Documentation](../README.md) / Design

Status: staged implementation map, recorded 2026-09-20. This file makes the
[Rips subsystem design](rips.md) concrete at directory and file level. Stages 1-4 are now implemented. The
[architecture](../development/architecture.md) owns the actual source tree and the
[construction guide](../guides/rips-construction.md) owns current usage.
Stage 5 adds the acceptance audit and workflow resource harness; hosted
release-commit verification remains an external gate. No placeholder directories were created.

## Scope by delivery stage

The complete target remains R1-R10 in the [capability matrix](rips.md#required-capability-matrix).
Capability implementation covers stages 1-4; stage 5 now has local integration
and resource evidence. Hosted validation of the submitted revision remains open.

| Stage | Code scope | Observable deliverable |
| --- | --- | --- |
| 1 | Input layouts, threshold graph construction, supplied graph validation, shared dense/sparse flag access, F2 H0/H1 integration, result context and initial execution controls | Both exact threshold Rips and supplied weighted graphs can be computed without a sparse-to-dense conversion |
| 2 | Explicit simplex storage/expansion, dimension-generic indexing, enumeration and F2 computation | Inspectable Rips complexes and persistence beyond H1, with independent dimension coverage |
| 3 | Prime fields, coefficient columns, representative computation and result payloads | Field selection and requested cycle/cocycle representatives |
| 4 | Greedy permutation, insertion radii, sparse-approximation edges/blockers and compatible persistence | Approximate Rips from input to an interpretable result |
| 5 | Integration review, full validation matrix, measurements, examples and release-readiness checks | Evidence for every R1-R10 item; not another mathematical implementation shortcut |

No Alpha, Cech, cubical, zigzag, mutable Simplex tree replacement, edge collapse,
Python bindings, dataframe integrations or new workspace crates are included.
Existing diagram descriptors remain consumers of the same diagram contract.

## Directories introduced in stage 1

The stage 1 maps and review units retain the original scope of that stage,
including its initial F2-only options and deferred additions. They are an
implementation record, not the current API inventory; later-stage decisions below
and the architecture page describe the completed extensions.

Paths are relative to the repository root. These are implementation directories,
not necessarily public module paths. Facades re-export the selected public types.

| New directory | Responsibility |
| --- | --- |
| `src/complex/` | Public domain for concrete topology storage |
| `src/complex/graph/` | Checked weighted graph and sorted adjacency |
| `src/filtration/rips/` | Exact Rips constructors and original-input provenance; replaces the existing single `rips.rs` file |
| `src/filtration/flag/` | Shared flag filtration access, dense/sparse enumeration, ordering and indices |
| `src/persistence/flag/` | Shared F2 H0/H1 computation and supplied-graph entry point |
| `src/persistence/flag/cohomology/` | Existing cohomology implementation moved and adapted to both access paths |
| `tools/reference/` | Native correctness workers for construction and graph computation; separate from timed benchmark workers |

The existing `geometry/`, `diagram/`, `tests/`, `examples/`, `docs/` and CI
directories gain files or edits. Do not create the later-stage directories in
advance.

## Stage 1 file map

### Production additions

| File | Code owned by this file |
| --- | --- |
| `src/geometry/matrix.rs` | `DissimilarityMatrixView`, explicit lower/upper/square layouts, validation and indexing |
| `src/geometry/distance.rs` | Shared checked scalar/cutoff validation; pair evaluation stays in `euclidean.rs` and borrowed layout access in `matrix.rs` |
| `src/complex/mod.rs` | Domain documentation and public storage exports |
| `src/complex/graph/mod.rs` | `WeightedGraph`, edge validation, vertex count, graph ownership and queries |
| `src/complex/graph/adjacency.rs` | Private sorted adjacency construction and access; no persistence semantics |
| `src/filtration/rips/mod.rs` | Rips construction exports and domain documentation |
| `src/filtration/rips/exact.rs` | Point/matrix/callback threshold constructors, `ThresholdRips`, construction range and original vertex identity |
| `src/filtration/flag/mod.rs` | `FlagFiltration`, mathematical contract and private access exports |
| `src/filtration/flag/access.rs` | Narrow engine access contract shared by actual dense and sparse implementations |
| `src/filtration/flag/dense.rs` | Current implicit edge/triangle access adapted to matrix layouts |
| `src/filtration/flag/sparse.rs` | Edges and common-neighbor triangle cofacets from adjacency without densification |
| `src/filtration/flag/order.rs` | Single production authority for simplex-entry comparison and tie order |
| `src/filtration/flag/index.rs` | Checked edge/triangle IDs and decoding retained for the specialized H1 path |
| `src/persistence/flag/mod.rs` | Supplied-graph entry point and shared private engine dispatch |
| `src/persistence/options.rs` | New computation options, initially admitting only implemented F2 H0/H1 requests |
| `src/persistence/execution.rs` | Per-call work counters, declared operation limits and cooperative cancellation |
| `src/diagram/computation.rs` | `PersistenceResult` and owned exact-Rips/supplied-flag context around the existing diagram |

Custom distance callbacks are parameters to exact construction, not stored global
state or an automatically public trait. Numeric input validation belongs to
geometry; threshold selection and original-filtration coverage belong to Rips.
Adjacency intersections for simplex generation belong to filtration, not graph
storage. The new options/result types contain only implemented capabilities;
later variants are recorded in the stage 2-4 implementation decisions below.

### Existing implementation moves and edits

| Previous file or subtree | Implemented destination/change | Reason |
| --- | --- | --- |
| `src/filtration/rips.rs` | Split into `filtration/flag/{dense,index,order}.rs`; move original-Rips-only range/cone decisions into `filtration/rips/exact.rs` | Flag access is shared; stopping rules and source semantics need their own authority |
| `src/persistence/rips/h0.rs` | Move to `persistence/flag/h0.rs`, adapt edge input | Reuse connectivity reduction without requiring a distance matrix |
| `src/persistence/rips/union_find.rs` | Move to `persistence/flag/union_find.rs` | Both shared H0 and H1 consumers use it |
| `src/persistence/rips/cohomology/` | Move to `persistence/flag/cohomology/`; adapt access, counters and imports | One reduction engine serves dense and sparse exact flag filtrations |
| `src/persistence/rips/mod.rs` | Keep existing entry points; add richer exact-Rips dispatch and provenance assembly | Preserve the public API while connecting the new path |
| `src/persistence/rips/options.rs` | Retain current `RipsOptions` and validate compatibility with the new options | No unsupported dimension/field silently becomes accepted |
| `src/persistence/mod.rs` | Add exports/dispatch and retain shared interval assembly | Coverage determines unpaired endpoint interpretation at the result boundary |
| `src/geometry/euclidean.rs` | Extract reusable checked pairwise distance evaluation; preserve existing dense conversion behavior | Threshold construction can evaluate pairs without allocating all distances |
| `src/geometry/dissimilarity.rs` | Add a no-copy adaptation to new access if needed; preserve public methods and lower-triangle meaning | Existing consumers and raw-buffer semantics remain valid |
| `src/geometry/mod.rs`, `src/diagram/mod.rs` | Export implemented new types | Domain facades own public names |
| `src/filtration/mod.rs`, `src/lib.rs` | Expose construction/storage domains; keep engine access private | Public construction does not expose internal reducer data |
| `src/error.rs` | Add structured graph/layout/request/limit errors | No partial success, silent repair or generic string-only errors |
| `src/persistence/tests.rs`, `src/persistence/reference/` | Update imports/adapters and add independent graph cases as needed | Keep the existing reference compiled only for tests and mathematically independent |

Move mechanical code first and verify old tests, then add access abstractions and
sparse behavior in reviewable steps. Reusing one engine does not justify changing
its pivot order or optimization conditions. A cone bound justified for full dense
Rips must not be applied to an arbitrary sparse graph by filling in unknown edges.
Disable an optimization on a new path until its conditions are established.

### Validation, tooling and documentation

| Files | Stage 1 scope |
| --- | --- |
| `tests/matrix.rs`, `tests/graph.rs`, `tests/flag.rs`, `tests/rips_construction.rs` | New public input, graph, computation and provenance contract tests |
| Existing `tests/{contracts,euclidean,rips}.rs` and private cohomology tests | Compatibility, dense/sparse parity, ordering, failures and optimization regressions |
| `examples/rips_graph.rs` | Point cloud to threshold graph, inspection, persistence and existing descriptors |
| `examples/flag_persistence.rs` | Supplied graph with isolated vertices and explicit complete/truncated semantics |
| `tools/reference/{rips_gudhi.cpp,rips_ripser.cpp,rips_cocycle.rs}` | Native structured outputs for graph/construction and persistence comparisons |
| `tools/compare_rips.py`, `tools/test_compare_rips.py` | Reproducible fixtures, pinned-source compilation, comparison and explicit unsupported-case reporting |
| `tools/build_native.py` | Reuse pinned-source verification; change shared helper extraction only if both controllers need it |
| `.github/workflows/ci.yml` | Run new examples and bounded native correctness comparisons, retaining failure evidence |
| `docs/guides/rips-construction.md` | Usage of implemented construction and supplied-graph computation |
| `docs/reference/mathematics.md`, `docs/development/{architecture,testing}.md` | Implemented semantics, actual source layout and validation evidence |
| `docs/README.md`, `tools/README.md`, `docs/design/{rips,roadmap}.md`, `CHANGELOG.md` | Navigation, commands, achieved/pending capability status and user-visible changes |
| `README.md`, `CONTRIBUTING.md` | Update supported scope and affected check commands when implementation lands |

The native workers do not modify downloaded GUDHI/Ripser checkouts. Reuse source
pins and fixture conventions from the benchmark infrastructure. Keep the existing
`tools/compare_ripser.py` behavior until an explicit migration is documented.
Benchmark worker/controller edits are admitted only when adding measurements for
a new path; preserve measurement semantics and keep generated outputs outside Git. A new sparse timing
comparison must include construction and avoid hiding any conversion cost.

## Later directory and code additions

This table preserves the original staged allocation of responsibilities. These
additions now have implementations; the decisions below record their final form,
and the architecture page owns the current file map. The stage gates in the main
design still govern completion.

| Stage | Additional directories/files | Existing code expanded |
| --- | --- | --- |
| 2 | `src/complex/simplicial/{mod,simplex,incidence}.rs`; `src/filtration/flag/expansion.rs`; `src/filtration/rips/expansion.rs` | Shared flag indices/enumerators become dimension-generic; cohomology gains dimension progression; explicit Rips carries construction-dimension provenance |
| 3 | `src/algebra/field/`, `src/algebra/column/`; `src/persistence/flag/representatives/`; `src/diagram/representative.rs` | Prime-field options, oriented reduction, requested representative outputs and stable interval associations |
| 3 or 4, at first actual need | `src/algebra/reduction/`; an explicit-complex adapter within the consuming persistence module | Independent production reduction for representatives or blocker-aware explicit approximation; test oracle remains separate |
| 4 | `src/filtration/rips/approximation/{mod,options,greedy,edges,blocker}.rs`; `src/persistence/rips/approximation.rs`; `src/geometry/metric.rs` | Approximation provenance, declared/checked hypotheses, blocker-aware computation and native sparse-Rips comparisons |
| 5 | Extend tests, examples, reference workers and relevant benchmark scenarios | Cross-capability acceptance, resource reports, MSRV/platform/package verification |

An algorithm-specific greedy permutation initially stays with its approximation
consumer. Move it into a public sampling domain only when a second concrete use
establishes a reusable contract. Field and column operations must not depend on
Rips vertex storage or source geometry.

## Implemented API adjustments

The concrete implementation preserves the planned responsibility boundaries with
these API refinements: `DissimilarityMatrixView::new(values, n, MatrixLayout)`
selects layout explicitly; ordinary compute calls take options and limits.
`with_field(PrimeField)` selects coefficients, and `_with_representatives` calls
add a request slice. Context records the actual field. Stage 4 adds separate approximation
entry points, with no empty traits. Distance
validation is shared as concrete operations; `FlagAccess` is the private trait
justified by the dense and sparse implementations. Exact context uses identity vertex
mapping without an O(n) vector; approximate context owns the actual sampling map.

## Change boundaries and review units

Stage 1 changes production algorithms as well as data structures: it introduces
sparse edge/cofacet access and changes how the existing reduction obtains input.
It is therefore not a directory-only refactor. No runtime dependencies are planned;
any later dependency proposal must justify its capability, license and MSRV.

| Review unit | Required result |
| --- | --- |
| 1. Ownership and relocation | Shared flag implementation ownership; old public APIs and existing tests still work |
| 2. Inputs and construction | Validated matrix layouts and graphs, streamed threshold construction, precise provenance |
| 3. Computation integration | Exact sparse F2 H0/H1, correct coverage, independent/native parity and no densification |
| 4. User-facing completion | Context, initial limits/cancellation, examples, CI, current documentation and recorded evidence |

All four units are required for stage 1. Approximate construction remains a
required later stage; unsupported requests
must remain errors until their gates pass. There is no line-count target: assess
the change by responsibilities, compatibility, mathematical coverage and evidence.

The following remain stable during stage 1: current interval/coverage meaning,
`PersistenceDiagram` as descriptor input, existing point/condensed-input public
paths, F2 parity and zero-length interval conventions. Benchmark outputs are
external evidence, not source files. Do not add placeholder modules, rewrite descriptor algorithms,
alter package dependencies or publish a release as incidental scope.

## Stage 2 implementation decisions

The explicit domain is `src/complex/simplicial/{mod,simplex,incidence}.rs`.
`filtration/flag/{cliques,expansion}.rs` shares clique access and explicit
construction; `filtration/rips/expansion.rs` retains dimension/scale provenance.
`persistence/rips/expanded.rs` consumes stored incidence with independent coverage
checks. `persistence/flag/cohomology/dimensions.rs` implements dimension progression,
clearing and implicit transformation-column reconstruction for F2.

Higher-dimensional simplex keys are increasing vertex tuples, with decreasing
colex order, rather than an enlarged binomial index table. This avoids rejecting
a sparse complex just because absent simplices would have unrepresentable IDs.
The optimized H1 index/enumerator and pair shortcuts remain in place. Both key
forms use the ordering authority in `complex/simplicial/simplex.rs`. Explicit
incidence supplies a third concrete access implementation; no public backend
registry or general boundary trait was added.

The independent high-dimensional forward boundary oracle and topology contracts
live in `tests/rips_expansion.rs`; `examples/rips_sphere.rs` demonstrates H2.
Native workers compare complete stored simplex/value sets with GUDHI C++ and
interval multisets through H4 with GUDHI and upstream Ripser C++. These are
correctness checks, not evidence of performance parity. Construction still has
no cooperative execution limits. Representative APIs were added in stage 3
and approximation APIs in stage 4 below.

## Stage 3 implementation decisions

`algebra/field` exposes `PrimeField`; `algebra/column` shares private ordered
coefficient columns across implicit cohomology, forward reduction and dual solves.
`algebra/reduction` contains the production forward boundary reducer required by
representatives. It has no geometry, filtration or diagram dependencies and does
not reuse the test oracle. Generic cohomology now preserves oriented coefficients;
the compact F2 H1 path retains its parity specialization and pair shortcuts.

`persistence/flag/representatives/{mod,basis,request,complex,dual}.rs` owns request
validation, opt-in skeleton work, persistent cycle selection and query-scale dual
cocycles. All exact input paths expose `_with_representatives` variants; ordinary
signatures remain unchanged. `diagram/representative.rs` owns original simplex
vertices, coefficients, request position and local interval identity. Repeated
intervals remain distinct. Nonempty requests activate extra storage and execution
work; empty requests retain ordinary implicit computation.

`tests/prime_fields.rs` independently checks modular ranks and representative
bases, including field-sensitive flag RP2 and the largest supported u32 prime.
`examples/rips_representatives.rs` and the [guide](../guides/rips-representatives.md)
show a finite F3 cycle and its dual. Native workers compare prime-field diagrams,
with explicit pinned-upstream field/precision limits. No native representative
vector comparison or high-dimensional performance equivalence is claimed.

## Stage 4 implementation decisions

`filtration/rips/approximation/` owns `options`, `greedy`, `edges`, `blocker`,
`access` and `expansion`. Sampling uses deterministic exhaustive farthest-point
selection; no metric pruning is applied to unchecked distances. The private
`SimplicialAccess` now supports original vertex labels with compact connectivity
positions. Exact paths keep identity mapping. Shared expansion and oriented
cohomology consume the blocker-aware access; representatives use that same topology.

`geometry/metric.rs` owns metric policy, evidence and exhaustive exact-stored-f64
triangle checks. `diagram/approximation.rs` owns sampling provenance and conditional
ideal-arithmetic bounds. `persistence/rips/approximation.rs` owns scale/dimension
checks and implicit/explicit entry points. Frozen sparse expansions are a distinct
type so they cannot acquire exact-Rips provenance accidentally.

The [sparse guide](../guides/sparse-rips.md), runnable example and public regression
tests cover current contracts. A real metric clique rejected by the blocker guards
against ordinary flag expansion. The native adapter changes only GUDHI's sampling
call to align deterministic start/tie choices, checks unmodified upstream metric
sampling where choices are unique, and leaves edges/blockers/reduction unchanged.
Header and binary hashes record this instrumentation. Stage 5 covers broader
resource and performance acceptance; construction/expansion remain uncontrolled.


## Stage 5 implementation decisions

`benches/pipeline/{cocycle.rs,common.hpp,gudhi.cpp,ripser.cpp}` measures real public
workflows. `tools/benchmark_rips_pipeline.py` owns fixtures, process limits,
source-pinned builds, exact-output checks and sample aggregation. Its protocol
regressions live in `tools/test_benchmark_rips_pipeline.py`. The correctness-only
adapters stay separate and are not timed as if their oracle work were part of a
normal computation. The old H0/H1 benchmark protocol is unchanged.

`tests/rips_resources.rs` checks per-call failure recovery across twelve richer
paths and independent concurrent field computations. Sparse callback caching now
shares the geometry layer's checked condensed pair count, with wide-integer
boundary validation. No global workspace or public instrumentation hook was added.
The [acceptance audit](rips-acceptance.md) records R1-R10 evidence and resource
limits; hosted CI for the eventual submission remains distinct from local results.
