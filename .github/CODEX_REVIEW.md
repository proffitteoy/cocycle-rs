# Codex GitHub review

[Repository review instructions](../AGENTS.md)

This is the repository-specific review playbook and prompt template. Root
`AGENTS.md` contains the applicable `Code Review Rules`; its explicit link here
keeps detailed instructions centralized without relying on a nested file to
govern unrelated `src/` paths.

This playbook is fork-local review configuration for `proffitteoy/cocycle-rs`.
It does not require local planning files or a machine-specific checkout.
The existing project contribution and mathematical contracts remain authoritative.

## Current code and review routing

This map was checked against committed baseline
`0cfc7b5cf280644aa60a0f92a96a6bb0f04c42bf`. Recheck the reviewed head's
[exports](../src/lib.rs) and [manifest](../Cargo.toml): at that baseline this is
one Rust 2024 crate, MSRV 1.91, no runtime dependencies, with unsafe code forbidden.
It implements Rips/flag persistence and descriptors; it has no public diagram
distance API or maintained Topp comparison command. An uncommitted implementation
or a separate builder proposal is not part of this baseline. New capabilities
must be evaluated from their actual PR code and public contracts.

Read the affected owners and tests rather than applying every row to every PR:

| Affected area | Current implementation owners | Existing contract/oracle tests |
| --- | --- | --- |
| Diagram consumers | [intervals](../src/diagram/interval.rs), [diagram/coverage](../src/diagram/persistence_diagram.rs), [context](../src/diagram/computation.rs) | [contracts](../tests/contracts.rs), [descriptors](../tests/descriptors.rs) |
| Input layouts and distances | [matrix views](../src/geometry/matrix.rs), [Euclidean norms](../src/geometry/euclidean.rs) | [matrix](../tests/matrix.rs), [input contracts](../tests/contracts.rs) |
| Exact Rips versus supplied flag | [Rips range/entry points](../src/persistence/rips/mod.rs), [flag dispatch](../src/persistence/flag/mod.rs) | [construction](../tests/rips_construction.rs), [supplied flag](../tests/flag.rs) |
| H1 and generic reduction | [F2 cohomology](../src/persistence/flag/cohomology/mod.rs), [generic dimensions/fields](../src/persistence/flag/cohomology/dimensions.rs), [simplex order](../src/complex/simplicial/simplex.rs) | [optimization variants](../src/persistence/flag/cohomology/tests.rs), [independent high-dimensional reduction](../tests/rips_expansion.rs) |
| Sparse approximation | [blockers](../src/filtration/rips/approximation/blocker.rs), [sparse access](../src/filtration/rips/approximation/access.rs), [bound metadata](../src/diagram/approximation.rs) | [sparse Rips](../tests/sparse_rips.rs) |
| Representative bases | [cycle basis](../src/persistence/flag/representatives/basis.rs), [dual solves](../src/persistence/flag/representatives/dual.rs) | [prime fields and independent basis checks](../tests/prime_fields.rs) |
| Work limits and recovery | [execution](../src/persistence/execution.rs) | [resource recovery](../tests/rips_resources.rs), [sparse bounded work](../tests/flag.rs) |

Shared result contracts matter even when the reducer is unchanged:

- `PersistenceInterval::new` allows signed finite scales and arbitrary dimensions;
  finite death must exceed birth, while censoring may equal birth. Finite endpoint
  subtraction can still overflow. Rips itself uses nonnegative edge-length scales.
- `PersistenceDiagram::new` sorts without deduplication. Complete coverage rejects
  censored intervals; truncated coverage rejects essential intervals and checks
  every endpoint against the inclusive cutoff. See
  `canonical_order_retains_multiplicity_and_ignores_input_permutation` and
  `empty_diagrams_distinguish_absent_intervals_from_uncomputed_dimensions`.
- `PersistenceResult::into_diagram` explicitly discards context/representatives;
  a raw diagram cannot establish coefficient-field or construction provenance.
  Review such conversions when a consumer relies on that information.
- Existing descriptors are consumers, not distance solvers: `betti_curve` has a
  finite, nonnegative, strictly increasing query grid and includes censored cutoff;
  `finite_lifetime_summary` excludes essential/censored intervals and uses entropy
  in nats. See their public contracts and `tests/descriptors.rs` when affected.

## Cloud prerequisite

Official OpenAI documentation was checked on 2026-09-22:

- [Review GitHub pull requests with Codex](https://developers.openai.com/codex/integrations/github/)
- [Custom instructions with AGENTS.md](https://developers.openai.com/codex/guides/agents-md/)

The target repository must be connected to Codex cloud and have Code review
enabled in [Codex review settings](https://chatgpt.com/codex/settings/code-review).
Select `proffitteoy/cocycle-rs`. Automatic reviews are optional; the desired
workflow here is a user-requested review on a PR.

Repository instructions and templates do not enable that account/repository
setting. Verify the setting and an actual review response before reporting the
integration as active. No extra GitHub Action, API-key secret or scheduled review
is required by this playbook.

## Request template

Post this as a **PR comment**, after replacing the focus and evidence fields:

```text
@codex review

Follow root AGENTS.md and .github/CODEX_REVIEW.md. Review the current PR diff
against its actual base, read-only; do not edit, commit, push or merge.

Focus: <VR / diagram distance / shared contracts / review configuration>.
Check the mathematical contracts and the applicable reference matrix in
.github/CODEX_REVIEW.md. Separate demonstrated defects from missing evidence
and existing issues. Give precise changed-file/line evidence for each finding.

Evidence: <actual CI run/artifact links, or explicitly unavailable>.
Affected owners/tests: <paths or symbols from the code map, adjusted to this head>.
```

The minimal trigger is exactly `@codex review`. A bare `@codex` mention or another
task phrase can start a cloud task instead of a review. The user posts the
request; do not send it on their behalf without an explicit instruction.
After a new push, verify which head SHA was reviewed and request another review
when needed. An old review does not cover new commits.

Official documentation currently describes GitHub review as reporting P0/P1
findings. Do not upgrade minor findings to force them into that output. Review
rules guide the service; they are not tests, branch protection or merge approval.
The structure below guides evidence and manual records, not a guarantee that
the hosted service emits an identical Markdown layout.

## Required evidence

Use [CONTRIBUTING.md](../CONTRIBUTING.md) and the
[mathematical specification](../docs/reference/mathematics.md) for existing
contracts and maintained commands. New distance capabilities must supply their
own runnable comparison harness rather than imply one already exists.

| Changed contract | Independent expectation and external comparisons |
| --- | --- |
| Exact VR construction/persistence | Hand calculation, property or independent reducer; GUDHI and Ripser on shared supported cases |
| Sparse Rips approximation | Independent sampling/blocker checks and GUDHI sparse construction; Ripser cannot validate higher-simplex blockers, so retain separate exact-Rips regressions |
| Diagram distances | Hand calculations and tiny exhaustive matching independent of production; GUDHI and Topp with aligned order/norm and essential-point semantics |
| Shared diagram/context conversion | Apply both VR and distance checks when both contracts are affected |
| Documentation/templates only | Source and local-link checks; no invented algorithm or performance evidence |

Record reference revisions, precision, field, dimensions, cutoff/coverage,
multiplicity, tolerance justification and explicit unsupported cases. Compare
equivalent work for performance and retain regressions. Missing comparisons are
evidence gaps, not proof of a bug; unsupported cases need independent validation.

Use the maintained commands below after [native setup](../benches/native/README.md),
with a fresh output directory for every run. They supplement the authoritative
[CONTRIBUTING verification list](../CONTRIBUTING.md#verification), not replace it.

| Changed behavior | Existing additional command/evidence |
| --- | --- |
| Exact Rips, graph or matrix semantics | `python3 tools/compare_rips.py --output target/rips-reference-new` |
| Sparse approximation | `python3 tools/compare_sparse_rips.py --output target/sparse-reference-new`, plus applicable exact-Rips regressions |
| Native workers/controllers/protocol | `python3 -m unittest discover -s tools -p 'test_*.py'` and `python3 tools/benchmark_native.py --quick --samples 1 --output target/native-smoke-new` |
| Workflow timing or resource accounting | `python3 tools/benchmark_rips_pipeline.py --quick --samples 1 --output target/pipeline-smoke-new` |
| Docs/templates only | Source/Markdown checks and staged artifact check in CONTRIBUTING; affected examples/doctests only |

The exact/sparse correctness CLIs do not accept `--quick` or `--samples`.
Source identities come from [sources.json](../benches/native/sources.json).
Ripser float32 and field limits require explicit exclusions for unsupported
f64/field cases; see [native correctness scope](../tools/README.md).
Native resource measurements require Linux. Do not reset a user's reference
checkout to satisfy a pin; use a dedicated pinned source directory.

At the audited baseline, [CI](workflows/ci.yml) runs Rust/platform/MSRV/package
checks and native GUDHI/Ripser smoke comparisons, with no Topp distance harness.
Inspect any changed workflow/harness at the reviewed head. A distance PR must
provide its actual command, versions and results, rather than inherit a claimed
Topp pass from unrelated native jobs.
Smoke correctness is not a performance ranking: use [reporting.md](../benches/reporting.md)
and the [native](../benches/protocol.md) or [pipeline](../benches/pipeline/README.md)
protocol as applicable. Retain adverse cases and compare identical input/output
work; point construction and representative requests are not diagram-only timing.

## Review procedure

1. Establish repository, base/head SHA and diff. Read root instructions, this
   playbook, the affected project contracts and relevant call sites/tests.
   For local review, inspect dirty state separately from the committed PR diff.
2. Classify the change using the evidence matrix above. Follow affected data
   from input/construction through reduction/results and distance consumers.
3. Check concrete correctness/compatibility risks below. Only report a defect
   when a triggering case and consequence are supported by code or evidence.
4. Inspect actual checks and artifacts for the reviewed revision. Missing local
   paths in cloud, omitted comparisons and expired artifacts are verification
   gaps. Unsupported backend cases require an explicit alternative expectation.
5. Report actionable findings first, then unresolved evidence limits if the
   service provides a summary. Do not fabricate findings to fill a template.
   No findings means no supported findings in this review, not a correctness proof.

Do not broaden an incremental review into an unrelated full-repository audit.
Formatting and deterministic lint checks belong to CI. Existing unrelated
problems should not be attributed to this PR or silently fixed during review.

## VR-specific focus

- **Range and result assembly:** `resolve_rips_range` and threshold entry points
  use the original input range; supplied flag missing edges never enter. Neither
  the largest retained edge nor internal cone stopping establishes complete Rips
  coverage. `assemble_diagram` omits zero bars and maps unpaired births through
  coverage, including H0. Relevant regressions include
  `original_range_is_not_inferred_from_the_largest_retained_edge` and
  `cone_stopping_preserves_user_coverage_and_closed_boundary`.
- **F2 H1 changes:** `compare_filtration`/`SimplexEntry` share value/dimension
  ordering with reverse colex ties. Column positions and simplex IDs are distinct.
  `find_shortcut` inspects the original column; an owned first equal cofacet must
  fall back to reduction. `pop_parity` cancels duplicates modulo two rather than
  set-deduplicating. Preserve the independently switchable optimization checks,
  including `every_three_level_four_vertex_filtration_matches_reference_with_each_optimization`
  and `heap_entries_cancel_by_parity_instead_of_set_deduplication`.
- **Generic dimensions/fields:** H0 union-find and F2 H1 dispatch do not replace
  `dimensions::compute` for higher dimensions/odd primes. Preserve oriented
  cofacets and inverse-pivot normalization. Clearing skips reductions, not
  simplices needed by later dimensions. Hq needs a q+1 skeleton unless clique
  exhaustion is certified. Check `torsion_changes_the_diagram_and_representatives`
  and `high_dimensions_match_independent_boundary_reduction`.
- **Sparse approximation:** `SparseRipsAccess` includes higher-simplex blockers;
  replacing it with a flag expansion of `graph()` changes topology. Preserve
  original vertex IDs versus compact positions and conditional bound metadata:
  unchecked metrics or epsilon >= 1 have no bound; positive-radius subsampling
  changes the bound target to the retained subset. Check
  `blocker_excludes_a_real_metric_clique_and_preserves_face_closure` and
  `noncontiguous_h1_representatives_are_closed_dual_and_owned`.
- **Representatives:** finite cycles must die with their associated interval;
  they use reduced death columns, not arbitrary birth transformations. Query-scale
  cocycles annihilate boundaries and pair with the active cycles as a dual basis.
  Repeated intervals retain distinct local `interval_index` values. Check
  `finite_cycles_die_with_their_intervals_and_paths_agree` and
  `duplicate_intervals_and_selections_keep_local_identity`; external diagram
  agreement does not validate representative vectors.
- **Storage/work:** supplied sparse graphs must not silently densify; the rich
  point path with a cutoff constructs a threshold graph, while representatives
  deliberately request extra skeleton work. Keep cancellation failures atomic
  and recoverable; cooperative limits are not hard RSS/wall-time caps. Check
  `sparse_h1_handles_many_isolated_vertices_with_bounded_work` and
  `all_rips_paths_recover_after_budget_and_cancellation_failures`.

## Distance additions: requirements for new implementations

These are the agreed Topp-compatible targets, not claims about the baseline's
existing API. Geometry's point/matrix distances are a different operation.
For a new module, review the actual exported functions, documented mathematical
domain and error behavior before applying these checks; do not demand a planned
batch/prepared API or a particular solver design.

- Preserve multiset matching and unlimited diagonal copies. Keep matching within
  the requested homology dimension and distinguish empty from uncomputed.
- Correct diagonal costs: `(death-birth)/2` for L-infinity and
  `(death-birth)/sqrt(2)` for L2. W2 accumulates squared costs before square root.
- Essential classes match only supported compatible types/counts. Censoring
  must be rejected for full-diagram distance; silently using cutoff or infinity
  changes the mathematical input.
- Validate finite/infinite/NaN forms, overflow and exact-versus-approximate
  semantics. A float64 interface does not repair earlier float32 rounding.
- Keep field, scale and approximation provenance where comparison requires it.
  Distance between approximate diagrams is not automatically distance between
  their original exact-Rips diagrams.
- GUDHI and Topp must be independent, pinned oracles with aligned metric and
  tolerance settings. Small exhaustive matching must not reuse production logic.

## Finding and review-record templates

Use this content for each supported finding; attach a tight current-diff range
when GitHub inline review is available:

```text
[P0/P1 as justified] <concrete defect>
Location: <changed path and exact line(s)>
Trigger: <input/options/call path>
Consequence: <incorrect topology, distance, behavior or material resource failure>
Evidence: <code reasoning, minimal reproducer, or an actual result>
Suggested correction: <smallest required invariant/behavior change>
```

For a human-maintained review record, use:

```text
Reviewed repository / PR:
Base SHA / head SHA:
Scope and relevant contracts:
Findings: <supported issues, or no actionable findings>
Verified evidence: <local / hosted / oracle / performance, separately>
Unverified or unsupported scope:
Follow-up resolution: <fix SHA and actual revalidation, when available>
```

Do not assert that every oracle was run just because its row appears in a PR.
Do not claim a cloud reviewer accessed `F:` paths or local `target/` results.

## Troubleshooting

If no reaction/review appears, verify Code review is enabled for the exact
repository and that the comment uses `@codex review`. Check the connected account
and repository access in the official settings UI. Record setup/authentication
failures separately from code-review findings. Do not add a competing bot/action
or request credentials in a PR as a workaround.
