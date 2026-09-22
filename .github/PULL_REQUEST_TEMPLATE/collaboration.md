<!-- Optional collaboration template. Keep the existing default template intact. -->
<!-- Target proffitteoy/cocycle-rs:main from a feature branch. -->
<!-- Follow CONTRIBUTING.md and .github/CODEX_REVIEW.md. -->

## Problem and resulting behavior

Describe the concrete problem, resulting behavior and scope.
State any mathematical/public API change and intentional exclusions.
Name the affected public function/type, implementation owner and protecting test.
For a new capability, identify the added API rather than imply it existed on main.

## Affected contract

Keep only applicable items; a documentation PR need not fill mathematical fields.

- Path: legacy `RipsOptions` / `PersistenceOptions` / threshold Rips / supplied
  flag / sparse approximation / representatives / diagram consumer / new distance.
- Inputs and output: point or matrix layout/supplied graph, field, dimensions,
  edge-length cutoff, complete versus censored coverage, diagram or representatives.
- Invariant or intentional change: ordering/ties, interval multiplicity/endpoints,
  sparse blocker/mapping/bound, ownership, cancellation or numeric error behavior.
- New distance only: bottleneck or Wasserstein order/internal norm; diagonal,
  essential and censored input policy; exported API and independent expected result.

## Validation

List commands actually run with results. Distinguish local, hosted, oracle and
performance evidence; label pending or unavailable checks explicitly.

| Evidence | Result, revision and accessible artifact |
| --- | --- |
| Applicable CONTRIBUTING checks and actual hosted CI head/run | |
| Existing regression or independent expectation (path/test name) | |
| Exact VR/flag/matrix: compare_rips.py, GUDHI + Ripser | |
| Sparse approximation: compare_sparse_rips.py, GUDHI; Ripser exclusion and exact regressions | |
| New distance: actual harness command, GUDHI + Topp versions/results | |
| Performance only: baseline/candidate SHAs, protocol and equivalent work | |

Use N/A with a reason for unaffected areas. For comparisons, state field,
dimensions, precision, cutoff/coverage, multiplicity and metric convention.
For performance, identify the baseline, equivalent work and retained regressions.
Machine-local artifacts alone are unavailable to a cloud reviewer.
Use the maintained command map in .github/CODEX_REVIEW.md and check this head's
actual workflow. GUDHI/Ripser native CI alone does not establish Topp agreement
or a speedup. Do not invent a distance command.

## Compatibility and risks

State endpoint, approximation, ownership, dependency/MSRV and API implications.
List remaining risks or verification gaps. Identify any necessary edits to
shared product documentation. Keep local planning and machine configuration out
of the PR; attach reproducible evidence or accessible artifacts instead.
Check source rustdoc, contract tests, relevant examples, mathematical specification
and unreleased changelog when public behavior changes; explain N/A where suitable.

## Review and handoff

Use the review prompt in .github/CODEX_REVIEW.md when requesting review.
Record the reviewed head and follow-up resolution. Merging and onward transfer
remain with the user; this PR does not request an upstream submission or release.
