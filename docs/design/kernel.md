# Rust TDA kernel design

[Documentation](../README.md) / Design

Status: adapter/resource corrections, the result-API revision, algorithm-boundary
cleanup and the reduction contributor walkthrough are implemented locally.
This page defines the
intended responsibilities, type boundaries and implementation sequence.
[Current architecture](../development/architecture.md) and rustdoc describe
the code; the [roadmap](roadmap.md) owns delivery priorities.

## Objective and scope

Build a native Rust TDA library with runnable analysis examples. Use GUDHI's C++
capabilities and Ripser's specialized computation as mathematical and algorithmic
references. Researchers should implement their algorithms without adopting a
project-wide execution protocol or learning unrelated construction families.

The design shares mathematical meaning, data access and useful implementation
facilities. Algorithms retain their representations, preparation, ordering and
working state. Ownership, conversion costs, output interpretation and unsupported
cases must be explicit. A CLI, bindings, dataframe integration and hosted services
remain separate from this kernel work.

This revision keeps one crate and the existing domain directories. There is no
planned engine registry, universal algorithm trait, mandatory directory split or
empty Alpha/cubical scaffold. New types below serve specific result semantics;
they are not a requirement for every mathematical operation.

## Current behavior and planned changes

| Area | Implemented today | Decision for this revision |
| --- | --- | --- |
| Persistence algorithms | Specialized H0 and F2 H1, implicit higher-dimensional prime-field cohomology, generic boundary reduction | Preserve independent implementations and their applicable optimizations |
| Explicit storage | Immutable, validated simplices with oriented incidence; signed values and unequal vertex births | Reuse construction guarantees and inexpensive structural metadata |
| Default API | Sealed `PersistenceExt`, Builder requests and private dispatch | Keep the default facade controlled; algorithms remain independently callable internally |
| Generic cells | Open four-method `FilteredComplex` with iterator-returning methods | Keep it an optional static generic adapter, not a universal algorithm interface |
| Results | `PersistenceResult` composes common `PersistenceData` with optional simplicial representatives | Compatible specialized outputs reuse the common data |
| Computed dimensions | Explicit nonempty sets; default builders still compute `0..=q` | Consumers check membership; maximum alone is insufficient |
| Execution | Cooperative budgets/cancellation across Builder terminals; documented legacy exceptions | Preserve scopes; keep controls independent of mathematical source data |
| Other domains | Alpha and cubical construction are not implemented | Define extension boundaries without adding placeholder implementations |

The existing core is not universally routed through one reducer. The first step
addresses particular adapters: redundant validation of immutable input, repeated
structural scans and unnecessary exact-Rips edge retention.
They do not establish a mathematical defect in the filtered-cell contract.

## Design decisions

### Dependency and type design

| Component | Responsibility | Dependency boundary |
| --- | --- | --- |
| Geometry and complex data | Coordinates, distances, topology, incidence and filtration values | No persistence requests, dispatch or reducer working state |
| Filtration construction | Construct the mathematical source and establish its source-specific guarantees | Does not invoke persistence merely to produce an explicit complex |
| Algorithm implementation | Required access, preparation, parameters, workspace and mathematical output | Uses domain data and selected algebra/execution tools; does not depend on the default Builder or its dispatch |
| Default API adapter | Interpret the request and select a compatible implementation | Calls algorithms; algorithms do not call back into default dispatch |
| Result and analysis | Own interpreted outputs and derive measurements from compatible data | Analysis does not depend on the producing algorithm |

The implementation routes are:

```text
default Builder -> source adapter -> algorithm function -> mathematical output
optional specialized entry ------> algorithm function -> mathematical output
```

These are responsibility boundaries, not mandatory materialized stages. A direct
persistence algorithm may interleave geometric access, construction and reduction.
It need not first build a generic complex. An explicit construction operation
remains independently callable.

Ordinary algebra components do not import geometry, source context or diagram
assembly. Use private working indices where index spaces differ; an internal
integer is not a universal public cell identity. Local helpers stay with their
algorithm until their shared semantics justify reuse.

### Algorithm implementations and default integration

An implementation may support a specified subset of fields, dimensions or inputs.
It need not implement every default Builder option. Unsupported requests are
explicit; automatic selection cannot silently weaken the requested guarantee.

Independent algorithms can reuse the same parameter and result types. Clearing,
enumeration and column-storage alternatives can remain private. A specialized
public entry point is appropriate when users need a meaningful algorithm choice,
different parameters or a different mathematical operation. Neither a new public
function nor a new options type is required for every internal implementation.

`FilteredComplex` serves boundary-based consumers; `ZeroBornSimplicialAccess`
serves the existing zero-born coface algorithms. Other algorithms need not adopt
either contract. Sharing union-find mechanics, for example, does not imply sharing
a pairing rule for arbitrary vertex births. Share arithmetic and access only when
domains, ordering, overflow behavior and costs agree.

### Ownership and execution

Use a borrow when an operation only needs access, and an owned argument when it
needs ownership. Avoid borrowing merely to clone immediately. Stable borrowed
sources remain unmodified; consuming algorithms may own mutable working data.
Pivots, transformation columns, annotations and union-find state belong to the
computation or an explicit workspace.

Builders configure operations; small mathematical functions need no Builder.
Reusable construction requests may borrow input, while a one-shot stateful
callback may require a consuming terminal. These choices follow ownership rather
than a universal Builder convention.

Each controlled public terminal creates one cooperative budget and passes it
through internal composition. An internal call does not reset the budget.
Cancellation cannot interrupt arbitrary user callbacks, allocation or sorting.
Work counts are algorithm work, not milliseconds or bytes; no hard deadline,
process-RSS bound or recovery from every allocation failure is promised.

A reusable workspace may retain capacity but resets logical state and cannot
keep stale input references. Preserve `Send`/`Sync` where the data permits them;
do not require thread-safety bounds on every algorithm in anticipation of future
parallelism. Batch use starts with ordinary iteration. A future scheduler must
define ordering, failure behavior and concurrency separately.

### Extension boundaries

| Extension surface | Policy |
| --- | --- |
| Default `.persistence()` facade | Sealed and library-controlled |
| External filtered-cell storage | Open `FilteredComplex` implementation with the documented mathematical contract |
| In-crate algorithm contribution | Reuse internal facilities and expose a focused operation when useful |
| Runtime-selected implementations | Separate future adaptation at the selection boundary |
| Stable external algorithm/plugin API | Outside this revision; internal reducers and budgets are not promised stable |

The current `FilteredComplex` returns `impl Iterator` from trait methods and is
not `dyn` compatible. Its intended use is static generic dispatch. Do not box
every cell iterator or impose dynamic dispatch solely to prepare for hypothetical
plugins. Static external type adaptation and runtime plugin loading are different
capabilities.

An algorithm needing an additional access operation can define a suitably narrow
interface without enlarging every existing complex's obligations. The module
organization stays domain-based; implementation details remain private. Public
trait bounds are placed where needed rather than propagated through every type.

### Source facts and result assembly

| Fact | Authority |
| --- | --- |
| Source identity, construction settings, scale convention, vertex mapping and approximation hypotheses | Source or constructor |
| Available source scale and skeleton coverage | Construction evidence |
| Requested field, analysis range and outputs | Operation request |
| Computed dimensions, intervals and established coverage | Computation using source evidence and the request |

Equivalent entry points for a source family reuse a private assembly function
with these facts. Direct computation carries them without materializing a complex;
prepared and explicit routes use stored facts. Assembly produces interpreted
result data, not an execution plan or an algorithm driver.

Temporary retention thresholds, cone stopping bounds and working indices do not
overwrite requested settings. A completeness proof may establish output coverage;
an unexplained internal stop cannot. A new filtration family owns its own
interpretation rather than modifying Rips rules to describe itself.

Legacy APIs with deliberately different context or budget semantics use explicit
compatibility adapters. They are not silently treated as equivalent new routes.

### Validation and preparation

The private ingestion paths are distinct:

- `&SimplicialComplex`: use immutable construction guarantees for uniqueness,
  face closure, ordering and oriented incidence. Select needed cells and convert
  coefficients without repeating external validation.
- `&impl FilteredComplex`: validate the supplied traversal and selected
  boundaries under the existing selected-field validation contract.

Both can feed the existing boundary reducer. An algorithm that does not consume
boundary columns bypasses both. No public trust flag or unchecked escape hatch
selects a fast path. Cache cheap structural facts during construction instead of
rescanning all simplices for each query.

Requested construction range, source coverage and analysis range remain distinct.
A direct exact-Rips query can retain fewer edges when its analysis cap is lower,
without changing source metadata or skipping required distance validation.
Preparation for sparse approximation may need information beyond the analysis
range; sampling, mappings, hypotheses and high-dimensional blockers remain local
to that algorithm. Common range meaning does not prescribe a common preparation
sequence.

### Results carry enough information to be interpreted

`PersistenceData` owns a `PersistenceDiagram` and
`ComputationContext`, with private fields and borrowed `diagram()`/`context()`
access. It contains no representatives or engine state. Computed dimensions and
established coverage live in the diagram; source facts, coefficient field and
requested range live in the context.

The existing `PersistenceResult` composes this data with its optional simplicial
representatives and retains delegating accessors. Compatible specialized results
compose the same data with their own typed auxiliary output. Diagram-only
operations can return the common data directly. Operations with different interval
semantics use their own results.

Existing representatives retain their persistent-cycle and dual-cocycle guarantees.
A cellular cycle or an independently obtained cocycle must not be forced into
that type. Its output specifies cell identity/lifetime, orientation, field,
query scale and any interval association actually established. Witness identities
refer to the final owning diagram; projection or reordering must explicitly
remap associations. Shared access does not permit changing the diagram beneath
its witnesses.

#### Borrowing and conversion costs

Compatible wrappers and `PersistenceData` itself implement
`AsRef<PersistenceData>` by borrowing already stored data. This conversion is
infallible and performs no allocation, copying, sorting, validation or context
assembly. It is not a mathematical compatibility certificate.

Context-aware analysis uses a thin generic entry that borrows the common data,
then calls concrete internal computation functions. Raw-diagram operations
retain their narrower contract. Specialized results do not have to discard
context to reuse field/scale checks. Expensive or fallible representation changes
use explicit methods or appropriate conversion traits instead of `AsRef`.

This adapter supports results containing the common data; it does not promise
zero-copy interoperability with every external representation. A distinct
borrowed view can be designed if an actual storage family needs it.

#### Computed dimensions

`ComputedDimensions` represents a nonempty set of dimensions actually computed.
Its public contract is membership, iteration and maximum; its storage is private.
Normalized inclusive ranges are an implementation option, not a public layout
promise. An empty interval list in a computed dimension differs from an
uncomputed dimension.

`PersistenceDiagram::new(q, ...)` keeps contiguous `0..=q` construction.
`ComputedDimensions::new` normalizes an explicit list; `through(q)` creates the
contiguous set without enumerating it. `PersistenceDiagram::with_dimensions`
accepts the declared set, exposed by `computed_dimensions()`. Every interval belongs to
the declared set. Dimension-specific descriptors and distances reject uncomputed
dimensions. An operation over all dimensions requires matching sets or an
explicitly selected common domain; it never silently intersects away requested
information.

#### Result construction and external integration

This revision creates complete common data through in-crate source assemblers.
External callers can still construct raw diagrams under the existing API.
Wrapping a library-produced `PersistenceData` does not open arbitrary context
construction or algorithm registration.

Private fields protect representation and invariants; they do not imply that
all constructors must remain private forever. A future public result-import API
must distinguish caller-declared metadata from construction-established source
guarantees. Structural checks cannot prove that an external algorithm computed
the diagram correctly or establish a Rips approximation theorem. No import API
may silently confer such a certificate. That boundary is separate from a stable
external reducer API and is not implemented by this revision.

### Result API migration

This is an implemented pre-release breaking revision. Previously,
`max_dimension()` promised that every lower dimension was computed. It now returns
the greatest member of a possibly noncontiguous set; its Rust signature is
unchanged, but its semantics differ. Migrate callers as follows:

1. Add membership and iteration. Replace availability checks and applicable
   `0..=max_dimension()` loops, and update dimension-error diagnostics.
2. Preserve contiguous construction and existing default Builder request meaning.
   Document the new maximum as an upper bound, not proof of membership.
3. Introduce common result data and compose existing results around it. Generalize
   context-aware entry points while preserving field/scale checks. Include
   function-pointer and type-level signature changes in the migration statement.
4. Update rustdoc, guides, examples and maintained callers together.

Use `diagram.computed_dimensions().contains(k)` for availability and
`diagram.computed_dimensions().iter()` for traversal. `DimensionNotComputed` keeps
its `computed_max` field as an upper bound, including errors for gaps below it.
No current public operation implicitly combines all dimensions; future ones must
require matching domains or explicit selection.

The three context-aware distance functions now accept independent generic types
`L, R: AsRef<PersistenceData> + ?Sized`. Ordinary calls with two
`PersistenceResult` values still work. Function-pointer annotations select concrete
operand types; explicit specialization can use
`bottleneck_distance_results::<PersistenceResult, PersistenceResult>`.
`PersistenceResult::into_data` discards representatives while preserving context;
`into_parts` moves both without changing interval associations. Full-result import
remains crate-internal.

This migration does not remove legacy
construction APIs or change their documented context and budget scope.

## Mathematical and resource boundaries

Keep the following distinctions in both types and operation contracts:

- Exact threshold Rips, an arbitrary supplied flag graph, sparse approximation,
  subsampling and persistence-preserving simplification have different guarantees.
  A sparse graph's largest edge does not prove original-source coverage.
- A finite analysis cutoff restricts the mathematical question. Cancellation and
  budget exhaustion return execution errors, not a successful partial diagram.
- Filtration scale convention, physical units and metric validity are distinct.
  Radius, squared radius and edge length cannot be substituted implicitly.
- Input preprocessing is explicit. Do not silently normalize, deduplicate, sample
  or replace a nonmetric dissimilarity by a metric.
- Signed filtrations remain valid where supported; NaN, infinity, masking and
  numeric policies belong to each domain. Missing cells are not large finite values.
- Representatives are optional and need not be computed for diagram-only results.
  Different valid bases are not required to have identical coefficients.
- Shared output shape does not equate ordinary, extended and zigzag persistence.

Analysis operations consume the required result data directly. Feature grids,
bandwidths, dimension selection and endpoint policies must agree across samples.
Matching distances retain their [supported endpoint and context contracts](../reference/mathematics.md#16-diagram-matching-distances).
These rules do not add a dataframe, plotting or asynchronous runtime dependency.

Document representation costs, including dense pairwise storage, retained edges,
incidence, conversion and reduction fill-in. An allocation counter is not a hard
RSS limit. Deterministic work counters diagnose implementation costs but are not
cross-algorithm performance units.

## Future capability placement

The [current architecture](../development/architecture.md) owns the source tree.
The table describes responsibility, not new directories to create now.

| Capability | Reusable parts | Capability-specific responsibility |
| --- | --- | --- |
| Alpha construction | Explicit simplicial storage and compatible persistence/result analysis | Predicates, triangulation, degeneracies, filtration values and radius conventions |
| Cubical topology and persistence | Optional filtered-cell access, suitable algebra, compatible results | Grid representation, arithmetic incidence and specialized computation |
| Additional persistence algorithm | Relevant access, field/column operations and interpreted outputs | Ordering, preparation, reduction, workspace and optional witnesses |
| Diagram descriptors and metrics | Diagram/result contracts and meaningful numerical helpers | Formula, endpoint support and derived representation |
| Scalar-line persistence | Compatible ordinary results | Direct scalar algorithm without mandatory complex construction |
| Zigzag, cover/nerve or reconstruction | Applicable local primitives | Event, graph or geometric output semantics; no forced ordinary-diagram conversion |

Pure Rust geometry requires appropriate predicates and numerical algorithms, not
only translated formulas. New coefficient domains similarly require their own
algebraic semantics. Future capabilities do not weaken the selected
[Rips scope](rips.md#required-capability-matrix) or imply current GUDHI parity.

## Implementation sequence

Step 1 is implemented: frozen simplicial storage caches structural facts; the
private `persistence/simplicial/input.rs` reader converts selected stored incidence
without repeating external-cell validation; direct exact point analysis retains
only edges within both caps. Public signatures and result semantics are unchanged.
External `FilteredComplex` validation remains in `persistence/filtered.rs`.
Approximate Rips preparation is unchanged because its sampling and blocker rules
need separate analysis.

Step 2 is implemented: `diagram/dimensions.rs` and `diagram/data.rs` own the new
contracts; internal source adapters share result/context assembly; descriptors
and distances respect dimension membership; context-aware distances accept
borrowed common data. The migration, architecture, contributor guidance and
diagram example are synchronized.

Steps 3–4 are implemented. `persistence/boundary` owns selected column input and
diagram computation independently of external filtered-cell validation. Exact
flag selection lives in `flag/dispatch.rs`; explicit/approximate Rips call
simplicial algorithms directly. The shared optional-representative policy for
zero-born access lives in `simplicial::finish_zero_born`. Algorithms remain free
to use their own preparation, access and outputs. No new algorithm registry or
public selection API was needed. The
[reduction walkthrough](../development/persistence-reduction.md) and its focused
CI check cover direct mathematical work followed by source integration.

| Step | Code scope | Structural change |
| --- | --- | --- |
| 1. Adapter/resource correction | `complex/simplicial`, `persistence/simplicial`, `persistence/filtered.rs`, direct Rips preparation | Reuse immutable guarantees, cache structural facts, avoid unused exact-Rips edges; keep public result semantics |
| 2. Result API revision | `diagram`, result assembly, descriptors and diagram distances | Add common data and computed dimensions, thin reference adapters, shared context assembly and the explicit migration above |
| 3. Algorithm integration | Existing persistence algorithms and source adapters | Keep algorithms independent of the default facade; reuse compatible types; expose only meaningful specialized operations |
| 4. Contributor documentation | Architecture, algorithm guides and examples | Describe actual call boundaries, ownership and extension scope; update the module map to match implemented code |

Keep mathematical optimization changes separate from adapter and API refactors.
Local file splits follow implemented responsibilities. Public naming and trait
bounds are compatibility decisions, not consequences of how many files exist.
External result import, runtime plugins, generic cellular witnesses and additional
filtration families remain separate workstreams.

## Verification of the design implementation

The dependencies, types and ownership above define the architecture. Verification
checks their implementation; it does not replace design with admission scenarios.
Use the [testing strategy](../development/testing.md) and independent mathematical
oracles, preserving their independence from production algorithms.

Exercise independently varying construction/query caps, built/requested dimensions
and diagram/representative requests. Include unequal and signed births, non-flag
topology, fields and approximation blockers. Validate witness equations and
associations rather than requiring arbitrary bases to match.

Native comparisons use pinned GUDHI/Ripser C++ paths with compatible capabilities
and timing boundaries. Update the existing [performance report](../../benches/reports/rips-comparison.md)
against full commits under the [reporting rules](../../benches/reporting.md).
Generated evidence stays outside Git; unit tests do not assert machine timings.
A documentation revision makes no new performance or algorithm-correctness claim.

## Design references and adopted choices

These primary references inform the choices above; they do not prescribe a single
architecture for Rust libraries. Linked library APIs are design examples, not new
dependencies or a recommendation to copy their entire frameworks.

| Reference | Adopted choice |
| --- | --- |
| [Rust API Guidelines: flexibility](https://rust-lang.github.io/api-guidelines/flexibility.html) | Expose useful data, make ownership/copying explicit, and weigh genericity against signature and code-size costs |
| [Rust API Guidelines: type safety](https://rust-lang.github.io/api-guidelines/type-safety.html) | Types distinguish meaningful states; Builder ownership follows the operation |
| [Rust API Guidelines: future proofing](https://rust-lang.github.io/api-guidelines/future-proofing.html) | Private representation and deliberate sealed facades preserve evolution space |
| [Standard-library AsRef](https://doc.rust-lang.org/std/convert/trait.AsRef.html) | Borrow existing data cheaply and infallibly; no conversion work or mathematical certification |
| [Rust Reference: dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility) | Separate static iterator-based adaptation from runtime trait-object interfaces |
| [petgraph Dijkstra](https://docs.rs/petgraph/0.8.3/petgraph/algo/dijkstra/fn.dijkstra.html) | Algorithms request relevant access capabilities rather than one concrete graph storage |
| [ndarray ArrayView](https://docs.rs/ndarray/latest/ndarray/type.ArrayView.html) | Distinguish borrowed views from owned storage without requiring callers to copy |
| [argmin Solver](https://docs.rs/argmin/0.11.0/argmin/core/trait.Solver.html) | Separate mathematical work from execution support; its iteration protocol is specific to that framework and is not imposed on TDA operations |
| [GUDHI C++ study](../research/gudhi-cpp.md) | Permit specialized representations and algorithms with operation-specific contracts |
