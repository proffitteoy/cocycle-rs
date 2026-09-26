# Complete Rips subsystem

[Documentation](../README.md) / Design

The [API design](rips-api.md) supersedes this document's original entry-point
sketches. It defines named explicit-complex construction and direct-persistence
workflows; simplex inspection requires expansion. These workflows are implemented
with legacy compatibility. Current usage remains in the guides and rustdoc.

Status: subsystem target recorded 2026-09-20, with exact dense/sparse and
explicit dimension-generic prime-field paths and requested representative bases
implemented. Sparse approximation also has blocker-aware construction and computation. The
[current architecture](../development/architecture.md), [usage guide](../guides/rips.md)
and [construction guide](../guides/rips-construction.md) remain authoritative for
supported behavior. The API sketches below describe the complete target and are
not all callable today. This document owns the Rips target, historical API
sketches, dependency boundaries, and acceptance gates; the [roadmap](roadmap.md)
owns project priorities.

The [implementation scope](rips-implementation.md) lists stage-specific directory
additions, file moves, code edits, tooling and review units. It refines the target
layout below: exact Rips and supplied graphs share flag access and computation,
while Rips-specific construction and provenance remain separate.

## Objective and completion boundary

Build a native Rust Rips subsystem whose inputs, construction, persistence, and
results work together. Combine GUDHI's inspectable construction capabilities with
Ripser's specialized computation strategies and Cocycle's ownership and error
contracts. Neither C++ library becomes a runtime backend.

Completion includes dense and sparse exact computation, explicit construction,
higher homology dimensions, prime fields, requested representatives, and the
GUDHI-style sparse approximation with its own valid computation path. A collection
of constructors feeding only the existing dense H0/H1 engine is insufficient.
Delivery is incremental, but intermediate milestones must not be relabeled as
completion of this target.

The target is ordinary, one-parameter Rips persistence of finite inputs. Arbitrary
Simplex tree editing, Alpha/Cech, cubical and zigzag persistence, weighted-Rips
variants with different definitions, edge collapse, GPU execution, bindings, and
file-format compatibility are separate capabilities. The in-memory API must be
usable without an application runtime, CLI, or file conversion. "Higher dimensions"
means a dimension-generic algorithm with checked index and resource limits, not a
promise to finish every finite input.

## Three-project baseline

Compare the pinned sources in [sources.json](../../benches/native/sources.json):
GUDHI `cba915e3ab8e1f5b1fe26eb44b407285f7af4e78` and upstream Ripser
`01add51ff64aaf40889483260cc5c3b7d0f2a1e7`. GUDHI's integrated Ripser is a
separate implementation from the upstream executable. Source observations below
are not new benchmark results or a claim that every upstream path was audited.
The Cocycle column records the starting point before stages 1-4, not the current
feature set; use the capability matrix below and the acceptance audit for current
status.

| Concern | GUDHI C++ | Upstream Ripser C++ | Cocycle starting baseline and design consequence |
| --- | --- | --- | --- |
| Inputs | Rips accepts points plus a distance function or matrix access | Parsers for triangular/full matrices, point clouds, and sparse triplets | Borrowed Euclidean points and lower-triangle dissimilarities only; add explicit layouts and custom distances |
| Exact construction | Threshold graph followed by expansion into a receiving complex | Graph/distance storage serves an implicit computation | Add inspectable construction while retaining direct computation |
| Sparse storage | Graph and complex operations; integrated Ripser is another path | Sorted neighbor lists and common-neighbor cofacet enumeration | Replace full-vertex candidate scans on the sparse path, not the dense path universally |
| Approximation | `Sparse_rips_complex` changes edges and applies higher-simplex blockers | Mainline has no corresponding epsilon approximation constructor | Approximation needs its own filtration access and provenance |
| Dimensions and fields | General filtered-complex engines, prime-field and multi-field paths | Dimension-generic Rips; prime fields enabled by a build option | Current H0/H1 over F2 must grow into dimension-generic F2 and prime-field paths |
| Representatives | Selected matrix configurations support cycles | Representative output is described in separate experimental branches | Design explicit requests and verification; do not claim ordinary upstream CLI parity |
| Numerical interface | Filtration value types depend on instantiation | Pinned `value_t` is `float` | Keep f64 contracts; declare precision in each differential experiment |
| Engineering | Header-based capability modules and operation-specific concepts | Specialized implementation concentrated in one C++ source | Separate domains and private working state without recreating either physical layout |

Evidence: [GUDHI source study](../research/gudhi-cpp.md),
[exact Rips header][g-exact], [sparse Rips header][g-sparse],
[Ripser source][r-source], and [Ripser README][r-readme].

Useful implementation details from Ripser include combinatorial simplex IDs,
dimension-by-dimension cohomology, clearing, apparent/emergent pairs, and sorted
neighbor intersections. Its finite-threshold point-cloud path can build adjacency
without first materializing the dense distance matrix, but still checks point
pairs. Its initial coboundary search retains generated entries for the fallback
path. Reuse these ideas with explicit invariants and measurements, not a presumed
speedup. The [H1 optimization sequence](roadmap.md#h1-implementation-sequence)
remains separately reviewable.

## Required capability matrix

Every target below needs implementation, contract tests, a usable example or
documented composition, and the applicable independent comparison. Status reflects
stages 1-4, with integration/resource evidence linked in the
[acceptance audit](rips-acceptance.md). Hosted release-commit CI remains separate.

| ID | Target | Current status | Required completion evidence |
| --- | --- | --- | --- |
| R1 | Borrowed lower/upper condensed and full square matrices | Implemented | Layout equivalence, shape/symmetry/diagonal checks, no forced layout conversion |
| R2 | Euclidean points and custom symmetric pairwise functions | Implemented exact constructors | Stream threshold construction, numeric checks, documented callback contract |
| R3 | Exact threshold graph and supplied weighted graph | Implemented prime-field path | Isolated vertices, missing-edge semantics, sparse H0/H1 parity without densification |
| R4 | Explicit Rips expansion and frozen structure access | Implemented frozen complexes and incidence | Arbitrary requested simplex dimension, closure, boundary, lookup and filtration traversal |
| R5 | Dense/sparse higher-dimensional persistence | Implemented dimension-generic computation | F2 results above H1, killing cofacets, checked indexing and independent expectations |
| R6 | Prime-field computation | Implemented prime-u32 fields | Oriented coefficients, modular arithmetic, field-sensitive topology tests |
| R7 | Optional cycle and cocycle representatives | Implemented scale-specific bases | Valid nontrivial representatives at a declared scale, input mapping and interval association |
| R8 | Sparse Rips construction and persistence | Implemented blocker-aware path | Greedy ordering, modified edges, blockers, applicable hypotheses and native differential checks |
| R9 | Owned interpretable results | Field, filtration identity, representatives and approximation metadata implemented | Add field, filtration identity, approximation context, dimensions and optional representative ownership |
| R10 | Resource behavior and reproducibility | Cooperative controls, failure recovery, ownership audit and workflow measurements implemented | Declared allocations, cooperative limits, repeatability, failure behavior and benchmark evidence |

Full target completion covers R1-R10. Representative requests are optional for a
caller, not optional work that can disappear from the completion checklist.

## Mathematical and input contracts

### Exact inputs and scales

Vertices are born at zero. Edge values are finite nonnegative f64 values and a
simplex enters at the maximum of its edges; cutoffs are inclusive. The empty
simplex is not exposed. Exact constructions retain distinct indices for duplicate
points and zero-distance pairs. Nonmetric symmetric dissimilarities are valid
for exact flag/Rips computation; validating a matrix is not a metric certificate.

The proposed matrix view supports three explicit row-major layouts. Condensed
layouts omit the diagonal; square inputs require zero diagonal and exact numeric
symmetry after signed-zero canonicalization. No tolerance-based repair, averaging,
implicit squaring, or deduplication occurs. `None` denotes no cutoff, not a NaN or
infinite sentinel. All layout arithmetic is checked before indexing/allocation.

A custom distance operation receives original indices or borrowed points and
returns `Result<f64>`. Its contract requires a stable symmetric function with
zero self-distance. Threshold construction samples each unordered pair once,
validates every returned value even if outside the cutoff, and never samples the
diagonal. This establishes the sampled symmetric input, not that an arbitrary
callback is symmetric on unsampled calls. Repeated-query algorithms require the
same stable-function contract and document their query costs. Callback errors
abort construction without a successful partial result.

### Three distinct graph meanings

| Object | Meaning of an absent edge | Meaning of complete coverage |
| --- | --- | --- |
| Exact threshold Rips | Not present through the construction cutoff; later value need not be retained | Complete original Rips only if construction certifies all pairs are covered; otherwise `Through(cutoff)` |
| Supplied weighted flag filtration | Absent at every scale of this supplied filtration | Complete for this graph's clique filtration, not for an unknown point cloud |
| Sparse Rips approximation | Governed by approximation rules, including more than the edge list | Complete for the constructed approximation only when scale coverage is complete; never exact original Rips by implication |

The graph constructor takes an explicit vertex count, retains isolated vertices,
normalizes endpoint order, rejects self-loops and repeated undirected edges, and
validates indices and values. A compact internal adjacency can remap retained
approximation vertices, but an explicit mapping back to original indices survives.

There is no public boolean for upgrading a supplied graph to certified threshold
Rips. Only a construction that checked the source establishes that provenance.
Extracting a raw graph is possible for inspection; treating it as a standalone
flag filtration is an explicit change of mathematical object.

### Dimensions, order and boundaries

Use `max_simplex_dimension` for construction and `max_homology_dimension` for
computation. Expanding to zero returns only vertices. GUDHI's pinned constructor
inserts its graph before expansion, so its dimension-zero behavior is not copied
silently; compare this case with the intended zero-skeleton explicitly.

Computing original Rips through Hq requires access to potential (q+1)-simplices.
A q-skeleton alone does not suffice: its top-dimensional essential classes may
die in the original filtration. An expanded-Rips adapter must reject insufficient
dimension coverage unless absence of the required higher simplices is certified.
Computing a skeleton as a separate mathematical complex must be explicitly named
and must not inherit the original Rips completeness claim.

Increasing filtration value, increasing dimension at equal value, then decreasing
colexicographic simplex order define the proposed deterministic total order. This
agrees with the current within-dimension combinatorial order. All faces precede
their cofaces. Optimizations must establish their assumptions for this order;
external implementations may use different tie refinements without changing the
interval multiset. Representatives need not match across tie conventions.

Simplex vertices are stored or exposed in increasing original-index order.
Deleting vertex i contributes boundary sign `(-1)^i`. Check boundary squared is
zero over F2 and odd primes. Keep storage handles, filtration positions and
combinatorial IDs distinct. Index overflow is an error, never a wrapped identity.

### Sparse Rips approximation

Track epsilon, starting vertex, greedy tie rule, insertion radii, retained original
vertices, minimum insertion-radius selection and maximum filtration scale. The
default start is deterministic (vertex zero for nonempty input); equal farthest
distances select the smallest original index. Zero insertion radii and a positive
minimum radius can remove vertices; expose the selected subset and do not claim
it is the unchanged input. Empty input has no starting vertex.

Follow the pinned GUDHI edge-length convention and its high-dimensional blocker.
An ordinary flag expansion of just the approximation graph is not a substitute.
The minimum-scale argument selects points by insertion radius; it is not permission
to delete low-valued faces and keep their cofaces. Store this choice separately
from upper-scale truncation.

Support finite positive epsilon. For epsilon at least one, explicitly report no
approximation guarantee. For epsilon below one, the
[GUDHI manual][g-manual] reports a `(1, 1/(1-epsilon))` interleaving for the
applicable metric construction. This is an upstream theorem reference, not a
certification of arbitrary floating-point matrices or additional subsampling.
Before R8 ships, record the exact theorem, hypotheses, scale convention and
effect of minimum-radius selection in the mathematical reference. Numerical tests
support an implementation; they do not establish the theorem.

The proposed API records whether metric hypotheses are caller-assumed or checked
under a documented numeric policy. Never label finite/nonnegative validation as
metric validation. Restrict or explicitly downgrade guarantees when hypotheses
are absent. Certified exact-real predicates and rigorous interval arithmetic are
not implicit promises of the f64 implementation.

Approximation computation uses a blocker-aware access path or an explicit valid
complex and independent production reduction. Every pairing shortcut needs a
proof for that path before reuse. Feeding the edge list into ordinary Ripser is
not an oracle for the full approximation.

## Architecture and reuse

This is a target layout, not a request to create empty files. Public types are
re-exported from their owning domains; private implementation files can evolve.

```text
src/
  geometry/                  validated views, pairwise distances and layouts
  complex/
    graph/                   weighted graph storage and sorted adjacency
    simplicial/              frozen simplices, incidence and lookup
  filtration/
    rips/                    exact/approximate construction and Rips provenance
    flag/                    shared dense/sparse flag access and expansion
  algebra/
    field/                   F2 and validated prime-field arithmetic
    column/                  concrete reusable coefficient-column operations
    reduction/               production explicit reduction when a consumer needs it
  persistence/
    rips/                    Rips entry points and approximation dispatch
    flag/                    shared dimension progression and private workspaces
    reference/               independent test-only oracle
  diagram/                   intervals, diagrams and owned result context
  descriptors/               diagram-only analysis operations
```

| Owner | Can depend on | Must not own or depend on |
| --- | --- | --- |
| Geometry | Error and numeric contracts | Complex storage, reducers or diagrams |
| Complex | Error and storage contracts | Rips construction policy, pivot tables or reducer state |
| Filtration | Geometry and complex access | Persistence computation or diagram assembly |
| Algebra | Field/column/boundary contracts | Concrete Rips types or geometric inputs |
| Persistence | Filtration, actual algebra components and diagrams | Mutation of borrowed input to store pivots |
| Diagram | Result validation and owned context | Borrowed engine state or concrete complex storage |
| Descriptors | Diagrams and declared result semantics | Geometry or reduction |

Store provenance alongside a constructed filtration, not inside the generic
adjacency container. Frozen explicit storage can include filtration values;
separating responsibilities does not require allocating the same topology twice.

Distance access, adjacency intersection, ordering, and checked indexing are actual
reuse candidates. Dense and sparse cofacet enumerators remain separate algorithms.
Introduce internal traits only when concrete consumers establish the contract;
public trait ecosystems, universal backend registries, and one all-purpose
`Complex` trait are not prerequisites.

F2 retains specialized parity operations. Prime fields support prime u32 moduli,
with validation, correctly sized intermediate arithmetic, inverses and oriented
coefficients. Generalized cohomology owns dimension transitions and clearing.
If a production explicit reducer is added for R8 or representatives, retain an
independent oracle instead of moving the test implementation into production and
comparing it to itself.

## Public API draft

Names below express ownership and mathematical distinctions for the complete
target. Some are implemented; the construction guide and rustdoc define their
actual signatures. In particular, matrix layout is selected with `new`, current
ordinary compute calls accept options and limits; opt-in `_with_representatives`
calls add a request slice. Exact expansion is available; approximation is not.

| Proposed name and owner | Contract |
| --- | --- |
| `geometry::DissimilarityMatrixView` | Borrow a validated lower, upper or square buffer; preserve the existing `DissimilarityView` API |
| `complex::WeightedGraph` | Owned checked edges and adjacency with an explicit vertex count; no claim about unseen distances |
| `filtration::FlagFiltration` | A supplied graph's full clique filtration with zero vertex births |
| `filtration::ThresholdRips` | Owned threshold graph plus source count and certified construction range |
| `filtration::SparseRips` | Approximation graph plus insertion radii, blocker rules, vertex mapping and parameter context |
| `filtration::SparseRipsOptions` | Validated epsilon, optional minimum insertion radius, optional maximum scale and starting vertex; deterministic ties |
| `geometry::MetricPolicy` | Explicit caller assertion or checked metric hypotheses under a documented numeric policy; never inferred from matrix shape |
| `complex::SimplicialComplex` | Frozen explicit simplices and filtration values, lookup, boundaries, faces/cofaces and traversal |
| `filtration::RipsExpansion` | Explicit complex plus origin, scale coverage and constructed dimension; storage accessor borrows |
| `persistence::PersistenceOptions` | Named homology dimension, validated field and optional inclusive scale cutoff; separates resource controls from mathematics |
| `persistence::RepresentativeRequest` | Requested cycle/cocycle kinds at explicit query scales |
| `persistence::ExecutionLimits` | Optional cooperative work limits and cancellation, with precisely documented accounting |
| `diagram::PersistenceResult` | Owned ordinary-persistence diagram, mathematical context and optional owned representatives; supports Rips and supplied flag filtrations |

Prefer operation names that state inputs and outputs. The following are interface
sketches in text, deliberately not Rust examples that pretend to compile:

```text
DissimilarityMatrixView::lower_triangle(values, vertex_count) -> Result<View>
DissimilarityMatrixView::upper_triangle(values, vertex_count) -> Result<View>
DissimilarityMatrixView::square(values, vertex_count) -> Result<View>

threshold_rips_from_points(points, max_edge) -> Result<ThresholdRips>
threshold_rips_from_distances(matrix_view, max_edge) -> Result<ThresholdRips>
threshold_rips_with_distance(points, max_edge, distance_fn) -> Result<ThresholdRips>
sparse_rips_from_points(points, approximation_options) -> Result<SparseRips>
sparse_rips_from_distances(matrix_view, approximation_options, metric_policy)
    -> Result<SparseRips>

FlagFiltration::new(weighted_graph) -> FlagFiltration
FlagFiltration::expand(max_simplex_dimension) -> Result<SimplicialComplex>
ThresholdRips::expand(max_simplex_dimension) -> Result<RipsExpansion>
SparseRips::expand(max_simplex_dimension) -> Result<RipsExpansion>

compute_rips_from_points(points, options, requests, limits) -> Result<PersistenceResult>
compute_rips_from_distances(matrix_view, options, requests, limits) -> Result<PersistenceResult>
compute_threshold_rips(&threshold_rips, options, requests, limits) -> Result<PersistenceResult>
compute_flag(&flag_filtration, options, requests, limits) -> Result<PersistenceResult>
compute_sparse_rips(&sparse_rips, options, requests, limits) -> Result<PersistenceResult>
compute_expanded_rips(&expansion, options, requests, limits) -> Result<PersistenceResult>
```

`View` abbreviates `DissimilarityMatrixView` in these sketches. Dimension and
cutoff defaults must be explicit in the real constructors. The richer computation
options also carry the requested inclusive scale cutoff; they cannot request a
range beyond a constructed threshold/approximation's known coverage. For supplied
graphs, a cutoff restricts their own otherwise complete flag filtration.

`PersistenceResult` distinguishes supplied flag filtration from original exact
Rips in its context. It does not imply support for extended or zigzag persistence.
Convenience functions use default requests/limits, so ordinary
diagram-only calls do not require configuring telemetry or representatives.
Sparse construction entry points accept distances or points plus validated
approximation options and the explicit metric-assumption policy described above.
Euclidean point construction records its distance definition without claiming a
formal exact-real certificate for rounded pairwise values. Custom distances use
the same explicit hypothesis policy as matrix inputs.

Construction owns retained graph and mapping data, releasing source borrows when
it returns. A point-cloud threshold constructor uses O(n+m) retained graph space
without an intermediate O(n^2) distance buffer; it may still perform O(n^2 d)
distance work for n points in d dimensions. Dense computation borrows a matrix
view and does not require graph materialization. Persistent workspaces are private
and per computation. Explicit expansion is an opt-in allocation, potentially
exponential in size. Sparse approximation preprocessing documents its distance
query/storage choices separately; its name is not an unconditional memory bound.

Result context records filtration kind, scale convention, field, requested and
computed dimensions, source vertex mapping, scale coverage, and approximation
parameters/hypotheses. Algorithm telemetry is separate. An owned context does not
have to copy all source coordinates or adjacency; reproducing the input requires
the caller's dataset or an explicitly saved fixture.

Representatives are requested at finite scales within the known range, evaluated
after every simplex at that scale has entered. Store kind, dimension, field,
scale, interval identity (including repeated intervals), original simplex vertices
and coefficients. Cycles must be closed and non-boundaries; cocycles must be
closed and non-coboundaries, with the requested persistent-class association.
Neither is promised shortest or canonical across algorithms. Verify basis/rank
properties as well as closure. Mapping retained approximation vertices back to
input indices is not a chain map into the original Rips at the same scale; a
transport claim needs its own construction and verification.

### Compatibility and migration

Keep `geometry::DissimilarityView`, `persistence::RipsOptions`, existing
`rips_from_points`/`rips_from_dissimilarities`, and their diagram results working.
Do not change the meaning of `DissimilarityView::values()` from condensed storage
to an arbitrary layout. New matrix access adapts old views without a buffer copy.
New richer operations can eventually back old convenience functions, provided
coverage, interval multiplicity, numeric semantics and error behavior remain
compatible. Any intentional prepublication break requires a changelog entry and
usage migration, not just a private file move.

Retain `PersistenceDiagram` as a descriptor input. New context wraps it rather
than forcing every descriptor to know an engine or graph. Do not extend the old
dimension option before generalized computation and its acceptance tests exist.

## Resource and failure contract

Use checked size/binomial arithmetic and fallible reservations where possible.
Errors distinguish invalid input, unsupported requests, numerical overflow,
allocation failure, work-limit exhaustion and cancellation. A failure returns no
apparently successful partial complex/diagram. Caller cutoffs are successful
mathematical restrictions; cancellation is not another cutoff.

Begin enforceable controls with generated simplex/cofacet counts and cooperative
cancellation checkpoints. Specify exactly which candidates and repeated work are
counted, and check before exceeding the stated operation limit. A tracked-buffer
budget is admitted only when transient allocations and reallocation are accounted
for; do not call it a hard process-RSS limit. No claim of universal OOM recovery.

Deterministic ordering and approximation starts are default. Float operations may
still differ across numerical kernels/platforms. No global mutable workspace or
random generator belongs in the Rust kernel. Batch examples initially use normal
Rust iteration; workspace reuse requires reset/isolation tests before exposure.
No automatic switch may change the filtration, field, precision or approximation.

## Verification and native comparison

Use the [testing policy](../development/testing.md) and
[native protocol](../../benches/protocol.md). The matrix below specifies new
obligations across all stages. Implemented coverage and remaining release gates are mapped in the
[acceptance audit](rips-acceptance.md). Cover meaningful interactions between
axes, not only one happy-path example per feature.

| Axis | Required fixtures and checks |
| --- | --- |
| Layout and callbacks | Identical lower/upper/square/point/function data; shapes, asymmetry, diagonal, NaN/infinity, signed zero, callback failure and explicit zero/one vertex inputs |
| Graph semantics | Empty graph, isolated vertices, duplicate edges, self-loops, nonmetric weights, zero edges, threshold equality, missing edges, dense-versus-threshold parity and distinct supplied-graph coverage |
| Structure | Triangle, square, tetrahedron, disconnected graphs, exhaustive small graphs; closure, max-edge values, lookup round trips, filtration order and oriented boundary squared |
| Dimension | Octahedral sphere with later filling for H2; small higher-dimensional cross-polytope flag spheres; truncated skeleton rejection and top-dimensional killing cells |
| Field | F2 specialized versus field-generic; F3/F5 and large supported primes; invalid moduli, inverses, overflow and a torsion-sensitive triangulation made flag by subdivision |
| Approximation | Empty/singleton/duplicates, ties, fixed start, insertion radii, epsilon around 0.5 and 1, positive minimum radius, cutoff boundaries, overflow and higher-simplex blockers |
| Representatives | Cycle/cocycle equations, nontriviality, basis rank, active interval association, duplicate intervals, original IDs and scales outside coverage |
| Ownership and failure | Drop source after construction/result, repeat calls, cancellation and limits, index overflow, input immutability and no partial-success result |

Reference roles are deliberately different:

1. GUDHI C++ compares exact and approximate simplex sets and filtration values,
   then persistence with matching field/dimension conventions. A matching diagram
   alone is insufficient evidence for construction.
2. Upstream Ripser C++ compares exact dense/sparse persistence and prime fields
   with the correct build option. It is not a full approximate-filtration oracle
   or a mainline representative-output oracle.
3. Hand expectations, independent enumeration/rank reduction and algebraic
   identities check assumptions shared by both external programs. Keep the current
   H0/H1 oracle and add dimension-generic evidence independently.

For sparse approximation, align the start and greedy ordering when demanding
identical simplices. GUDHI may resolve ties differently: record its ordering and
compare matched-order construction, and independently validate each public tie
policy. Do not skip all tied fixtures or classify unequal constructions as equal
merely because the diagrams happen to match.

Compare exact shared filtration values without arbitrary tolerance inflation.
The pinned Ripser uses float32; use shared exactly representable fixtures for
three-way checks and separate full-f64 GUDHI comparisons. Record quantization,
finite/unpaired endpoint normalization and unsupported external cases. Keep native
workers free of Python TDA wrappers; Python may orchestrate processes.

Measure input validation/conversion, distances/graph construction, expansion,
persistence and output assembly separately, plus end-to-end time and process
peak memory. Test dense and thresholded inputs, circles, nonmetric cases,
bipartite graphs and representative-enabled runs. Keep timeouts/regressions and
state compiler, field, dimension, threshold and source hashes. Algorithm-only
timing must not conceal a compulsory dense conversion. Existing retained runs do
not measure new implementations.

## Delivery sequence and exit gates

Every stage includes implementation, contract tests, relevant native comparisons,
current-behavior documentation, an example, and review of resource costs. Entries
below retain the full target. Stages 1-4 are implemented; their current validation
is described in the testing guide. Stage 5 evidence and remaining hosted checks
are distinguished in the [acceptance audit](rips-acceptance.md).

| Stage | Work and dependency | Exit gate |
| --- | --- | --- |
| 1. Exact sparse path | R1-R3, owned context from R9, initial R10 controls; connect graph access to existing F2 H0/H1 | Dense/sparse/point parity through cutoff; complete supplied-graph semantics; no hidden dense buffer in sparse computation |
| 2. General exact Rips | R4-R5; generalized indexing/enumeration, dimension progression and explicit access | Native H2+ comparisons, known sphere/filling fixtures, explicit/implicit agreement and dimension sufficiency checks |
| 3. Fields and representatives | R6-R7, field/representative portion of R9; production algebra components as actually needed | Odd-prime and torsion tests, independently checked representative bases, unchanged diagram-only semantics |
| 4. Approximate Rips | R8 using proven earlier access/reduction components | Full construction and persistence parity under matched conventions; blocker regressions; published hypotheses and provenance |
| 5. Subsystem acceptance | Audit R1-R10 across stages | All public paths documented/tested, cross-platform/MSRV/package checks, source-pinned native evidence and resource reports |

The initial API review focuses on graph/provenance separation, matrix compatibility,
result naming, and explicit dimension coverage. Later implementation reviews must
settle the exact sparse-approximation theorem and minimum-radius guarantee,
representative interval association, and every optimization's admissibility.
These are concrete preimplementation obligations, not decisions to silently defer
until after declaring parity. A design document alone satisfies none of R1-R10.

[g-exact]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Rips_complex/include/gudhi/Rips_complex.h
[g-sparse]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Rips_complex/include/gudhi/Sparse_rips_complex.h
[g-manual]: https://gudhi.inria.fr/doc/3.13.0/group__rips__complex.html
[r-source]: https://github.com/Ripser/ripser/blob/01add51ff64aaf40889483260cc5c3b7d0f2a1e7/ripser.cpp
[r-readme]: https://github.com/Ripser/ripser/blob/01add51ff64aaf40889483260cc5c3b7d0f2a1e7/README.md
