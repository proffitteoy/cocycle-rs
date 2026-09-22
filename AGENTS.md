# Repository review instructions

Existing project contracts in `CONTRIBUTING.md`, `docs/` and source rustdoc still
apply. Keep changes focused and preserve unrelated work.

## Code Review Rules

Follow [.github/CODEX_REVIEW.md](.github/CODEX_REVIEW.md) for a formal PR review.

- Flag VR changes that silently alter fields, filtration ordering, multiplicity,
  computed dimensions or cutoff/coverage semantics. A speedup must preserve the
  declared mathematical problem; unsupported oracle cases are exclusions.
- Flag distance changes that confuse W1-infinity with W2-Euclidean, discard
  repeated/essential points, or convert right-censored deaths to finite values
  or infinity. Reject unsupported input explicitly instead of repairing it.
- Assess the actual base-to-head diff and trace affected callers. Report concrete
  consequential defects with changed-file/line evidence; distinguish unrun
  GUDHI/Ripser/Topp comparisons from failures and from verified results. Review
  is read-only unless the user explicitly requests fixes.
