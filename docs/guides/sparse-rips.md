# Sparse Rips approximation

[Documentation](../README.md) / Guides

Sparse Rips changes the filtration to reduce its size. This differs from storing
an exact threshold graph sparsely. Its higher simplices obey an insertion-radius
blocker; ordinary flag expansion of the resulting graph can give different
homology. The [mathematical specification](../reference/mathematics.md#15-sparse-rips-approximation)
states the construction and conditional theorem.

## Construct and compute

```rust
use cocycle::filtration::ApproximateRipsBuilder;
use cocycle::geometry::{MetricPolicy, PointCloudView};
use cocycle::persistence::PersistenceExt;

let coordinates = [0., 0., 1., 0., 1., 1., 0., 1., 0., 0.];
let points = PointCloudView::new(&coordinates, 5, 2)?;
let construction = ApproximateRipsBuilder::from_points(points, 0.5, MetricPolicy::Check);
let result = construction.persistence().compute()?;
let metadata = result.context().approximation().unwrap();
assert_eq!(metadata.retained_vertices(), &[0, 1, 2, 3]);
assert_eq!(metadata.bound().unwrap().factor(), 2.0);
# Ok::<(), cocycle::Error>(())
```

Run `cargo run --example sparse_rips` for F3 persistence, a requested H1 cycle
and its dual cocycle. The [representative guide](rips-representatives.md) describes
scale-specific basis semantics. Sparse representatives belong to the approximate
filtration; their original vertex labels do not establish a correspondence to
exact Rips classes at the same scale.

Use `ApproximateRipsBuilder::from_distance_matrix` for any validated matrix layout,
or `from_distance_fn` for a one-shot unordered-pair callback. Callbacks must be
consumed by `prepare()` before analysis, or directly by `build_complex(dimension)`. The callback runs once
per pair with the smaller index first, declares symmetry/zero diagonal, and
caches its values. Matrix inputs are borrowed without a copy. Euclidean distances
are evaluated as needed without retaining a dense matrix.

## Parameters and hypotheses

The builder constructors require finite positive epsilon and an
explicit metric policy. `Check` exhaustively verifies the triangle inequality on
the stored binary64 values, without a tolerance; zero-distance duplicates are
allowed. `Assume` records the caller's metric assertion. `Unchecked` constructs
the filtration but reports no approximation bound. Rounded Euclidean distances
are not silently certified as a metric; strict checking may reject roundoff-level
violations. Inspect or deliberately assume the hypothesis for your data.

The default first vertex is original ID zero; `start_vertex` overrides it.
Farthest-point ties choose the smallest original ID. Full permutation and insertion
radii are retained, including omitted points. The first radius is `None` (infinity).
The default empty input is valid; an explicit start on an empty input is not.

`min_insertion_radius(r)` retains the initial point and the greedy prefix
whose positive insertion radii are at least `r`. Equality is inclusive. Noninitial
zero-radius points are always omitted. Positive-radius omissions change the
conditional theorem's target to the retained subset. Metadata reports its actual
covering radius; it does not claim the same multiplicative bound to the full input.

`max_filtration_value(t)` retains modified edge values at most `t`, in edge-length
units. `Coverage::Complete` concerns this approximate filtration only. An absent
sparse edge never enters. The construction reports `Through(t)` if any otherwise
admissible edge was removed by this cutoff. Computation can narrow the range,
but cannot extend an incomplete construction beyond its known threshold.

For `0 < epsilon < 1` and checked/assumed metric data, `bound()` reports the
nominal factor `1 / (1 - epsilon)` and its target. This is the ideal-arithmetic
theorem, **not a certified bound on floating-point numerical error**. Epsilon at
least one is accepted with GUDHI-style ordinary expansion of its modified graph,
but `bound()` is `None`. An underflowed blocker factor or overflowing edge
intermediate returns `NumericalFailure`. Rescale extreme distances if needed;
no silent infinity comparisons or relaxed metric tolerances select topology.

## Inspect topology and reuse computation

`construction.build_complex(max_simplex_dimension)` applies the blocker and
returns `SimplicialFiltration`. Only this explicit result exposes simplex queries
through `.complex()`. Simplex vertices use original IDs. Zero constructs vertices
only; `is_dimension_complete()` certifies exhaustion within the scale range.

The explicit result supports `.persistence()` over stored incidence. Hq requires
construction through q+1 unless expansion proved exhaustion. Both workflows accept
`.field(...)` and `.representatives(...)` on the same analysis request. They share
oriented reduction and result types with exact Rips; no dense-cone or ordinary
flag H1 shortcut bypasses the sparse blocker.

Advanced `.prepare()` returns an owned `ApproximateRipsFiltration` for repeated
analysis or construction. Its raw graph uses compact IDs;
`approximation().retained_vertices()[compact_id]` gives the original ID. It offers
no on-demand simplex query interface. Existing `SparseRips` names are compatibility
imports during migration.

## Costs and reproducibility

Greedy sampling and edge candidates take O(n²) distance evaluations with O(n)
sampling workspace; metric checking takes O(n³). Euclidean evaluations also cost
coordinate dimension. The graph uses O(n+m) storage; cached custom callback
values add O(n²). Expansion and representative requests can require exponentially
many simplices. These bounds do not promise a smaller complex for every epsilon
or dataset, or a certified memory ceiling.

Builder controlled terminals share one budget across construction, expansion or
analysis as applicable; legacy free functions retain their older scope. Controls
are cooperative, not hard memory or time caps. Failure exposes no partial result. Source-independent result
metadata owns the full sampling provenance, adding O(n) to approximate results.

The [native comparison protocol](../../tools/README.md#native-sparse-rips-checks)
compares GUDHI's C++ edges, blockers, filtered simplices and prime-field diagrams
with fixed sampling semantics. Upstream Ripser has no matching approximation
constructor. Exact-Rips comparisons against Ripser remain a separate suite.
