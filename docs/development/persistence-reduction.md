# Contributing a persistence algorithm

[Documentation](../README.md) / Development / [Contribution paths](algorithm-contributions.md)

Start with the algorithm's mathematical input and output. A boundary-matrix
researcher can work directly on ordered columns; an implicit cohomology researcher
can work on coface access. Neither implementation calls the default Builder.
This guide covers in-crate contributions. The internal types and functions below
are reusable implementation tools, not a stable external reducer API.

## Find the relevant computation

| Mathematical work | Implementation | Input and output |
| --- | --- | --- |
| Forward column reduction | [algebra/reduction/boundary.rs](../../src/algebra/reduction/boundary.rs) | Owned columns over a prime field to reduced columns, pairings and optional transformations |
| Ordinary intervals from selected boundaries | [persistence/boundary](../../src/persistence/boundary/mod.rs) | Selected ordered columns with dimensions/values to a diagram |
| Zero-born H0 | [flag/h0.rs](../../src/persistence/flag/h0.rs) | Vertex count and weighted edges to merge intervals |
| Specialized implicit F2 H1 | [flag/cohomology](../../src/persistence/flag/cohomology/mod.rs) | Ordered edge/triangle access to ordinary intervals |
| Prime-field implicit cohomology | [simplicial/cohomology.rs](../../src/persistence/simplicial/cohomology.rs) | Zero-born coface access to ordinary intervals |

The [mathematical specification](../reference/mathematics.md#4-persistence-and-reference-boundary-reduction)
defines orientation and persistence pairing. Verify the applicable assumptions
before sharing an implementation: zero-born union-find pairing does not implement
arbitrary vertex-birth persistence, and a sparse Rips blocker cannot be replaced
by ordinary clique enumeration.

## Work directly on boundary columns

The algebra reducer consumes its columns. Its pivots and transforms belong to
that invocation. It knows no point cloud, source context, diagram type or Builder.
It accepts a checkpoint callback; maintainers connect that callback to the
operation's existing work budget instead of starting another budget.

`reduce_pairs` retains reduced columns and death partners without basis
transformations. `reduce` additionally returns V with D V = R. Nonzero reduced
columns have unique normalized pivots. The `deaths[i]` entry associates birth
column i with its death column. An empty reduced column is a birth candidate;
later columns may pair it, so do not declare it essential during the forward scan.

For ordinary interval assembly, `boundary::diagram` takes owned `BoundaryInput`,
maximum dimension, field, established coverage and the current budget. Readers
select the q+1 skeleton first. `BoundaryInput` holds ordered IDs, dimensions,
values and boundary columns; its shared append operation keeps these arrays in
step. It contains no source certificate or Builder settings. This helper declares
the contiguous output domain `0..=q` used by current default adapters.

The two existing readers establish their guarantees differently:

- [filtered.rs](../../src/persistence/filtered.rs) validates external cell IDs,
  ordering, selected-field incidence and the boundary-square condition.
- [simplicial/input.rs](../../src/persistence/simplicial/input.rs) reuses frozen
  simplicial construction guarantees and converts stored oriented incidence.

An algorithm with a different column representation, traversal, preparation or
workspace can own it. It can use the algebra reducer without using `BoundaryInput`,
or implement another reducer. Shared ordinary output types do not require shared
internal storage. For H1-only or gapped outputs, declare `ComputedDimensions`
explicitly rather than using a contiguous maximum as an availability claim.

## Derive a small case before integrating

Consider three vertices born at different signed scales, then their edges and a
delayed face. Increasing vertex order fixes orientation:

| Column | Cell | Value | Boundary |
| --- | --- | --- | --- |
| 0 | a | -4 | 0 |
| 1 | b | -3 | 0 |
| 2 | c | -2 | 0 |
| 3 | ab | 0 | b - a |
| 4 | ac | 1 | c - a |
| 5 | bc | 2 | c - b |
| 6 | abc | 3 | bc - ac + ab |

The complete diagram has H0 intervals `[-4, infinity)`, `[-3, 0)` and `[-2, 1)`,
and one H1 interval `[2, 3)`. These endpoints follow from component merges and
the delayed filling; they are not obtained from a second production solver.

[boundary/tests.rs](../../src/persistence/boundary/tests.rs) constructs these
columns directly and calls the computation without a source adapter. It also
checks D V = R using independent dense modular arithmetic, normalized unique
pivots, field-dependent cellular incidence and recovery after work exhaustion.
The existing test-only [reference reducer](../../src/persistence/reference/mod.rs)
remains independent; do not reuse production elimination to generate its answers.

Once the local mathematics works, this public composition checks source integration:

```rust
use cocycle::{Result, algebra::PrimeField, complex::{Simplex, SimplicialComplex}};
use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use cocycle::persistence::PersistenceExt;
let cells = [
    (vec![10], -4.), (vec![20], -3.), (vec![30], -2.),
    (vec![10, 20], 0.), (vec![10, 30], 1.), (vec![20, 30], 2.),
    (vec![10, 20, 30], 3.),
].into_iter().map(|(vertices, value)| Simplex::new(vertices, value))
 .collect::<Result<Vec<_>>>()?;
let complex = SimplicialComplex::new(cells)?;
let result = complex.persistence().field(PrimeField::new(3)?).compute()?;
let expected = PersistenceDiagram::new(1, Coverage::Complete, vec![
    PersistenceInterval::new(0, -4., IntervalEnd::Essential)?,
    PersistenceInterval::new(0, -3., IntervalEnd::Finite(0.))?,
    PersistenceInterval::new(0, -2., IntervalEnd::Finite(1.))?,
    PersistenceInterval::new(1, 2., IntervalEnd::Finite(3.))?,
])?;
assert_eq!(result.diagram(), &expected);
# Ok::<(), cocycle::Error>(())
```

## Connect a default operation when appropriate

`source.rs` handles supported sources and context. `flag/dispatch.rs` selects H0,
specialized F2 H1 or general cohomology for exact flag inputs; algorithms never
call back into it. Explicit and approximate Rips adapters call simplicial
cohomology directly. `simplicial::finish_zero_born` implements the existing choice
between an implicit diagram and materialized simplicial representatives, only
for compatible zero-born access. It is not a mandatory pipeline for new algorithms.

Keep capability selection explicit. Unsupported fields, dimensions, source
coverage or witnesses must not silently change the requested operation. Reuse
mathematical parameters when they fit. Add a specialized public entry only when
users need a meaningful choice or a different operation; an internal optimization
does not require another Builder or registry.

Maintainers assemble library-produced `PersistenceData` from the diagram and
actual source context. Compatible results can compose it with their own auxiliary
output. Existing `Representative` means persistent simplicial cycles or their
query-scale dual cocycles; arbitrary cellular witnesses need their own semantics.
Keep source certificates, requested settings and the actual computed domain
distinct. See [result composition](../design/kernel.md#results-carry-enough-information-to-be-interpreted).

Run the focused check while developing:

```sh
python3 tools/check_algorithm.py persistence-reduction
```

It runs direct persistence algorithm tests, independent Rips oracle comparisons,
filtered-input/field/representative/resource integration tests, the existing flag
example and this guide's code. It needs no C++ installation. Maintainers complete
the [full checks](../../CONTRIBUTING.md#verification) and relevant pinned native
comparisons before merge. Timing evidence follows the benchmark protocol; ordinary
tests do not assert machine-dependent performance thresholds.
