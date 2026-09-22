# Our collaboration instructions

The root `AGENTS.md` delegates to this file for work throughout the repository.
This directory is the primary workspace for our planning and collaboration
records, while production Rust code remains in the existing project modules.

## Scope and ownership

- Default to Chinese in conversations. Keep committed project material and
  templates in English to match this shared repository's conventions.
- Read relevant existing `AGENT.md`/`AGENTS.md`, README, `docs/`, rustdoc and code
  before editing. Follow [phase 1](phase-1.md) and [validation](validation.md).
- Phase 1 is Rust VR optimization and native Rust Topp-compatible diagram
  distances. GUDHI/Ripser/Topp are independent oracles, not automatically added
  runtime dependencies. Do not turn this into a general refactor or Python app.
- Put our investigations, plans and review records in `collaboration/`. Do not
  add our navigation or progress reports to the main README, `docs/README.md`,
  upstream roadmap or contributing guide. If a feature changes a public contract,
  update the necessary source rustdoc/tests and only the directly affected shared
  documentation; explain that bounded change in its PR.
- Preserve unrelated changes and all external reference checkouts. Start with
  `git status --short`, branch/remotes and the actual diff. Never clean/reset
  another repository or overwrite frozen results to make a comparison pass.
- Prefer the smallest verified change in an existing owner. Avoid speculative
  modules, generic backend registries, dependencies and unused abstractions.

## Evidence and completion

- Mathematical changes require GUDHI. VR changes additionally require Ripser;
  distance changes additionally require Topp, following the supported-case and
  explicit-exclusion rules in `validation.md`.
- Keep a genuinely independent small oracle or hand-derived expectation. Do not
  use the new implementation as its own expected-value generator.
- Preserve fields, precision, scale convention, coverage and multiplicity when
  comparing. Label approximation, endpoint projections and unsupported cases.
- After edits, check affected tests, examples, comments and documentation. Run
  the applicable CONTRIBUTING checks and `python3 collaboration/check.py`.
  Separate local checks, hosted CI, oracle results and performance evidence.
- Raw inputs/results, downloaded sources and logs stay under ignored `target/`
  or approved artifact storage. Never commit credentials or generated artifacts.
- Do not claim a completed phase, speedup, oracle agreement or active GitHub
  review integration based only on a plan, configuration file or unrun command.

## Submission and review

- Follow [workflow.md](workflow.md) and the commit/PR templates it links.
  Create a feature branch; target PRs explicitly at `proffitteoy/cocycle-rs:main`.
  Do not push directly to main, force-push shared history, merge, release, or
  submit to the collaborator's upstream unless the user specifically requests it.
- Check the push URL and PR base/head before every submission. The user handles
  subsequent synchronization to the collaborator's repository.
- Formal reviews use [review.md](review.md), focus on the PR diff, and remain
  read-only. Missing evidence is a verification gap, not proof of a code defect.
- User instructions take precedence over this workflow. Continue already
  authorized reversible work without inventing additional approval gates.
