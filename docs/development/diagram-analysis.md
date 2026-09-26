# Contributing diagram analysis

[Documentation](../README.md) / [Algorithm contributions](algorithm-contributions.md)

This walkthrough is for researchers implementing statistics, curves or features
from persistence diagrams. It needs Rust 1.91 or later with rustfmt and Clippy,
and Python's standard library for the focused checks. It needs no C++ reference
installation or Python TDA package. Start from intervals; persistent homology is
already computed input to this part of the library.

## The objects you need

| Object | Meaning |
| --- | --- |
| `PersistenceInterval` | One interval, with its homology dimension and birth |
| `IntervalEnd::Finite(d)` | Finite death; the interval is `[birth, d)` |
| `IntervalEnd::Essential` | Never dies in the complete supplied filtration |
| `IntervalEnd::RightCensored { through: t }` | Known alive at the inclusive cutoff `t`; eventual death is unknown |
| `PersistenceDiagram` | Validated interval multiset, computed dimensions and coverage |
| `ComputedDimensions` | Nonempty set of computed dimensions, including computed empty ones |
| `Coverage` | Complete source or a computation known only through a cutoff |

Constructors validate finite scales and endpoint consistency. Signed scales are
valid, multiplicities are preserved, and finite zero-length intervals are
rejected. `PersistenceDiagram::new(q, ...)` records every dimension through q.
`with_dimensions` accepts an explicit set, including H1-only or gapped results.
Querying an absent dimension is an error even below `max_dimension()`; use
`computed_dimensions().contains(k)` or `.iter()` instead of inferring membership.

A complete diagram rejects censored intervals. A truncated diagram rejects
essential intervals: survival at a cutoff alone cannot certify essentiality.
The library does not verify the mathematical origin of a manually supplied diagram;
the caller must declare its coverage truthfully.

```rust
use cocycle::diagram::{ComputedDimensions, Coverage, PersistenceDiagram};
use cocycle::descriptors::betti_curve;
let h1 = PersistenceDiagram::with_dimensions(
    ComputedDimensions::new(vec![1])?, Coverage::Complete, vec![],
)?;
assert_eq!(betti_curve(&h1, 1, &[0.0])?, vec![0]);
assert!(betti_curve(&h1, 0, &[0.0]).is_err()); // H0 was not computed.
assert_eq!(h1.computed_dimensions().iter().collect::<Vec<_>>(), vec![1]);
# Ok::<(), cocycle::Error>(())
```

## Define a small descriptor

As a teaching exercise, count observed finite intervals in dimension `k`:

`N_k = number of intervals in dimension k with a finite death`.

Count repeated intervals separately. Exclude essential and censored intervals;
do not impute their deaths. An empty computed dimension returns zero; an
uncomputed dimension returns an error. The scan takes O(m) time and O(1) extra
space for m selected intervals, following dimension lookup. No lifetime
subtraction is necessary, so extreme finite endpoints cannot overflow this count.

The following complete example implements that definition and checks its
endpoint and dimension rules. It is compiled and executed by the focused check.

```rust
use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use cocycle::{Error, Result};

fn finite_interval_count(diagram: &PersistenceDiagram, dimension: usize) -> Result<usize> {
    let intervals = diagram.intervals_in_dimension(dimension)?;
    Ok(intervals
        .filter(|interval| matches!(interval.end(), IntervalEnd::Finite(_)))
        .count())
}

let repeated = PersistenceInterval::new(1, -2.0, IntervalEnd::Finite(1.0))?;
let diagram = PersistenceDiagram::new(
    2,
    Coverage::Complete,
    vec![
        PersistenceInterval::new(1, 0.0, IntervalEnd::Essential)?,
        repeated,
        repeated,
    ],
)?;
assert_eq!(finite_interval_count(&diagram, 1)?, 2);
assert_eq!(finite_interval_count(&diagram, 2)?, 0);
assert!(matches!(
    finite_interval_count(&diagram, 3),
    Err(Error::DimensionNotComputed { .. })
));

let truncated = PersistenceDiagram::new(
    1,
    Coverage::Through(1.0),
    vec![
        repeated,
        PersistenceInterval::new(1, -2.0, IntervalEnd::RightCensored { through: 1.0 })?,
    ],
)?;
assert_eq!(finite_interval_count(&truncated, 1)?, 1);
# Ok::<(), cocycle::Error>(())
```

`finite_interval_count` is local tutorial code, not an additional public API.
The production `finite_lifetime_summary` already reports a finite count together
with other statistics. The local count deliberately does not compute lifetimes;
the production summary can fail if lifetime arithmetic overflows.

The Rust mechanics here are small: `&` borrows the diagram, `?` propagates a
structured error, `matches!` selects an endpoint variant, and `Result<usize>`
returns either a count or an error. An ordinary loop is equally acceptable; no
custom trait, lifetime parameter or builder is needed.

## Read a production implementation

[Lifetime statistics](../../src/descriptors/lifetime_statistics.rs) extends this
pattern with sums, maxima and entropy. Read its documented contract, then the
loop over `intervals_in_dimension`. The summary records excluded endpoint counts
so users know what was measured. Compensated summation and overflow checks keep
floating-point behavior explicit; a valid pair of endpoints does not guarantee
that their difference is representable as `f64`.

[Betti curves](../../src/descriptors/betti_curve.rs) count all live classes instead
of just finite lifetimes. Births count at their scale, finite deaths do not, and
censored classes still count at the cutoff. Queries beyond censored coverage
fail. The sorted-event implementation is an optimization of that definition;
the definition belongs in [mathematics section 7](../reference/mathematics.md#7-diagram-descriptors).

Run [diagram_analysis.rs](../../examples/diagram_analysis.rs) to see both operations
on complete and truncated diagrams, without constructing a complex:

```sh
cargo run --locked --example diagram_analysis
```

## Turn a new formula into a contribution

1. Specify which dimensions and endpoint kinds participate, the output's units,
   normalization, empty behavior and numerical limits. State how missing
   information is excluded or rejected. Do not silently invent finite deaths.
2. Implement the operation in a named file under `src/descriptors/`, borrowing a
   diagram and returning an owned result. For a small operation, start with a
   function. Matching algorithms belong in `src/diagram_distances/`; use the
   [distance contribution path](#contribute-diagram-distances) for those operations.
3. Add tests to `tests/descriptors.rs`, starting from hand-built intervals. Update
   the relevant mathematical specification and write rustdoc beside the function.
4. Work with a maintainer on the export in `src/descriptors/mod.rs`, error variants
   if needed, a usage example and changelog. Allocation and execution policies are
   integration work; numerical assumptions remain part of the algorithm review.

You should normally edit the descriptor, its tests and mathematical description.
Changing Rips, complex storage or persistence builders is unnecessary for an
operation that consumes only a diagram. No new public descriptor is introduced
by this walkthrough itself.

## Choose tests from the definition

Use [the descriptor tests](../../tests/descriptors.rs) as executable examples:

| Case or property | What it detects |
| --- | --- |
| Two equal lifetimes | Entropy is `ln(2)` in nats, not a base-two or normalized value |
| No finite intervals | Zero total and `None` entropy; exclusions remain visible |
| Repeated intervals and mixed dimensions | Multiplicity and dimension selection are preserved |
| Birth, death and cutoff queried exactly | Half-open finite intervals and inclusive censoring |
| Negative scales and translation | No accidental assumption that all births are zero |
| Positive rescaling | Lifetimes scale; normalized entropy stays unchanged |
| Extreme finite endpoints | Arithmetic failure is distinct from a valid empty result |

Use exact assertions for integer counts and exactly representable fixtures.
For logarithms or accumulated floating-point error, justify a small tolerance
from the operation; never enlarge it merely to make a failure disappear.
Construct expected values by hand or from an independent property, not by
calling another production descriptor. Shared test helpers are optional; keep
the mathematical input visible when a fixture is already short.

## Check and submit

Run the [focused command](../../CONTRIBUTING.md#focused-algorithm-checks) from a
source checkout. It checks source/documentation hygiene, domain formatting and
Clippy, public contract and descriptor tests, the runnable example, and this
tutorial's Rust code. The script stops at the first failure and leaves Cargo
output in its configured target directory. It does not run Rips computations or
native benchmark suites.

For a Rust failure, the file and line in the diagnostic identify the first place
to inspect. Preserve the failing mathematical input when seeking help. The
maintainer checklist and CI still cover release/MSRV builds, packaging and wider
regressions before merge; a focused pass is not evidence for those other checks.

## Contribute diagram distances

Matching algorithms live in `src/diagram_distances/`, independently of Rips and
complex construction. The public facade validates diagrams and context; the
bottleneck and Wasserstein modules own matching, numeric preparation and private
workspaces. Start with the [distance example](../../examples/diagram_distances.rs)
and [mathematical contract](../reference/mathematics.md#16-diagram-matching-distances).

Decide the ground metric and Wasserstein order explicitly. Preserve repeated
intervals, require complete coverage and distinguish unequal essential counts
(mathematical infinity) from failed arithmetic (an error). For example, a single
interval of lifetime two has diagonal costs one for bottleneck/W1 and sqrt(2)
for W2, including when its birth is negative:

```rust
use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use cocycle::diagram_distances::{bottleneck_distance, wasserstein_1_infinity,
    wasserstein_2_euclidean};

let first = PersistenceDiagram::new(0, Coverage::Complete, vec![
    PersistenceInterval::new(0, -2.0, IntervalEnd::Finite(0.0))?,
])?;
let empty = PersistenceDiagram::new(0, Coverage::Complete, vec![])?;
assert_eq!(bottleneck_distance(&first, &empty, 0)?, 1.0);
assert_eq!(wasserstein_1_infinity(&first, &empty, 0)?, 1.0);
assert!((wasserstein_2_euclidean(&first, &empty, 0)? - 2.0_f64.sqrt()).abs() < 1e-14);
# Ok::<(), cocycle::Error>(())
```

Raw-diagram functions assume the caller has established comparable scales and
appropriate coefficient fields. The `_results` wrappers additionally require equal
fields and declared edge-length conventions. A supplied complex has unspecified
units, so these wrappers reject it, even against another unspecified source.
After establishing a common scale, pass `result.diagram()` explicitly. This
choice discards automatic context checks; it does not rescale the values.
A certified Rips expansion retains the convention, while extracting its bare
complex discards that certificate. Physical units and normalization remain the
caller's responsibility even when both sources declare edge lengths.

The `_results` functions accept `AsRef<PersistenceData>` operands independently.
A result wrapper borrows the diagram/context data it already owns; borrowing
performs no cloning or metadata reconstruction and certifies no additional units.
For example, move library-created data with `result.into_data()` and retain it
alongside algorithm-specific output. Use `into_parts()` when keeping associated
representatives. Raw-diagram descriptors remain usable directly on `data.diagram()`.
See the [migration notes](../design/kernel.md#result-api-migration) for changed
maximum-dimension semantics and generic distance signatures.

Put public contracts and hand-derived cases in `tests/diagram_distances.rs`.
Use the private kernel tests for matching invariants and an independent exhaustive
oracle. Test ties, multiplicity, diagonal costs, essential points, numerical
boundaries and symmetry. Expected answers must not come from another production
matching path. Keep routing experiments and diagnostics out of ordinary builds.

Run `python3 tools/check_algorithm.py diagram-distances`. It checks formatting,
lint, public contracts, private kernel oracles, the example and this tutorial's
code. It needs no C++ environment. Before merging a matching-kernel change,
maintainers additionally run the pinned [native distance comparisons](../../benches/distances/README.md)
and record relevant numerical limitations. Local Rust checks alone do not certify
agreement with external implementations or establish performance rankings.
