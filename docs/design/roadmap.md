# Roadmap

[Documentation](../README.md) / Design

This document records priorities, not a delivery schedule. Supported behavior is
listed in the [user guide](../guides/rips.md); completed user-visible work belongs in the
[changelog](../../CHANGELOG.md).

## Direction

Build a native Rust TDA kernel with GUDHI's C++ capabilities as a reference and
specialized Rips computation where appropriate. Prioritize coherent input and
result contracts, composable analysis operations, ownership, and resource
behavior. Runnable Rust examples demonstrate the kernel; language bindings and
application frameworks are not first-stage deliverables.

The [GUDHI C++ study](../research/gudhi-cpp.md) maps upstream capabilities and remaining
reading work. The [kernel design](kernel.md) proposes responsibility boundaries.
The [Rips subsystem design](rips.md) defines the selected complete Rips target,
including upstream Ripser, its API draft and acceptance gates. These are design
inputs, not claims of implemented parity or a request to copy either source layout.

## Current scope

The implemented core provides borrowed point clouds and matrix layouts, exact
threshold graph construction, supplied weighted flag filtrations, ordinary prime-field
persistence through dense/sparse access, explicit simplicial expansion and
incidence queries, requested cycle/cocycle bases, blocker-aware sparse approximation,
owned diagrams/context and basic descriptors.
Cooperative work limits and cancellation are available on the richer compute
entry points. An independent explicit boundary implementation remains a test oracle; the
production representative path owns a separate reducer. The
crate has not been published. See the [construction guide](../guides/rips-construction.md).

## Next priorities

The [API design](rips-api.md) is implemented with a compatibility stage:
`RipsBuilder::build_complex` creates inspectable topology and `.persistence()`
configures direct analysis. Approximation, callback ownership, source context and
whole-operation execution controls follow the same contracts. Guides and native
workers use the new paths. Legacy names and functions remain available; removing
them requires a separately declared pre-release breaking revision. Algorithm
optimizations and new filtration families remain separate review units.

1. **Complete the Rips subsystem.** Follow the
   [delivery sequence and exit gates](rips.md#delivery-sequence-and-exit-gates):
   exact sparse input through computation, explicit and higher-dimensional Rips,
   prime fields and requested representatives, then sparse approximation through
   computation. Stages 1-4 are implemented, including sparse approximation
   with hypotheses, blocker rules and provenance. Stage 5 has an [acceptance audit](rips-acceptance.md), resource
   regressions and native workflow measurements. Hosted CI for the submitted
   revision remains a release gate. Review new API contracts before extending scope.
   Each stage delivers a usable path with tests, examples and documentation;
   finishing one stage does not complete the whole target.
   The [implementation scope](rips-implementation.md) identifies exact directories,
   existing-file edits and review units for the first stage and later additions.
2. **Extend native evidence with each capability.** The
   [native suites](../../benches/README.md) compare pinned GUDHI and upstream
   Ripser C++ workers. Exact and sparse correctness suites now cover construction,
   higher dimensions, fields and approximation; the pipeline records phase and
   end-to-end resource snapshots. Extend those fixtures with each new contract,
   following the [verification matrix](rips.md#verification-and-native-comparison)
   and [reporting rules](../../benches/reporting.md). Stronger performance claims
   need a fresh comparative study with externally retained evidence.
3. **Retain focused H1 performance work.** The scoped sequence below can improve
   the existing path while broader capabilities arrive. Include nonmetric and
   bipartite fixtures alongside geometric inputs; performance claims need fresh
   native evidence.
   Review enumeration/reduction optimizations separately from API and storage
   refactors.

This Rips-first direction supersedes the earlier suggestion to add a diagram
representation or scalar-line operation next. Those remain future library
capabilities; they are not substitutes for completing Rips. The selected target
includes ownership, resources, reproducibility and result interpretation, not only
an expanded constructor list. See [R1-R10](rips.md#required-capability-matrix).

The first crates.io release remains a separate readiness gate: verify the name
and publisher, confirm hosted CI for the release commit, and inspect the package
using the [release procedure](../../CONTRIBUTING.md#release-procedure). This planning
work does not publish a crate or commit to a release date.

Performance-only changes to existing paths must preserve pivot order, F2 parity,
multiplicity, and public coverage. New fields and generalized algorithms establish
their own documented invariants. Check each optimization against the reference and external
implementations; do not remove hard cases or use point count as a universal limit.

## H1 implementation sequence

Keep maintenance refactors separate from changes to enumeration, reduction, or
storage. The current baseline has a single result-normalization path, named
transformation columns, and an isolated original-column shortcut search. Its
test-only reference and seven optimization configurations remain the correctness
checks for subsequent work.

| Order | Work | Evidence required |
| --- | --- | --- |
| 1 | Reuse cofacet enumeration when the original-column shortcut fails, including empty columns | Bipartite cutoff cases retain every censored interval; tie cases still fall back when the earliest pivot is owned; measure candidate scans as well as yielded cofacets |
| 2 | Compress pending working-heap entries by F2 parity | Preserve pivot order and odd multiplicities; measure circle heap peaks and the cost on small/easy inputs |
| 3 | Reduce repeated cofacet generation during transformation-column reconstruction | Nonmetric cases retain complete diagrams; measure regeneration counts together with added cache/storage costs |

For each step, run the reference and external comparisons, then compare the same
fixtures under the same precision and timing boundaries before and after the
change. Include uniform, circle, nonmetric, and full/truncated bipartite inputs.
Keep timeouts and regressions in the results. Candidate-vertex scans are currently
absent from the cofacet counter; zero yielded cofacets does not mean zero work.

Historical benchmark records describe the source at measurement time. A replay
against their saved diagrams checks agreement with those outputs; it is neither
a fresh external-library run nor a new performance baseline. Record new source
hashes and fresh measurements before making a speed or memory claim.

Dense computations and the legacy point entry point still retain a full distance
buffer. New threshold point construction and sparse computation avoid that buffer.
H0 edge sorting remains a separate scaling cost.
Sparse inputs and higher dimensions are now explicit Rips deliverables, with
their own API and validation gates rather than being folded into these H1
optimizations. Other mathematical capabilities remain separate work.

## Feature admission

A new capability needs a mathematical specification, clear input/output contracts,
an independent correctness check, and a reasonable ownership/error model.
Avoid speculative backend registries, unused traits, placeholder modules, and
application-specific research features. Language bindings remain separate from
the kernel's Rust API and release process.
