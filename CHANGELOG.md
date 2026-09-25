# Changelog

## 0.1.0 (unreleased)

- Initialize F2 H1 coboundaries in one pass with reusable safe buffers. Omit
  stored zero-lifetime apparent pairs and reconstruct their transformations when
  later columns need them. Avoid allocating discarded zero-lifetime H1 intervals.
  Preserve public APIs, f64 ordering, coverage and interval multiplicity; extend
  independent-oracle, transformation-replay and intermediate resource-failure
  checks.

- Add reusable exact/approximate Rips builders with explicit `build_complex`
  construction and a shared `.persistence().compute()` analysis workflow. Preserve
  implicit engines, fields, representatives, callback ownership and source coverage.
- Add contextual `SimplicialFiltration` for exact, approximate and supplied-flag
  expansion. Keep simplex queries on explicit topology; move source provenance
  below result assembly while retaining existing import paths.
- Add cooperative `Execution` controls spanning whole builder operations, including
  preparation and expansion. Legacy functions/options remain compatibility entry
  points with their existing control scope. Migrate guides, examples and native
  workers without changing benchmark protocols or previously measured results.

- Remove all repository-local historical experiment outputs and reports. Keep
  generated data in ignored local directories or external artifacts; enforce
  artifact admission in CI and bind concise future reports to measured commits.

- Add phase-separated native Rips workflow measurements, per-process memory
  evidence, failure retention and an R1-R10 acceptance audit. Cover cancellation
  and work-budget recovery across twelve paths and concurrent prime-field calls.
- Reuse overflow-safe condensed pair counting in sparse callback construction;
  distinguish numerical overflow/underflow from invalid nonfinite input.

- Add deterministic sparse Rips approximation with explicit metric hypotheses,
  insertion-radius provenance, perturbed edges and higher-simplex blockers.
  Support implicit/explicit prime-field persistence and requested representatives
  with original vertex IDs. Add a runnable example and native GUDHI C++ checks.

- Add borrowed lower/upper/square matrix views, streamed exact threshold graph
  construction from Euclidean or custom distances, and checked weighted graphs.
- Share dense/sparse flag H0/H1 computation over F2 with neighbor-intersection
  cofacets. Preserve existing Rips APIs and distinguish original-input censoring
  from essential classes in a supplied graph's complete filtration.
- Add owned computation context, cooperative work limits/cancellation, graph
  examples, independent graph-oracle tests and pinned native C++ correctness
  workers.
- Add frozen simplicial expansion, lookup, oriented boundaries and cofacet queries.
  Keep construction dimension and scale coverage separate in `RipsExpansion`;
  reject insufficient skeletons before reporting Rips persistence.
- Extend `PersistenceOptions` and rich computation entry points to arbitrary F2
  homology dimensions using implicit coboundary reduction with clearing. Retain
  the legacy `RipsOptions` H0/H1 contract and specialized H1 engine.
- Add the H2 sphere example, H2/H3/H4 analytic and independent boundary-oracle
  tests, and native C++ comparisons of expanded simplices and high-dimensional
  interval multisets.

- Add validated prime-u32 fields with overflow-safe modular arithmetic and
  oriented implicit cohomology. `PersistenceOptions::with_field` selects the
  field; F2 remains the default and the specialized H1 path remains available.
- Add opt-in `_with_representatives` entry points for all exact input paths.
  Results own persistent cycle bases and their query-scale dual cocycles, with
  local interval identities preserving multiplicity. Ordinary calls retain
  implicit computation without materializing a representative skeleton.
- Add field-sensitive flag RP2 fixtures, independent modular rank and basis
  verification, a representative example, and native prime-field comparisons.
  Record upstream coefficient limits explicitly instead of narrowing moduli.

- Consolidate code conventions and change-specific verification rules. Enforce
  source hygiene and Python syntax in CI, validate Markdown reference links,
  and keep native build probe output in its build directory.
- Establish native GUDHI C++ and upstream Ripser C++ benchmark workers with pinned
  sources, shared fixtures, per-sample validation and process memory records.
  Separate the current protocol and native sources from historical wrapper reports.
- Organize documentation into usage guides, mathematical reference, development,
  design, and upstream research. Add task-based navigation and preserve recursive
  link checking, guide examples, and documentation packaging.
- Group Rips options and algorithms in a private module, extract shared
  connectivity, and distinguish edge positions from transformation-column
  positions. Preserve public entry points and update private profiling paths.
- Organize geometry, diagrams, and descriptors into consistent domain directories
  with private implementation files. Clarify internal filtration and result
  assembly names while preserving public paths and mathematical behavior.
- Maintenance: clarify H1 transformation indices and shortcut handling, share
  result normalization at the public entry point, and fix a strict Clippy warning
  in the test reference. Public APIs and mathematical conventions are unchanged.
- Repository organization: isolate the test-only reference algorithm and developer
  profiling, retain public API paths, and provide English user/contributor/math
  documentation with a local-link check in CI.

Initial pure Rust implementation, with no runtime dependencies. Requires Rust 1.91.

- Validated borrowed Euclidean point clouds and condensed symmetric dissimilarities.
- Ordinary Vietoris–Rips H0/H1 persistence over F2, with closed edge-length cutoffs.
- Independent union-find H0 and implicit Rips H1 persistent cohomology; retain
  explicit sparse-column boundary reduction as a test oracle.
- Checked combinatorial indexing, on-demand cofacets, H0 clearing, implicit
  change-of-basis columns, apparent/initial-column emergent shortcuts and a
  cone stopping bound that preserves public coverage and censoring semantics.
- Owned persistence diagrams retaining multiplicity and distinguishing finite,
  essential and right-censored intervals.
- Finite lifetime summaries, natural-log persistence entropy and Betti curves.
- Mathematical contracts, independent rank and stability checks, optional Ripser
  comparison, a runnable example and reproducible performance benchmarks.
- Bounded scaling experiments, analytic bipartite-filtration regression checks,
  explicit timeout/omission records and private workload diagnostics for H1.

H1 avoids constructing the full 2-skeleton; repeated enumeration and reduction
fill-in still limit scalability. Diagram distances, other filtrations and Polars bindings are future work.
