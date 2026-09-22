# Repository review instructions

Existing project contracts in `CONTRIBUTING.md`, `docs/` and source rustdoc still
apply. Keep changes focused and preserve unrelated work.
Use the actual reviewed head's `Cargo.toml` and `src/lib.rs` for supported Rust,
dependencies and public capabilities; a plan or another branch is not an API.

## Code Review Rules

Follow [.github/CODEX_REVIEW.md](.github/CODEX_REVIEW.md) for a formal PR review.
Its code/test map identifies the existing owners and regressions to inspect.

- Flag VR changes that silently alter fields, filtration ordering, multiplicity,
  computed dimensions or cutoff/coverage semantics. A speedup must preserve the
  declared mathematical problem; unsupported oracle cases are exclusions.
- Preserve the distinction between `ThresholdRips` original-input coverage and
  `FlagFiltration` permanently absent edges. Internal cone stopping cannot turn
  a caller's truncated result into complete coverage. Sparse Rips blockers must
  survive implicit/explicit paths; its graph alone does not define the complex.
- Check `PersistenceInterval` and `PersistenceDiagram` contracts before adding
  consumers: signed scales and arbitrary dimensions are valid representations,
  repeated intervals remain distinct, and an empty computed dimension is not
  an uncomputed dimension. Do not apply Rips-only restrictions globally.
- Flag distance changes that confuse W1-infinity with W2-Euclidean, discard
  repeated/essential points, or convert right-censored deaths to finite values
  or infinity. Reject unsupported input explicitly instead of repairing it.
- Assess the actual base-to-head diff and trace affected callers. Report concrete
  consequential defects with changed-file/line evidence; distinguish unrun
  GUDHI/Ripser/Topp comparisons from failures and from verified results. Review
  is read-only unless the user explicitly requests fixes.
