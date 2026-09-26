# Contributing an explicit complex constructor

[Documentation](../README.md) / [Algorithm contributions](algorithm-contributions.md)

This path is for researchers turning data into simplices and filtration values.
It uses the existing immutable `SimplicialComplex`, its queries and the persistence
engine. Rust 1.91 or later, rustfmt, Clippy and Python's standard library are enough
for this walkthrough. No native C++ installation is required.

## Start from a concrete construction

The [runnable example](../../examples/complex_construction.rs) implements a
lower-star filtration on supplied simplicial topology. A finite scalar function
on vertices determines each simplex's value by taking the maximum over its
vertices. The definition, monotonicity argument and hand-derived circle are in
[mathematics section 18](../reference/mathematics.md#18-lower-star-construction-example).

This example constructs a filtration on a given complex. It does not infer a
triangulation from points or construct Rips, Alpha or cubical topology. The local
`lower_star_complex` function is teaching code, not a new public library API.

| Input | Contract |
| --- | --- |
| `vertex_values` | Finite scalar values; the slice index is the vertex ID; all vertices are included, even isolates |
| `nonvertex_simplices` | Every edge and higher-dimensional simplex exactly once, including all their nonvertex faces |
| Each vertex list | Strictly increasing IDs, each indexing `vertex_values`; no repeated vertices |
| Output | A validated immutable complex, ordered by filtration, with lookup and oriented incidence |

The vertex values may be negative or equal. Topological input order is arbitrary.
The function adds the listed vertices, assigns values to the supplied nonvertex
simplices, then calls `SimplicialComplex::new`. Missing edges of a triangle and
duplicate simplices are errors; neither the example nor the storage constructor
repairs them. A production topology-generating algorithm must enumerate the
required faces itself, with an explicit policy for shared faces.

## Read the implementation in three parts

1. Validate vertex values by creating one `Simplex` per vertex. This also preserves
   isolated vertices. The count/allocation check is integration code, kept adjacent
   to the owned output buffer.
2. For each supplied nonvertex simplex, check that its vertices exist and compute
   their maximum value. `Simplex::new` checks the vertex ordering. This loop is the
   mathematical algorithm; it uses slices, loops, a vector and `Result`.
3. Pass the collection to `SimplicialComplex::new`. Shared storage validates closure
   and monotonicity, sorts simplices, and creates incidence. Reuse it rather than
   writing another lookup table, boundary representation or reducer.

Assigning values takes O(V + L) time and O(V + L) output space, where V is the
number of vertices and L is the total number of vertex occurrences in supplied
nonvertex simplices. Freezing the complex additionally sorts and builds incidence;
these bounds do not describe the complete constructor. The example clones each
supplied vertex list into owned storage and has no zero-copy claim.

## Inspect before computing persistence

The runnable example prints the four-cycle's simplices, an edge boundary and its
persistence intervals. Use these queries during algorithm development:

| Query | What to inspect |
| --- | --- |
| `simplices()` | Generated vertex sets and their filtration values, in filtration order |
| `find(&[...])` | A simplex's local handle; vertices must be in increasing order |
| `simplex(id)` | The vertex list and value behind a handle |
| `boundary(id)` | Oriented codimension-one faces |
| `cofacets(id)` | Stored codimension-one cofaces |

`SimplexId` identifies a position in this particular frozen complex. It is not an
original vertex label or a stable handle shared between independently built
complexes. Equal filtration values put faces before cofaces; rely on that contract
rather than hard-coding IDs from a particular tie order.

This small complete example isolates oriented incidence from any constructor:

```rust
use cocycle::complex::{Simplex, SimplicialComplex};

let complex = SimplicialComplex::new(vec![
    Simplex::new(vec![10, 30], 0.0)?,
    Simplex::new(vec![30], 0.0)?,
    Simplex::new(vec![10], -2.0)?,
])?;
let edge = complex.find(&[10, 30]).unwrap();
let terms: Vec<_> = complex.boundary(edge).unwrap().iter()
    .map(|term| (complex.simplex(term.face).unwrap().vertices()[0], term.coefficient))
    .collect();
assert_eq!(terms, [(30, 1), (10, -1)]);
assert!(complex.find(&[30, 10]).is_none());
# Ok::<(), cocycle::Error>(())
```

Once the topology and values are correct, `complex.persistence().compute()` uses
the existing engine; import `PersistenceExt` to obtain that method. The example
tests use multiple prime fields and a negative cutoff. No persistence engine
modifications or custom `FilteredComplex` implementation are required.

## Test the construction itself

The example's four colocated tests execute the actual `lower_star_complex`
function, not a copied implementation:

- The four-cycle has independently derived finite H0 and essential H0/H1
  intervals. A cutoff before the components merge leaves two censored H0 classes.
- A filled triangle with all vertex values tied has the signed boundary
  `[1,2] - [0,2] + [0,1]`, boundary squared zero over the integers, the expected
  cofacets and no positive-lifetime H1 class over F3.
- Empty inputs and isolated vertices remain valid and distinct.
- Invalid vertex references, nonfinite values, repeated/unsorted vertices,
  duplicate simplices and missing faces are rejected.

These are necessary checks for this example, not a universal validation suite
for all constructors. A new family needs its own geometric or combinatorial
oracle, degeneracy cases and approximation guarantees. A matching persistence
diagram alone does not prove that the generated complex is correct.

## Keep source meaning explicit

A supplied `SimplicialComplex` is the entire mathematical source. An unfilled
circle has an essential H1 class because no triangles ever enter that source.
It cannot certify that a larger construction would also leave the loop unfilled.
For certified Rips builder results, `SimplicialFiltration` additionally records
construction coverage and dimension sufficiency; preserve that wrapper when
analyzing the original source. See the [source-coverage guide](../guides/filtered-complexes.md#supplied-topology-versus-certified-construction).

For a future Alpha constructor, maintainers must also integrate units, vertex
mapping, degeneracy policies and construction coverage. This walkthrough does
not add Alpha or claim those problems are solved by a common storage type.

## Integrate with a maintainer

Keep algorithm-specific generation and value rules together in a named module
under `src/filtration/` when adding a production family. Reuse `src/complex/` for
storage and put independently reusable geometric operations in `src/geometry/`
only when their contracts warrant it. The mathematical author supplies the
algorithm, hypotheses and expected results; maintainers help with public exports,
source metadata, resource policies, compatibility and broader verification.

The example uses ordinary allocation for individual vertex lists and provides no
cancellation hook. `SimplicialComplex::new_with` controls its own validation work,
not the algorithm's earlier generation loop. Production resource integration
must cover generation as well; it cannot be accomplished by wrapping only the
last call. No new builder or public trait is needed for this teaching example.

Use the [focused construction command](../../CONTRIBUTING.md#focused-algorithm-checks)
to run formatting/lint checks, the example's algorithm tests, existing filtered
complex contract tests, the example and tutorial code. The
[full verification policy](../../CONTRIBUTING.md#verification) still applies before
merging a production algorithm. Relevant external comparisons remain a separate
responsibility; this example's analytic tests are not a GUDHI/Ripser parity claim.
