# Phase 1: Rust VR optimization and diagram distances

[Collaboration workspace](README.md) / [Validation](validation.md)

This is our collaboration plan, not a claim that new kernels are implemented.
The first phase has two deliverables: improve Vietoris-Rips (VR) computation and
bring Topp's supported diagram-distance capabilities into this Rust library.
Use this directory for our plans, review material and concise evidence reports.
Keep shared project documentation under its existing ownership; propose any
necessary public-contract documentation changes explicitly with the feature PR.

## Boundaries and starting point

The initialization baseline is Cocycle commit
`0cfc7b5cf280644aa60a0f92a96a6bb0f04c42bf` (observed 2026-09-22).
It already provides exact Rips/flag persistence, dimension-generic prime fields,
representatives, sparse approximation and result coverage metadata. Consult the
[architecture](../docs/development/architecture.md) and
[mathematical specification](../docs/reference/mathematics.md) for current behavior.
Diagram-distance computation is a new capability, not an existing Rust API.

- Production implementation is Rust, edition 2024, with the current Rust 1.91
  minimum. Preserve the existing prohibition on unsafe code and absence of
  external runtime dependencies unless a concrete, separately reviewed change
  establishes otherwise.
- GUDHI, Ripser and Topp are independent comparison implementations. Their
  Python/C++ adapters belong to verification infrastructure, outside the core.
- Topp's stable Python interface does not establish a stable C++ ABI. The prior
  external Python interoperability experiment is evidence of compatible data
  semantics, not the selected production integration architecture.
- Port algorithms and contracts deliberately. Verify licenses and preserve
  required attribution before copying implementation text.
- Work in small feature branches and PRs targeting
  [proffitteoy/cocycle-rs](https://github.com/proffitteoy/cocycle-rs). Do not push
  directly to main, merge the PR, or send it to upstream on the user's behalf.

## Track A: improve VR computation

Start with a reproducible profile of one public workload and a specific cost:
construction, cofacet enumeration, reduction, retained storage or export.
The existing H0/H1 native suite and whole-workflow pipeline measure different
scopes; retain that distinction in every before/after comparison.

1. Select a representative family and a constrained optimization hypothesis.
   Record exact baseline revision, input hashes, output scope and environment.
2. Establish agreement with independent mathematical expectations, GUDHI and
   Ripser before changing the algorithm; use [validation](validation.md).
3. Make the smallest implementation change in the existing owner. New data
   structures need a measured benefit; do not prebuild a general backend system.
4. Verify all affected public paths and failure recovery, then measure the same
   work on the same fixtures. Retain regressions and inconclusive results.
5. Submit the implementation and concise evidence in a focused PR. A smoke run
   establishes exercised correctness, not a general speedup.

Every optimization must preserve filtration ordering and ties, cutoff inclusion,
coefficient-field arithmetic, interval multiplicity, zero-length policy,
dimension coverage, essential versus censored endpoints, representative meaning,
and resource/cancellation errors. A failed or incomplete computation must not
become an apparently successful partial result.

Sparse storage and sparse approximation are distinct. The latter includes
modified edge values and higher-simplex blockers; substituting a flag expansion
changes the construction. Preserve its sampling, metric assumptions and
approximation provenance. Record Ripser's construction exclusion explicitly and
continue the separate exact-Rips regression suite.

## Track B: bring in Topp-compatible distances

Implement native Rust diagram-distance capability using the following initial
mathematical targets. Topp remains an external oracle; do not make the tested
Rust implementation and its only expected-value generator share solver logic.

| Capability | Required initial semantics |
| --- | --- |
| Bottleneck | Exact bottleneck distance with pointwise L-infinity cost |
| Wasserstein W1 | Order 1, pointwise L-infinity cost |
| Wasserstein W2 | Order 2, pointwise L2 cost, including the final square root |

"Exact" excludes algorithmic approximation; it does not remove floating-point
roundoff. Other Wasserstein parameter combinations are outside this initial
scope. Prepared diagrams, batch APIs and threshold queries can follow when a
concrete consumer and evidence justify them; do not create placeholder APIs.

Before exposing the first function, write its input, error and numeric contract
beside the implementation and review how it uses the existing diagram types:

- Preserve repeated intervals and distinguish an empty computed dimension from
  an uncomputed dimension. Compare each homology dimension separately.
- Match finite points to each other or the diagonal with the selected norm.
  Preserve essential-point categories and counts, and specify incompatible
  essential counts consistently with the selected Topp/GUDHI contracts.
- Reject right-censored intervals for a full-diagram distance. Replacing their
  unknown death by the cutoff or positive infinity is mathematically different.
- Retain and check relevant field, filtration-scale, coverage and approximation
  context when converting a `PersistenceResult`; raw diagrams alone cannot
  establish compatible provenance.
- Specify NaN, invalid endpoint, infinity, diagonal-point, overflow and extreme
  magnitude behavior. Do not silently drop points or reduce precision.

Implement bottleneck first, then W1 and W2 in reviewable increments. Each
capability needs hand-derived examples, an independent tiny exhaustive matching
oracle, GUDHI and Topp comparisons, meaningful boundary tests and a runnable Rust
example. Select the smallest sufficient implementation; share code only when
numeric and ownership contracts actually agree.

The maintained distance comparison adapter is still to be implemented. It must
pin versions, state GUDHI backend/precision settings, retain raw results, and
exercise real Rust outputs. Existing files under `target/topp-assessment/` are
exploratory evidence, not an installed dependency or a maintained test command.

## Acceptance and sequencing

Initialization establishes documents, agent/review instructions and submission
templates only. It does not ship either optimization or distance algorithms.
The first implementation PR should choose one bounded track item and identify
its mathematical and measurable acceptance criteria before coding.

A track item is complete only when its relevant checks in
[CONTRIBUTING.md](../CONTRIBUTING.md) and [validation](validation.md) pass, required
reference comparisons have retained evidence, public behavior is documented,
and the PR records exclusions and remaining limits. Missing native prerequisites,
pending hosted checks or unavailable artifacts remain incomplete evidence.
Do not declare the entire first phase complete after one workload improves or
one distance function passes a small smoke suite.
