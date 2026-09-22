<!-- Optional collaboration template. Keep the existing default template intact. -->
<!-- Target proffitteoy/cocycle-rs:main from a feature branch. -->
<!-- Follow collaboration/workflow.md and collaboration/validation.md. -->

## Problem and resulting behavior

Describe the concrete problem, resulting behavior and phase-one scope.
State any mathematical/public API change and intentional exclusions.

## Validation

List commands actually run with results. Distinguish local, hosted, oracle and
performance evidence; label pending or unavailable checks explicitly.

| Evidence | Result, revision and accessible artifact |
| --- | --- |
| Rust / MSRV / docs, as applicable | |
| Independent expected result | |
| GUDHI for mathematical changes | |
| Ripser for VR changes | |
| Topp for distance changes | |

Use N/A with a reason for unaffected areas. For comparisons, state field,
dimensions, precision, cutoff/coverage, multiplicity and metric convention.
For performance, identify the baseline, equivalent work and retained regressions.
Machine-local artifacts alone are unavailable to a cloud reviewer.

## Compatibility and risks

State endpoint, approximation, ownership, dependency/MSRV and API implications.
List remaining risks or verification gaps. Identify any necessary edits to
shared product documentation; keep our planning/reporting in collaboration/.

## Review and handoff

Use the review prompt in collaboration/review.md when requesting review.
Record the reviewed head and follow-up resolution. Merging and onward transfer
remain with the user; this PR does not request an upstream submission or release.
