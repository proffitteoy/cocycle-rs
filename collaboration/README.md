# Collaboration workspace

This directory owns our contribution plans, investigations, agent rules and
review/submission process for the shared Cocycle repository. Production work is
Rust and stays in the existing source tree. Shared product documentation retains
its original ownership; this index intentionally lives outside `docs/`.

| Start here | Purpose |
| --- | --- |
| [Agent rules](AGENTS.md) | Scope, ownership, evidence and submission constraints |
| [Phase 1](phase-1.md) | VR optimization and native Rust Topp-compatible distances |
| [Validation](validation.md) | GUDHI plus Ripser for VR; GUDHI plus Topp for distances |
| [Git workflow](workflow.md) | Branch, commit template, own-repository PR and handoff |
| [Codex review](review.md) | Cloud prerequisite, exact trigger, focus and review template |
| [Commit message template](templates/commit-message.txt) | Default commit editor content |
| [PR template](../.github/PULL_REQUEST_TEMPLATE/collaboration.md) | GitHub-recognized optional collaboration template |
| [Initial repository investigation](research/topp-ripser-assessment.md) | Dated source comparison and local experiment evidence |

The only discovery bridges outside this directory are the root `AGENTS.md`, the
optional GitHub PR template and the collaboration-document check workflow.
They do not replace the existing root README, contributing guide, default PR
template, source architecture or main CI workflow.

## Working boundary

The confirmed submission repository is
[proffitteoy/cocycle-rs](https://github.com/proffitteoy/cocycle-rs).
Work goes from a feature branch to a PR against its `main`; the user handles
later synchronization. Initialization is not permission to merge or release.

Start from [workflow](workflow.md), choose a bounded item from [phase 1](phase-1.md),
and attach the applicable [validation](validation.md) evidence. The historical
investigation predates the Rust-first distance decision; use the phase plan as
the active scope instead of its earlier external-bridge recommendation.

## Local documentation check

```sh
python3 collaboration/check.py
```

This extends existing repository link/encoding checks to this directory without
changing the shared checkers. It does not run Rust algorithms or certify cloud
review settings. Source-level review rules are discovered through root
`AGENTS.md`; a nested file alone would not govern changes under `src/`.
