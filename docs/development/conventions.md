# Code conventions

[Documentation](../README.md) / Development

These rules govern maintained source and tooling. The [architecture](architecture.md)
describes the current implementation; [CONTRIBUTING.md](../../CONTRIBUTING.md)
owns verification commands and review procedures. Future capabilities must meet
the [kernel design gates](../design/kernel.md), not create empty module trees.

## File and dependency boundaries

- Give each implemented domain a directory with a `mod.rs` documenting its scope
  and exports, irrespective of line count. A concrete leaf operation may remain
  one file. `lib.rs` and the shared `error.rs` remain crate-level files.
- A file owns a coherent operation, data invariant, or storage component. Keep a
  type and its ordinary methods together. Facades contain exports, documentation,
  and small composition logic; substantial algorithms belong in named modules.
- Name modules by domain, types by the objects they represent, and functions by
  observable operations. Distinguish filtration access from persistence engines
  and diagram distances from geometric distances. Avoid catch-all `utils`,
  `types`, `config`, and `manager` modules.
- Default to private items, then `pub(super)` or a precise `pub(in ...)` boundary.
  Use `pub(crate)` for actual cross-module consumers. Private file nesting does
  not require public nesting; re-export established paths when splitting files.
  A public rename needs an explicit compatibility decision and migration.
- Keep local indices, options, and workspaces with their feature. Use distinct
  index types when confusing storage positions and mathematical IDs is a real
  risk. Shared helpers belong below their consumers, not in a sibling algorithm.
- Follow the semantic dependencies in the
  [architecture](architecture.md#production-code): geometry and
  filtration do not call persistence; descriptors need no input geometry or
  reducer state; production never depends on the test oracle. There is one
  authority for production Rips ordering, with independent oracle enumeration.
- Document a shared contract's owner, invariants, permitted dependencies, and
  validation strategy. A trait must distinguish guarantees implementations
  establish from conditions callers check. Similar formulas alone do not
  establish compatible overflow, ordering, or storage semantics.
- Co-locate private invariant tests; put public usage contracts in `tests/` and
  runnable workflows in `examples/`. Diagnostics remain test-only. Split crates
  only for an actual dependency, release, or integration boundary.

## Public contracts and ownership

The current public domains are `geometry`, `complex`, `filtration`, `persistence`,
`diagram`, `descriptors`, and `execution`. These are not a permanent closed list. Add a public domain when a
concrete implemented capability needs it, with documented inputs, outputs,
dependencies, and independent validation. Preserve established public paths when
reorganizing private files. Prefer concrete types until real implementations
justify a shared trait; do not expose reducer storage or tuning switches.

Validate borrowed input at construction. Keep engine workspaces private and
return owned analysis results that do not depend on the input or engine lifetime.
Preserve multiplicity, computed dimensions, and coverage; a censored endpoint is
not a finite death or a proof of essentiality. New capabilities must state their
own corresponding contracts rather than inherit assumptions from Rips.

Public rustdoc explains mathematical meaning, units, ordering, empty cases,
coverage, and error behavior where applicable. Provide a runnable example when
it helps use the API. Comments explain invariants and non-obvious choices; avoid
restating syntax. Keep shared mathematics in the [specification](../reference/mathematics.md)
and link to it rather than duplicating derivations.

## Numeric and error handling

- Unsafe code is forbidden. Check size/index arithmetic before allocating or
  narrowing integers. Use fallible reservations where supported, without claiming
  that every allocation failure is recoverable.
- Reject invalid user input through structured `Result` errors. Do not return an
  empty diagram, NaN, or an apparently successful partial result to hide failure.
  Keep error variants meaningful; human-readable `Display` text is not a stable
  serialization format.
- Do not use `unwrap`, `expect`, or assertions for recoverable user-input errors.
  An internal use needs a locally established invariant; document why it holds
  when non-obvious. Tests and fixed fixtures can use them to make failures clear.
- State precision, overflow, ties, zero-lifetime, and truncation behavior before
  optimizing. Do not silently normalize, deduplicate, quantize, approximate, or
  widen tolerances to make comparisons pass.
- Reuse code only when its numeric, ordering, and ownership contracts agree.
  The explicit test oracle stays independent of production enumeration and
  reduction, even when this duplicates a small formula.

## Language and formatting

Use English for documentation, comments, templates, and change descriptions.
Use descriptive domain vocabulary consistently. Rust modules/functions use
`snake_case`, types/traits use `UpperCamelCase`, and constants use
`SCREAMING_SNAKE_CASE`; prefer meaningful names over abbreviations except for
well-scoped mathematical indices.

| Source | Convention and enforcement |
| --- | --- |
| Rust | Edition 2024; `cargo fmt`, standalone `rustfmt` for tool drivers, and Clippy with warnings denied |
| Python tools | Four spaces, standard library for default checks/native orchestration, `snake_case` functions/modules; parse all tools and run unit tests |
| Native C++ adapters | C++17, two spaces, explicit standard includes and small protocol-focused adapters; compile and run native smoke comparisons |
| Markdown, TOML, YAML, JSON | Two-space indentation; descriptive headings and local relative links |
| Maintained text | UTF-8 without BOM, LF, final newline, no trailing whitespace or indentation tabs |

`.editorconfig` configures editor defaults. `tools/check_source.py` enforces the
listed text rules and Python syntax, not full Python/C++ style or indentation
width. Follow the surrounding source for quote and wrapping choices; avoid
unrelated formatting. No additional Python or C++ formatter is required.
C++ here is benchmark infrastructure; it must not become a core dependency.

Developer commands must fail visibly on invalid input or subprocess failure.
Keep generated files under `target/` or the explicitly selected output directory.
Record source revisions and numerical protocol for comparisons. Store generated
outputs under ignored `target/` or in external artifact storage. Do not commit
logs, raw results or archives, and do not reformat downloaded third-party sources.
Run `tools/check_artifacts.py` after staging to check artifact admission. Follow
the [reporting rules](../../benches/reporting.md) for comparison contracts,
measurement claims and retained evidence.

## Tests and documentation

Choose tests that fail when a contract breaks: a hand-derived expectation, an
independent oracle, a mathematical property, or a real regression. Do not copy
production logic into expected results or assert timings. Keep random tests
reproducible with fixed seeds and report enough input to reproduce failures.
Use exact comparisons for discrete structure and exact shared filtration values;
justify tolerances for floating-point calculations independently.

For a new public capability, cover a normal example, empty/degenerate inputs,
invalid inputs, relevant numeric boundaries, and ownership/coverage semantics.
For a fix, add a regression when it protects observable behavior or a nontrivial
invariant. A private file move or prose edit does not need artificial tests.
See [testing](testing.md) for existing independent validation layers.

Separate current usage, mathematical reference, implementation notes, future
design, and dated research using the [documentation index](../README.md).
Use ATX headings (`#`, `##`, etc.), single-line inline links or reference
link definitions, and fenced code blocks. These forms are checked by the local
Markdown checker, including local heading fragments and missing explicit
reference labels. It is not a complete Markdown parser, does not fetch external
URLs, and only flags CJK text as a limited check of the English convention;
review language and meaning manually. Use an explicit definition for shortcut
links; undefined bare brackets are ordinary prose to the checker.
