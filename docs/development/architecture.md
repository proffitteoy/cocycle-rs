# Current architecture

[Documentation](../README.md) / Development

This page describes implemented code. Proposed capabilities and their acceptance
gates belong in the [kernel design](../design/kernel.md).

Cocycle is one Rust crate with a small public API. Modules follow mathematical
responsibilities without requiring each computation to construct every possible
intermediate object. The [mathematical specification](../reference/mathematics.md)
defines invariants; rustdoc defines public signatures and error behavior.

## Production code

```text
src/
  lib.rs                         public domains and Error/Result exports
  error.rs                       shared structured errors
  execution/
    mod.rs                       shared controls and private per-operation budget
  algebra/
    field/mod.rs                 validated PrimeField and modular arithmetic
    column/mod.rs                private ordered sparse coefficient columns
    reduction/boundary.rs        forward reduction retaining transformations
  geometry/
    point_cloud.rs               borrowed coordinate validation
    dissimilarity.rs             existing condensed input contract
    matrix.rs                    borrowed lower/upper/square matrix layouts
    metric.rs                    explicit metric policy and exhaustive triangle checks
    distance.rs                  checked scalar distance/cutoff operations
    euclidean.rs                 shared pair evaluation and dense conversion
  complex/
    graph/
      mod.rs                     owned WeightedGraph, edges and queries
      adjacency.rs               compact sorted adjacency construction
    simplicial/
      mod.rs                     frozen complex, lookup and stored incidence
      simplex.rs                 canonical vertices, IDs and shared filtration order
      incidence.rs               oriented codimension-one boundary terms
  filtration/
    provenance.rs                construction coverage and source-kind identity
    expansion/mod.rs             contextual explicit result and controlled construction
    rips/
      builder.rs                 exact borrowed settings and one-shot callbacks
      exact.rs                   ThresholdRips construction, provenance, cone bound
      expansion.rs               explicit Rips and dimension/scale provenance
      approximation/
        builder.rs               approximate settings and one-shot callbacks
        metadata.rs              owned sampling provenance and conditional bounds
        options.rs               epsilon, sampling and range parameters
        greedy.rs                deterministic farthest-point permutation
        edges.rs                 modified sparse edge values
        blocker.rs               hereditary insertion-radius constraint
        access.rs                original-label blocker-aware cofaces
        expansion.rs             frozen sparse topology and provenance
    flag/
      mod.rs                     supplied FlagFiltration
      access.rs                  private dense/sparse access contract
      dense.rs                   matrix-backed edges and cofacets
      sparse.rs                  sorted neighbor-intersection cofacets
      index.rs                   checked edge/triangle IDs and decoding
      order.rs                   compact H1 entry comparison adapter
      cliques.rs                 dimension-generic dense/sparse/explicit access
      expansion.rs               shared explicit clique construction
  persistence/
    mod.rs                       public exports and result normalization
    execution.rs                 compatibility control imports
    builder.rs                   borrowed analysis settings and validated execution
    source.rs                    sealed extension and private source dispatch
    options.rs                   prime-field PersistenceOptions
    rips/
      mod.rs                     legacy/new Rips entry points and source coverage
      options.rs                 compatible RipsOptions
      expanded.rs                explicit incidence computation and dimension checks
      approximation.rs           sparse Rips computation and context assembly
    flag/
      mod.rs                     shared dispatch and supplied-graph entry point
      h0.rs                      independent H0 edge scan
      union_find.rs              private connectivity shared by H0/H1
      representatives/
        basis.rs                 persistent cycles and interval association
        request.rs               scale/dimension/selection validation
        complex.rs               opt-in skeleton and oriented boundary assembly
        dual.rs                  scale-specific dual cocycle basis
      cohomology/
        mod.rs                   shared dense/sparse implicit F2 H1 reduction
        dimensions.rs            prime-field implicit reduction with clearing
        tests.rs                 optimization/duality tests
        profiling.rs             explicit diagnostic tests
    tests.rs                     independent Rips/graph oracle comparisons
    reference/                   test-only explicit boundary reduction
  diagram/
    interval.rs                  interval validation and endpoint semantics
    persistence_diagram.rs       owned multiset and coverage
    computation.rs               owned PersistenceResult and source context
    representative.rs            owned chain/cochain terms and local interval IDs
  descriptors/                   diagram-only lifetimes and Betti curves
  diagram_distances/              complete-diagram bottleneck, W1 and W2 matching
    bottleneck/                  geometric decisions, matching and capacity flow
    wasserstein.rs               adaptive routing and component solves
    wasserstein/numeric.rs       checked scaling and original-cost reconstruction
    wasserstein/graph.rs         positive-saving candidates, storage and components
    wasserstein/dense.rs         dense SAP and small matching certificates
    wasserstein/sparse.rs        sparse primal-dual matching
    wasserstein/direct.rs        original-cost numerical fallback
```

Each domain has a documenting/exporting `mod.rs`; the tree lists the substantive
files. Public paths are re-exported from domains, not every private directory.

| Module | Responsibility | Dependencies within the crate |
| --- | --- | --- |
| `algebra` | Validated prime fields and private sparse arithmetic/reduction | Error utilities |
| `geometry` | Input validation and distance access | Error utilities |
| `complex` | Checked weighted graphs and frozen simplicial incidence | Geometry scalar validation, error utilities |
| `execution` | Immutable controls and private per-operation budget | Error utilities |
| `filtration` | Rips construction, provenance and shared flag access | Geometry, complex, execution, error utilities |
| `persistence` | Algorithms, options, interval and representative assembly | Algebra, geometry, filtration, diagram, execution |
| `diagram` | Algorithm-independent result ownership and validation | Algebra field identity, filtration provenance, error utilities |
| `descriptors` | Read diagrams without recomputing persistence | Diagram, error utilities |
| `diagram_distances` | Match complete diagrams with bottleneck/L-infinity, W1/L-infinity or W2/Euclidean costs | Diagram and context types, error utilities |

Geometry and diagram code do not call persistence. Filtration code does not call
persistence. Descriptors do not inspect source coordinates or algorithm state.
Public graph construction and frozen explicit simplicial expansion are available.
Public boundary-matrix traits and backend registries are not implemented.

Each implemented domain has a directory, even while its implementation is small.
The domain's `mod.rs` documents its scope and exports its public API; named child
files own concrete data invariants or computations. These files are private
implementation modules, so existing public type and function paths stay stable.
`lib.rs` and the shared `error.rs` remain crate-level files.

Here `geometry` includes nonmetric dissimilarities, `filtration` provides ordered
topological access, and `persistence` computes persistent homology using that
access. `diagram` is a mathematical result container, not plotting, and
`descriptors` computes measurements from diagrams. Geometric distance operations
belong to `geometry`; `diagram_distances` compares diagram multisets and does not
recompute persistence or inspect the original point clouds.

Distance counters and forced policies live in private `instrumentation.rs`
modules, compiled under `cfg(test)` or the standalone worker's
`cocycle_distance_bench` cfg. Ordinary library builds use empty private carriers
and the selected adaptive policy. Counter statements and capacity traversals
are removed by conditional compilation, including in debug builds. Binary-search
ablation and the experimental arena layout are excluded from ordinary builds.
The worker links the ordinary public crate for its separate `public` checks.

## Public workflow and ownership

`RipsBuilder` and `ApproximateRipsBuilder` configure borrowed inputs.
`build_complex(max_simplex_dimension)` performs explicit construction and returns
an owned `SimplicialFiltration`; only expanded topology exposes simplex queries.
Alternatively, importing `persistence::PersistenceExt` enables `.persistence()`
and the returned request's `.compute()`. The extension is implemented entirely
in the consumer module, so filtration never calls persistence.

Advanced `prepare()` returns an owned exact/approximate source for reuse. Callback
builders are consumed by preparation or explicit construction; analyses never
replay them. `Execution` is shared configuration, not a shared counter. A new
private budget spans each terminal's preparation and computation. Legacy free
functions retain their existing persistence-only scope and test coverage.

## Computation path

The legacy point API and unrestricted richer point API use a condensed distance
buffer. The richer point API with a finite cutoff streams distances into a
threshold graph. Supplied graphs never require a dense matrix. H0-only requests use
union-find. F2 H0/H1 requests use the specialized implicit Rips path: ordered edges supply H0
merges and candidate H1 births; triangle cofacets are generated during reverse
coboundary reduction. Stored change-of-basis columns and virtual zero-lifetime
apparent pairs supply later eliminations.

Within `src/persistence/flag/cohomology/mod.rs`, `run_access` classifies edges in
forward order and reduces their coboundaries in reverse order.
`initialize_coboundary` scans each original column once, combining initialization
with the apparent/emergent shortcut decision. If the first equal-valued candidate
cannot take a shortcut, the collected cofacets form the ordinary working column.
`initialize_two_pass` and its selection branch exist only under `cfg(test)` for
independent comparisons.

`pivot_owners` records only stored, non-virtual ownership: it maps triangle
simplex IDs to `ColumnPosition` values in the stored-column array.
`TransformColumn` holds `EdgePosition` values in the forward-ordered edge array.
These private types distinguish the two array positions from each other and from
combinatorial simplex IDs. Zero-lifetime apparent pairs occupy neither structure.
For a pivot without a stored owner, `zero_apparent_facet` checks the mutual
pairing conditions; the reduction loop then reconstructs the paired edge's full
coboundary and adds its position to the active transformation. Only an edge
already processed in reverse order can supply a virtual owner. This preserves
the [reconstruction invariant](../reference/mathematics.md#implicit-reconstruction-invariant).
Explicit reduced-column payloads and replay checks exist only in tests.

Higher-dimensional and odd-prime requests dispatch to `cohomology/dimensions.rs`. This path
classifies H0 edges, then advances through dimensions with clearing. It retains
one ordered simplex dimension, pivot owners and coefficient-bearing transformation
columns; reduced
coboundaries are regenerated on demand. Ordered vertex tuples avoid binomial-ID
overflow in sparse high-dimensional input. `cliques.rs` supplies dense candidates,
common neighbors from the shortest sparse adjacency list, or stored explicit
incidence through one private `SimplicialAccess` contract. The specialized H1
path keeps its existing apparent/emergent shortcuts; the generic path currently
uses clearing without those shortcuts. Oriented cofacets use the sign of their
omitted vertex; pivot columns are normalized over the selected field. No claim
of performance parity is made.

Explicit expansion uses unique increasing-vertex extensions and freezes all
stored simplices with lookup and both directions of incidence. Its ordering and
the compact H1 entry order use the comparison authority in `complex/simplicial`.
`SimplicialFiltration` retains construction dimension and scale provenance
separately for exact Rips, approximation and supplied flags. The old expansion
types remain available during migration.
Explicit computation reads its incidence, rejecting insufficient skeletons unless
expansion certified clique exhaustion. Graph construction and expansion do not
invoke persistence, and computation does not mutate stored topology.

H0 and H1 both use `src/persistence/flag/union_find.rs`, with private state.
The component only tracks connectivity; its callers decide when to stop scanning
and how a merge contributes persistence pairs. H1 does not import H0's algorithm.

`resolve_rips_range` selects the cutoff and caller-visible coverage. Both paths
normalize raw intervals in `assemble_diagram`. That step applies requested
dimensions, zero-lifetime removal, and the public coverage/censoring convention.
The internal cone stopping bound must not replace the caller's coverage.
Results own their data and contain no simplex IDs or borrowed work buffers.

Private `DenseFlag` and `SparseFlag` implement `FlagAccess`: forward-ordered
edges, decreasing-ID cofacets and latest-facet lookup. The engine provides a
checkpoint callback so filtration never depends on persistence execution types.
`SimplexEntry` combines a combinatorial ID and value, not an array position.
Sparse cofacets intersect two sorted adjacency lists and retain the dense tie
order. The cone bound applies only to complete pairwise input, not arbitrary
supplied graphs.

`ThresholdRips` certifies coverage against all original pairs. `FlagFiltration`
defines the supplied graph itself, including permanently absent edges. Their
entry points select coverage before calling the shared engine. `PersistenceResult`
adds owned source context without changing the diagram or descriptors. Explicit
CSR-like offsets and neighbors belong to graph storage; pivots and heaps stay
private to each computation.

Requested representatives use `flag/representatives`, materializing only the
required skeleton and assembling oriented boundaries. `algebra/reduction` owns
ordinary forward reduction and transformations, independently of filtration and
diagram types. `algebra/column` supplies sparse coefficient operations shared
with implicit cohomology and dual solves. The private test oracle remains separate.
Finite cycles use reduced death columns; unpaired cycles use birth transformations.
`representatives/dual.rs` solves scale-specific boundary-annihilation and cycle
pairing constraints. Terms leave the computation as original vertex lists and
canonical coefficients, associated with positions in this result's sorted diagram.
These opt-in entry points return the same diagram as their implicit counterparts.

The numeric choices are `f64` filtration values and validated prime-u32 fields,
with F2 the default.
Generic interval types allow finite signed scales and arbitrary dimensions;
Rips entry points and Betti queries enforce their narrower supported contracts.
This representational flexibility is not a promise of additional algorithms.

## Test reference implementation

`src/persistence/reference/` contains `complex`, `boundary`, `explicit`, `rips`,
`column`, and `reduction`. The entire module is gated by `cfg(test)` and is absent
from production builds. It materializes a small 2-skeleton and performs ordinary
left-to-right sparse boundary reduction, independently of the optimized path.

Its `FilteredBoundary` contract requires finite nondecreasing filtration values,
strictly ordered boundary indices before their column, adjacent dimensions, and
boundary squared equal to zero. Sparse-column storage and reduction strategy are
separate. A hand-built filtered triangle verifies the reducer independently of
Rips construction. This is an internal oracle, not an advertised extension API.

The high-dimensional oracle in `tests/rips_expansion.rs` independently enumerates
vertex subsets by bitmask and reduces the ordinary boundary matrix forward.
It does not reuse production clique access, clearing or implicit reconstruction.

Correctness tests belong beside private invariants or under `tests/` for public
contracts. Diagnostic counters and ignored profiling entry points are test-only;
`tools/profile_*.py` invokes them explicitly. Production timings come from the
public API workers, never instrumented test builds.

## Repository support code

- `examples/`: runnable public-API usage.
- `tests/`: external callers' contracts, hand calculations, and properties.
- `benches/rips.rs`: dependency-free Rust benchmark.
- `benches/native/`: native H0/H1 workers, shared source pins and setup.
- `benches/pipeline/`: native complete-workflow workers and timing boundaries.
- `tools/`: optional external comparisons, benchmark controllers, and diagnostics.
- `docs/`: usage, reference, development, design, and upstream research.
- `benches/reporting.md`: cross-suite report and evidence rules.
- `benches/reports/`: maintained comparisons with measured revisions and evidence status.
- `target/`: ignored local build and experiment output; generated artifacts never
  enter the source repository.

## Extension rules

Keep public paths stable when splitting a file into private submodules. Add a new
public abstraction only when a concrete capability needs it and its validation
contract can be specified. A second filtration need not use a simplex-based
representation: it can share diagrams while owning its own input and algorithm.

Higher-dimensional Rips belongs at the filtration/cohomology boundary. Diagram
distances and landscapes can consume existing diagrams without changing Rips.
Non-prime coefficient rings would require different algebra and reduction
semantics; the implemented prime-field column API does not support them. Mapper need not flow through
persistent homology at all. These are boundaries for future work, not empty
modules to create now. See the [roadmap](../design/roadmap.md).

The [kernel design direction](../design/kernel.md) explains how these boundaries
can grow into a native Rust data analysis kernel. Its future capability plans
remain proposals. The [GUDHI C++ study](../research/gudhi-cpp.md) records the upstream
source evidence behind that direction.

## Sparse approximation boundary

Sparse Rips construction owns its compact graph, insertion radii and blocker.
The graph alone is insufficient to reconstruct its higher topology. Its access
implementation and explicit incidence implement the same `SimplicialAccess`
contract, including zero-born original vertex labels and a compact position map
for union-find. Oriented generic cohomology and representative reduction reuse
this access without approximation-specific algebra. The blocker is hereditary,
so unique-extension enumeration may prune rejected simplices safely.

`filtration/rips/approximation/metadata.rs` owns numerical parameters, full greedy order and radii,
retained IDs, metric evidence and conditional bound targets. This module depends
only on geometry's evidence enum, not on a borrowed distance input or algorithm.
Descriptors still consume ordinary diagrams; a caller choosing to discard result
context also discards approximation provenance. See the [sparse guide](../guides/sparse-rips.md).
