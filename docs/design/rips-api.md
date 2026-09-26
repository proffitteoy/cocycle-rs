# Rips API design

[Documentation](../README.md) / Design

Status: implemented with a compatibility stage. Both primary workflows, exact and
approximate preparation, callbacks, explicit source context and whole-operation
controls are available. Guides and examples use the new API. Legacy free functions,
options and expansion methods remain available for migration and retain their old
control scope. Prepared-source names are compatibility aliases of existing owned
types; removing old names/functions requires a separate declared breaking revision.

The Rust blocks illustrate the implemented interface; the runnable guides and
rustdoc provide complete checked examples. This document supersedes earlier
entry-point sketches in the [Rips subsystem design](rips.md). Rips mathematical
semantics remain unchanged. Diagram-only explicit analysis retains the zero-born
coface reducer when its value and cutoff conditions hold; general supplied
filtrations use the public filtered-cell boundary contract. Existing timing
reports describe their measured commits, not later integration revisions.
The public analysis builder carries separate source and representative-request
lifetimes (`PersistenceBuilder<'s, 'r, S>`), normally inferred at the call site.

## Design evidence and decisions

Design from complete caller tasks before choosing public types or directories.
A useful interface makes the resulting object, execution boundary and ownership
predictable. Chaining and a `Builder` suffix are means, not acceptance criteria.

| Reference | Observed design | Application here |
| --- | --- | --- |
| [GUDHI C++ Rips_complex][g-rips] | Prepares a graph and exposes `create_complex(target, dim_max)` against a target concept | Name explicit complex construction and require its dimension at the operation; keep storage contracts separate from enumeration |
| [GUDHI C++ Persistent_cohomology][g-persistence] | Separates the filtered-complex and coefficient-field contracts from the algorithm | Preserve algorithm/storage boundaries; do not equate a fluent API with architectural extensibility |
| [Rust API Guidelines: builders][rust-builders] | Required data enters the constructor, setters configure, terminal methods perform the operation; consuming and borrowed forms serve different ownership needs | Expose configuration only where useful; do not require a builder at every internal phase |
| [std::process::Command][command] | Configures once and uses named execution methods such as `spawn` and `output` | Prefer an explicit operation name when an unqualified `build` would hide what is produced |
| [RegexBuilder][regex] and [ThreadPoolBuilder][rayon-builder] | A builder has a concrete result and a documented execution/error boundary; receiver ownership differs | Decide ownership from actual data transfer, not a universal builder style |
| [Rayon iteration traits][rayon-iter] | Trait imports enable common operations on existing types | An extension trait is a valid tradeoff, but examples must show the import and errors must be checked from an external caller |

These are design references, not dependencies or evidence of performance parity.
The GUDHI [Python Rips interface][g-python] remains a usability reference. Native
comparisons continue to use C++; Python defaults and mutable result caches are
not copied into the Rust contract. In particular, default coefficients remain F2.

The following decisions are local conclusions from these references and the
project's requirements, not rules attributed to those libraries:

- There are two primary tasks: construct an inspectable complex, or compute
  persistence directly. Unexpanded sources expose no public simplex lookup,
  filtration-value lookup for a simplex, or simplex traversal. Their implicit
  access remains private to algorithms.
- `build_complex(max_simplex_dimension)` produces stored topology and its context.
  The required argument removes an unset-dimension state. No separate
  `ExpansionBuilder`, `expand().build()` chain, or missing-dimension error is needed.
- `.persistence()` creates a configurable analysis request; `.compute()` returns
  an owned result. These steps remain separate because fields, dimensions,
  representatives and analysis range are independent choices.
- Preparation for repeated operations is advanced reuse of an existing capability,
  not a third inspection mode. It is optional in both primary workflows.
- Public types must establish a useful contract or ownership boundary. Internal
  dispatch traits, workspaces and intermediate enumeration state remain private.

## Two primary workflows

`points` below is an existing validated `PointCloudView`. Matrix callers substitute
`RipsBuilder::from_distance_matrix(matrix)` using a validated lower/upper/square
view. Input validation and its cost still precede these operations.

### Construct, inspect, then analyze

```rust,ignore
use cocycle::filtration::RipsBuilder;
use cocycle::persistence::PersistenceExt;

let rips = RipsBuilder::from_points(points).max_edge_length(1.0);
let filtration = rips.build_complex(2)?;
let complex = filtration.complex();

let triangle = complex.find(&[0, 1, 2]);
let simplices = complex.simplices();

let result = filtration
    .persistence()
    .max_homology_dimension(1)
    .compute()?;
```

`build_complex(2)` performs preparation and expansion through simplex dimension
two under one budget. It returns `SimplicialFiltration`, owning explicit topology
and its construction context. `complex()` borrows the existing
`SimplicialComplex`, with lookup, filtration-ordered traversal, oriented
boundary and stored codimension-one cofaces. Lookup uses strictly increasing
vertex IDs, as today. It is not a mutable simplex tree or a claim of full GUDHI
container parity. No topology is cloned for inspection.

Construction through q+1 generally supplies the killing cofaces needed for Hq.
Insufficient skeletons are rejected unless expansion certifies the absence of
higher simplices. No automatic re-expansion or reinterpretation as a complete
original Rips filtration occurs. Zero requests vertices only, including isolates.

### Analyze directly

```rust,ignore
use cocycle::filtration::RipsBuilder;
use cocycle::persistence::PersistenceExt;

let rips = RipsBuilder::from_points(points).max_edge_length(1.0);
let result = rips
    .persistence()
    .max_homology_dimension(1)
    .compute()?;
```

The caller obtains an owned `PersistenceResult`, not a queryable complex.
`.persistence()` stores the request without computing. `.compute()` validates,
prepares as needed and reduces. Exact dense implicit, threshold sparse, H0 and
specialized F2 H1 paths remain available. No explicit skeleton or dense conversion
is imposed merely to unify the interface. Representative requests keep their
separate documented materialization costs; direct computation is not a promise
that no internal simplices are ever stored.

Direct exact point analysis retains graph edges through the smaller of the
construction and analysis caps. It still validates all pair distances and records
the caller's construction cap independently of this temporary retention bound.
Reusable `prepare()` continues to retain the full requested construction range.

Both workflows compute the same mathematical source at equal field and range.
They must agree on interval multisets and coverage when the explicit construction
is sufficient. Representation metadata and work performed can differ.

## Public vocabulary

Ordinary users need the input adapter, Rips construction settings and the chosen
output. Return/request types remain nameable for function signatures, without
requiring users to construct every type explicitly.

| Type | Domain | Purpose |
| --- | --- | --- |
| `RipsBuilder<'a>` | `filtration` | Borrowed exact input and construction settings; typical variable `rips` |
| `ApproximateRipsBuilder<'a>` | `filtration` | Borrowed approximate input and settings; typical variable `approximate_rips` |
| `SimplicialFiltration` | `filtration` | Owned explicit filtered complex, source provenance, coverage and dimension certificate; `filtration` |
| `PersistenceBuilder<'s, 'r, S>` | `persistence` | Borrowed analysis request; normally inferred from `.persistence()` |
| `PersistenceExt` | `persistence` | Sealed extension supplying the analysis-request method |
| `PersistenceResult` | `diagram` | Existing owned diagram, context and optional representatives; `result` |
| `RipsFiltration`, `ApproximateRipsFiltration` | `filtration` | Advanced prepared sources replacing `ThresholdRips` and `SparseRips`; no public simplex queries |
| `FlagFiltration` | `filtration` | Existing supplied-graph source with permanently absent missing edges |
| `Execution<'c>` | `execution` | Optional immutable work/cancellation settings; counters remain private |

`SimplicialComplex` and `WeightedGraph` remain storage types. A bare complex
is not proof of original Rips completeness and is not accepted as an exact-Rips
analysis source. Its own `.persistence()` analyzes only the supplied topology;
see [filtered complexes](../guides/filtered-complexes.md). `SimplicialFiltration` adds that mathematical context without
copying the stored topology. Its constructors remain controlled by construction;
callers cannot manufacture coverage or exhaustion certificates.

A prepared `RipsFiltration` represents a filtration through its graph and rules;
it does not store all its simplices. The explicit result has the same source
semantics plus stored simplices. This distinction belongs in their first rustdoc
paragraphs. No `SimplexTree` name is used without that storage implementation.

Use `UpperCamelCase` types and descriptive `snake_case` methods/variables. Keep
existing names when their contracts remain appropriate. Do not rename a type
solely to make it resemble another language or to match a directory name.

## Configuration and execution contract

Reusable point/matrix builders have value-style setters (`self -> Self`) and
borrowed execution methods. This supports storing a configured value in a local
variable, using it for both tasks and retaining immutable settings during execution.
Conditional configuration reassigns the builder. This is a deliberate tradeoff:
mutable setters are also idiomatic, but returning a borrowed temporary builder
from a chained configuration is easier to misuse. No input data is cloned by a
setter. Repeating an operation may repeat preparation; there is no hidden cache.

| Owner | Configuration | Meaning/default |
| --- | --- | --- |
| Exact construction | `max_edge_length(f64)`, `full_range()` | Inclusive finite nonnegative original edge cutoff; uncapped by default |
| Approximate construction | Required `epsilon`, `MetricPolicy` | Explicit approximation and hypotheses; existing validity and underflow checks; epsilon >= 1 has no approximation bound |
| Approximate construction | `max_filtration_value(f64)`, `full_range()` | Cap/uncap modified approximate edge values, not original distances |
| Approximate construction | `start_vertex(usize)`, `min_insertion_radius(f64)` | Existing deterministic start convention and zero radius default; preserve retained-subset provenance |
| Analysis | `max_homology_dimension(usize)` | One by default, including lower dimensions |
| Analysis | `field(PrimeField)` | F2 by default; no narrowing of prime-u32 characteristics |
| Analysis | `max_filtration_value(f64)`, `available_range()` | Analyze within available source coverage; clearing an analysis cap does not widen the source |
| Analysis | `representatives(&[RepresentativeRequest])` | Borrowed requested bases; none by default; results own their payload |
| Execution | `max_work(u64)`, `cancellation(&AtomicBool)` | Unlimited work and no cancellation flag by default |

Repeated setters replace the prior setting. Numeric setters store configuration;
terminals validate the final settings before expensive work where possible.
Validated views, fields and representative requests retain their existing
construction checks. Approximation setters do not start sampling. Scalar errors
precede input-dependent feasibility checks; bounds established by preparation
are checked before reduction. No partial success is returned on failure.

| Terminal | Result | Advanced counterpart |
| --- | --- | --- |
| `build_complex(max_simplex_dimension)` | Owned `SimplicialFiltration` | `build_complex_with(max_simplex_dimension, &Execution)` |
| `compute()` on an analysis request | Owned `PersistenceResult` | `compute_with(&Execution)` |
| `prepare()` on a construction builder | Owned exact/approximate prepared source | `prepare_with(&Execution)` |

There is no unqualified construction `build()` and no independent expansion
request object. `max_simplex_dimension` is required at complex construction;
`max_homology_dimension` belongs to analysis. The positional construction argument
is a single required choice with a documented parameter name; it removes a public
builder that previously existed only to hold that value. If expansion later gains
real independent configuration, reconsider its shape with actual caller examples.

Default terminals delegate to their controlled counterparts with default execution.
Do not multiply methods for fields, representatives or input layout. Operational
settings stay out of introductory examples. `max_work` counts documented work,
not seconds, bytes or a cross-version performance score. No `max_simplices`, hard
memory cap, backend selector, timeout or thread setting is introduced speculatively.

## Approximation and supplied graphs

Approximation uses the same two terminal workflows with explicit hypotheses:

```rust,ignore
use cocycle::filtration::ApproximateRipsBuilder;
use cocycle::geometry::MetricPolicy;
use cocycle::persistence::PersistenceExt;

let approximate_rips =
    ApproximateRipsBuilder::from_points(points, 0.5, MetricPolicy::Assume)
        .max_filtration_value(2.0);
let filtration = approximate_rips.build_complex(2)?;
let result = approximate_rips
    .persistence()
    .max_homology_dimension(1)
    .compute()?;
```

`from_distance_matrix(matrix, epsilon, policy)` is the matrix alternative.
`Assume` records an assumption, not evidence that metric inequalities were checked.
Expansion and analysis both preserve blockers, sampling provenance and original
vertex mapping. Direct computation does not discard the blocker or silently
select approximation. Sparse input storage is a different concept from this
approximation and does not select it automatically.

`FlagFiltration::new(graph)` consumes an already validated `WeightedGraph`.
The resulting source supports `build_complex(dimension)` and `.persistence()`.
It needs no pass-through builder. Missing edges are absent at every scale;
this says nothing about distances in an unknown original point cloud. The explicit
result preserves flag-specific context instead of returning bare topology.

## Advanced preparation and callback ownership

Preparation preserves existing graph reuse without making it mandatory:

```rust,ignore
let prepared = rips.prepare()?;
let result = prepared.persistence().field(field).compute()?;
let filtration = prepared.build_complex(2)?;
```

A prepared source owns its graph and source metadata, plus the blocker for an
approximation. Repeated analyses and explicit builds can reuse preparation.
The prepared types support no `find`, simplex iterator or on-demand simplex-value
method. Existing raw graph access remains available for graph-level callers;
extracting it and constructing a `FlagFiltration` changes the mathematical source,
especially for an approximation. It does not certify original-input coverage.
This is reuse of computation, not an intermediate simplex inspection interface.

Custom distance callbacks keep one-shot ownership. Factories
`RipsBuilder::from_distance_fn(items, distance)` and the approximate counterpart
with epsilon/policy return `RipsCallbackBuilder` and
`ApproximateRipsCallbackBuilder`. These own `FnMut(&P, &P) -> Result<f64>` and borrow
items. Setters consume/return `Self`; `prepare` and `build_complex` consume the
builder. Both also have controlled counterparts. No automatic `Clone` or borrowed
`.persistence()` is supplied for these builders.

```rust,ignore
let prepared = RipsBuilder::from_distance_fn(items, distance)
    .max_edge_length(1.0)
    .prepare()?;
let result = prepared.persistence().compute()?;
```

Exact preparation evaluates each unordered pair once, validates even values above
the cutoff and retains O(n+m) graph storage. Approximate callback preparation
caches O(n²) pair values once before sampling/metric checks, preserving current
behavior for a stateful function. `build_complex` includes that same preparation
and expansion without exposing the intermediate object. Errors propagate and
cancellation is polled around callback calls. Analyses of a prepared source never
replay its callback. This advanced exception avoids hiding stateful input semantics
behind a supposedly reusable request.

## Architecture and extension boundaries

Keep `geometry/complex -> filtration -> persistence -> result assembly` as the
responsibility map for these Rips workflows, with consumers depending on their
inputs. It does not require every algorithm to materialize those stages: a direct
persistence algorithm may fuse preparation and reduction. `execution` lives below
construction and computation and imports neither. Filtration implementation files
do not import persistence; explicit construction terminals do not perform reduction.

`PersistenceExt` lives in `persistence` and is implemented for supported library
sources. Its sized-source method returns `PersistenceBuilder<'_, 'static, Self>` before representative requests. It borrows
the source, stores settings and performs no work. The source/request borrow must
not escape its owner; computed results are independent of both.

Trait import is a usability cost, not an architectural defect by itself. Retain
one explicit `use cocycle::persistence::PersistenceExt` in all examples. Do not add
a prelude, an application facade or a second namespace factory solely to hide that
line. Verify external-crate method discovery before stabilizing the API.

Source/engine dispatch remains private. GUDHI's open template concepts provide
more external customization than this initial sealed interface; that is a conscious
scope limit, not claimed superiority. Open a Rust source trait only after a real
external implementation establishes its mathematical, ownership and error contract.
Storage and engines must still be separable internally. No oversized trait should
force specialized F2 H1 and generic cohomology to expose the same workspace.

Coverage and source provenance now live in
`filtration/provenance.rs` and the approximation domain. `diagram` consumes these
facts and assembles `ComputationContext` with field and analysis range. Separate
filtration kind from input layout instead of expanding a cross-product enum.
Existing import paths and getters preserve their information. The legacy
`FiltrationKind` cross-product enum is retained during compatibility; separating
its public axes is deferred to the breaking revision.

A new filtration family supplies its own construction and source adapter when
needed; it is not forced through Rips clique enumeration. Avoid duplicating public
function families merely for another layout or optional output under the same
mathematical contract. A specialized algorithm with different preconditions,
parameters or result semantics may have a focused entry point independently of
default Builder dispatch. The [kernel design](kernel.md#algorithm-implementations-and-default-integration)
owns that contribution policy and the implemented result contracts.
Common `PersistenceData`, explicit computed-dimension sets and internal
[source-context assembly](kernel.md#source-facts-and-result-assembly) are implemented
locally. The [result API migration](kernel.md#result-api-migration) describes their
pre-release semantic and signature changes; legacy Rips entry points remain available.
The [extension boundaries](kernel.md#extension-boundaries) distinguish current
static cell adaptation and in-crate contributions from future external result
import and runtime plugins. This page describes the current Builder workflow
and compatibility stage.
Read-only explicit topology remains the current scope. Dynamic editing, new
filtration families, collapse operations and language bindings are separate work.

## Execution and error invariants

Each terminal call creates one fresh private budget spanning all its phases:
pair visits, metric checks, sampling, retained-edge processing, expansion,
reduction and requested representatives as applicable. Internal composition cannot
reset the budget. Separately invoked terminals intentionally use separate budgets.
Concurrent calls have independent workspaces; no hidden shared preparation cache.

Define counted construction checkpoints before implementing them: distance reads,
individual metric inequalities, sampling updates, simplex extension candidates and
stored incidences. Charge before the counted action; rejected candidates still
count. Sorting/allocation are phase-level cancellation checkpoints, not hard time
or memory accounting. A running callback is only interruptible at surrounding
checkpoints. Input validation already performed before the terminal is outside
its budget. Tests must cover failure recovery and shared multi-phase accounting.

Preserve exact scales, inclusive cutoffs, duplicate/zero-distance vertex identity,
original IDs, approximation hypotheses and subset target. Analysis cannot widen
coverage or resample an approximation merely because its analysis cap changes.
A direct path must keep construction and analysis caps separately in result context.

| Failure | Behavior |
| --- | --- |
| Invalid numeric settings/field/request | Existing structured parameter error; validate before preparation where knowable |
| Callback or metric validation failure | Original/structured error; no partial successful source |
| Analysis beyond available scale | `Error::IncompleteFiltration` |
| Explicit construction lacks necessary cofaces | `Error::InsufficientSkeleton`; no false essentiality claim |
| Budget exhausted/cancellation | Structured failure across preparation and computation |
| Size/index/allocation failure | Existing checked failure where supported; no empty-result substitute |
| Unsupported request | Explicit failure; no silent field, output or filtration change |

A missing construction dimension is not a runtime error: the function requires
that argument. No new `MissingParameter` variant is needed for this design.

## Implementation scope and migration

Implemented responsibilities (no placeholder modules):

| Location | Responsibility |
| --- | --- |
| `src/execution/mod.rs` | Move reusable controls below construction/computation; private shared budget |
| `src/filtration/rips/builder.rs` | Exact settings and callback ownership; named construction/preparation terminals |
| `src/filtration/rips/approximation/builder.rs` | Approximate settings and callback terminals using existing kernels |
| `src/filtration/expansion/mod.rs` | Shared explicit result and private construction coordination; no public expansion builder |
| `src/filtration/provenance.rs` | Coverage and source-context ownership |
| `src/persistence/builder.rs`, `src/persistence/source.rs` | Analysis request, public sealed extension and private dispatch |
| Existing exact/approximate/flag modules and exports | Adapt construction/results and prepared-type names without duplicating algorithms |
| Tests, examples, guides and native workers | Migrate complete caller tasks and keep measured boundaries accurate |

Delivery proceeds in reviewable stages: shared execution contract; both exact/flag
workflows with explicit metadata; approximation and callbacks; caller migration;
then removal of superseded APIs in a declared pre-release breaking revision.
Existing function families remain temporary tested adapters. Do not deprecate a
capability before its replacement works. No legacy public API is removed in this compatibility stage.

| Current entry/type | Target |
| --- | --- |
| Legacy `rips_from_*`, `RipsOptions` | `RipsBuilder` then `.persistence().compute()`; adapt old matrix views and explicitly discard context only when required |
| `compute_rips_from_points`, `compute_rips_from_distances` | Same direct workflow with explicit point/matrix constructor |
| `threshold_rips_from_*`, `threshold_rips_with_distance` | Exact builder/callback `.prepare()` for callers actually reusing the prepared graph |
| `ThresholdRips` | `RipsFiltration` with equivalent coverage and owned graph |
| Existing `expand(dimension)` | `build_complex(max_simplex_dimension)` returning contextual explicit topology |
| `RipsExpansion`, `SparseRipsExpansion`, bare flag expansion result | `SimplicialFiltration` |
| `compute_threshold_rips`, `compute_flag`, `compute_expanded_*` | `.persistence().compute()` on the corresponding source |
| `SparseRipsOptions`, `sparse_rips_from_*`, `sparse_rips_with_distance` | `ApproximateRipsBuilder` reusable/callback settings; choose complex construction, direct analysis or advanced preparation |
| `SparseRips`, `compute_sparse_rips` | `ApproximateRipsFiltration` and `.persistence().compute()` |
| `_with_representatives` functions, `PersistenceOptions` | Setters on the same analysis request |
| `ExecutionLimits` | Optional `Execution`; document the expanded whole-operation scope |

Temporary adapters retain old result and work-limit semantics. New names can
coexist with `expand(dimension)` during migration without Rust method overloading.
Aliases are appropriate only for unchanged contracts. The proposal introduces no
permanent parallel `build`, `expand` and `build_complex` entry points for the same
operation. Benchmark workers retain their measurement boundaries, and maintained
reports change only after measuring the actual new code.

## Acceptance before implementation is called complete

- Compile the two primary workflows as external callers. Cover stored and chained
  builders, conditional configuration, repeated execution, required trait imports,
  request lifetimes, source drops and owned outputs.
- Cover points, all matrix layouts, prepared graphs, supplied flags, exact and
  approximate callbacks, every metric policy, F2/odd primes, higher dimensions,
  empty/duplicate inputs and requested representative bases.
- Compare direct and sufficient explicit results using existing independent
  fixtures. Check dimensions, coverage, approximation metadata and representative
  payload, not just interval counts. Preserve implicit/sparse memory behavior.
- Verify compile-time absence of simplex queries on unexpanded sources and absence
  of callback replay through borrowed requests. Check invalid settings, insufficient
  skeletons, scale limits, cancellation, multi-phase budgets and recovery at runtime.
- Verify default and explicitly default execution agree. Run the applicable native
  exact/sparse differential suites and performance protocols after migration; keep
  report paths stable and generated evidence outside Git.

Design acceptance also asks whether a caller can identify the output of each
terminal without reading internals. Do not add a public type merely because a
private phase needs a struct, or a public method solely to expose an optimization.

[g-rips]: https://gudhi.inria.fr/doc/latest/class_gudhi_1_1rips__complex_1_1_rips__complex.html
[g-persistence]: https://gudhi.inria.fr/doc/latest/class_gudhi_1_1persistent__cohomology_1_1_persistent__cohomology.html
[g-python]: https://gudhi.inria.fr/python/latest/rips_complex_ref.html
[rust-builders]: https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder
[command]: https://doc.rust-lang.org/std/process/struct.Command.html
[regex]: https://docs.rs/regex/latest/regex/struct.RegexBuilder.html
[rayon-builder]: https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html
[rayon-iter]: https://docs.rs/rayon/latest/rayon/iter/trait.IntoParallelIterator.html
