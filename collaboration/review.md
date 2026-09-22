# Codex GitHub review

[Collaboration workspace](README.md) / [Submission workflow](workflow.md)

This is the repository-specific review playbook and prompt template. Root
`AGENTS.md` contains the applicable `Code Review Rules`; its explicit link here
keeps detailed instructions centralized without relying on a nested file to
govern unrelated `src/` paths.

## Cloud prerequisite and current verification boundary

Official OpenAI documentation was checked on 2026-09-22:

- [Review GitHub pull requests with Codex](https://developers.openai.com/codex/integrations/github/)
- [Custom instructions with AGENTS.md](https://developers.openai.com/codex/guides/agents-md/)

The target repository must be connected to Codex cloud and have Code review
enabled in [Codex review settings](https://chatgpt.com/codex/settings/code-review).
Select `proffitteoy/cocycle-rs`. Automatic reviews are optional; the desired
workflow here is a user-requested review on a PR.

Repository instructions and templates do not enable that account/repository
setting. During initialization the browser connection was unavailable, so its
current state could not be inspected or changed. Do not mark the integration
active until the repository setting and an actual review response are observed.
No extra GitHub Action, API-key secret or scheduled review is installed here.

## Request template

Post this as a **PR comment**, after replacing the focus and evidence fields:

```text
@codex review

Follow root AGENTS.md and collaboration/review.md. Review the current PR diff
against its actual base, read-only; do not edit, commit, push or merge.

Focus: <VR / diagram distance / shared contracts / collaboration tooling>.
Check the mathematical contracts and the applicable reference matrix in
collaboration/validation.md. Separate demonstrated defects from missing evidence
and existing issues. Give precise changed-file/line evidence for each finding.

Evidence: <actual CI run/artifact links, or explicitly unavailable>.
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

## Review procedure

1. Establish repository, base/head SHA and diff. Read root instructions, this
   playbook, the affected project contracts and relevant call sites/tests.
   For local review, inspect dirty state separately from the committed PR diff.
2. Classify the change using [validation](validation.md). Follow affected data
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

- Filtration values/order, equal-value tie handling and oriented boundaries;
  forward H0 classification and reverse cohomology must use compatible ordering.
- H0/H1 optimized dispatch versus generic dimensions/fields; preserve killing
  cofaces, clearing topology, pivot ownership and coefficient arithmetic.
- Implicit/explicit and matrix/graph consistency. An absent supplied edge is
  different from a threshold-truncated exact-Rips edge.
- Finite, essential and right-censored endpoints; inclusive cutoff, computed
  dimensions, zero-bar policy, duplicates and interval multiplicity.
- Approximation blockers, metric hypotheses, original vertex mapping and bound
  target. A graph-only Ripser comparison cannot validate blocker topology.
- Cancellation, size/overflow and work accounting; no silent partial success.
  Representative requests have different work and cannot be timed as diagram-only.
- GUDHI/Ripser identity and shared precision. Performance evidence must compare
  equivalent work and retain adverse cases; an optimized source shape is not a
  measured speedup.

## Distance-specific focus

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

For a human-maintained review record in this directory, use:

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
