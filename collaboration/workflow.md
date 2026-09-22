# Branch, commit and PR workflow

[Collaboration workspace](README.md)

The user confirmed `origin`, `proffitteoy/cocycle-rs`, as the submission target.
It is itself a GitHub fork; use that exact repository for PR creation instead of
letting a tool infer the fork network's parent. The user handles onward transfer.

## Start a change

1. Read the root and collaboration agent rules plus the affected project docs.
2. Inspect status, branch, remotes and ongoing work. Preserve unrelated changes.
3. Start a focused branch from the agreed base, normally current `origin/main`.
   Use `codex/<short-topic>` for Codex work; do not work directly on main.
4. Keep one reviewable concern per PR. Separate mathematical changes, performance
   changes and unrelated cleanup. Coordinate with pending upstream API work.

Example from a clean checkout, substituting an actual topic:

```sh
git status --short
git remote -v
git fetch origin
git switch -c codex/<topic> origin/main
```

Do not run that switch over unrelated uncommitted work; retain it or use an
isolated worktree. Branch names in examples are placeholders, not shell input.

## Commit message contract

Use [the template](templates/commit-message.txt) for every commit, including
noninteractive commits made with `-F` or `-m`. Use an English imperative subject
of the form `type(scope): summary`. Suggested types are `feat`, `fix`, `perf`,
`test`, `docs`, `chore`; useful scopes include `vr`, `distance`, `oracle`,
`collaboration`, and `ci`. Prefer a subject of at most 72 characters.

The body records the concrete problem/result, actual validation and remaining
risks. Use `not run (reason)` for unrun checks. For algorithm or benchmark work,
include oracle identities/evidence pointers and scope, not unsupported speedups.
Do not include raw logs, sensitive local paths, credentials or speculative claims.

Enable the editor template for this checkout only; this does not modify global
Git settings and must be repeated for a fresh clone:

```powershell
$repoRoot = git rev-parse --show-toplevel
git config --local commit.template "$repoRoot/collaboration/templates/commit-message.txt"
git config --local --get commit.template
```

The template assists writing; Git does not enforce it for `-m` or `-F`. Agents
must fill the same fields explicitly. No commit hook is installed by this setup.

## Validate and submit

1. Run the change-specific checks in [validation](validation.md) and
   [CONTRIBUTING](../CONTRIBUTING.md). Run `python3 collaboration/check.py` when
   our material changes. Review the complete diff.
2. Stage explicit paths only. Inspect `git diff --cached` and run
   `python3 tools/check_artifacts.py` after staging.
3. Commit using the template and inspect the resulting SHA and file list.
4. Verify `git remote get-url --push origin` targets `proffitteoy/cocycle-rs`,
   then push the named feature branch. Never push main or add a force option.
5. Open a PR explicitly in `proffitteoy/cocycle-rs`, with `base=main` and the
   pushed feature branch as head. Use the [collaboration PR template](../.github/PULL_REQUEST_TEMPLATE/collaboration.md).
6. Inspect hosted checks for that exact head. Mark missing or failed evidence
   visibly; a local pass does not substitute for hosted CI.
7. Hand the PR URL and validation/limitations to the user. Leave merging,
   release and onward synchronization to the user.

For GitHub CLI, write the completed body to an ignored file first. A typical
creation command, after replacing all placeholders, is:

```sh
gh pr create --repo proffitteoy/cocycle-rs --base main \
  --head codex/<topic> --title '<type>(<scope>): <summary>' \
  --body-file target/<completed-pr-body>.md
```

Never inline a long description through shell interpolation. To use the optional
web template after it reaches the default branch, append
`?template=collaboration.md` to the repository's compare/PR creation URL. The
existing default PR template is preserved. On the initialization PR, supply the
body explicitly because the new optional template is not yet on main.

## Review and update

The user can post the exact trigger from [review.md](review.md) after the cloud
repository has Code review enabled. Do not post a review request or comments as
the user unless asked. A plain `@codex` mention is not the review trigger.
Make requested fixes as focused follow-up commits on the same branch, rerun the
affected checks, and bind new evidence/review to the new head SHA. Avoid rewriting
already reviewed history unless specifically requested.
