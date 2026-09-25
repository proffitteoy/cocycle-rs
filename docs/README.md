# Documentation

Cocycle is a native Rust TDA library. It computes dimension-generic ordinary Rips
persistence over prime fields, constructs exact threshold graphs and frozen
simplicial complexes, and accepts supplied flag filtrations. Sparse Rips
approximation includes blocker-aware computation and explicit metric hypotheses.
Owned results preserve coverage, field and approximation context; cycle and
cocycle bases are available on request at specified scales. Descriptors derive
lifetime statistics and Betti curves from diagrams.
Start with a task below; design pages distinguish implemented decisions from
proposed APIs and remaining acceptance gates.

## Use the library

- [Rips guide](guides/rips.md): choose an input, compute persistence, interpret
  coverage and endpoints, and derive measurements.
- [Rips construction guide](guides/rips-construction.md): matrix layouts, custom
  distances, threshold graphs, sparse computation and execution controls.
- [Sparse Rips approximation](guides/sparse-rips.md): metric hypotheses, blockers,
  sampling provenance and implicit/explicit computation.
- [Fields and representatives](guides/rips-representatives.md): select a prime
  field and request owned cycle/cocycle bases associated with intervals.
- [Runnable square example](../examples/square.rs): run
  `cargo run --locked --example square` from the repository root.
- [API reference source](../src/lib.rs): build rustdoc with
  `cargo doc --locked --no-deps --open` for signatures and error contracts.

## Understand and develop the core

| Question | Read |
| --- | --- |
| What mathematical and numerical guarantees apply? | [Mathematical specification](reference/mathematics.md) |
| Which papers justify the definitions and algorithms? | [Bibliography](reference/bibliography.md) |
| Where does code belong and how do the parts interact? | [Current architecture](development/architecture.md) |
| How should modules, APIs, errors, and source files be written? | [Code conventions](development/conventions.md) |
| How do we check correctness independently? | [Testing and validation](development/testing.md) |
| Which commands, review rules, and release checks apply? | [Contributing](../CONTRIBUTING.md) |

For implementation work, read architecture, the relevant mathematical section,
and its validation obligations together. The specification keeps shared notation
and numbered derivations in one place; rustdoc owns individual API contracts.

## Plan and compare

| Document | Status and purpose |
| --- | --- |
| [Roadmap](design/roadmap.md) | Selected priorities; not a delivery schedule |
| [Rips API design](design/rips-api.md) | Implemented workflow design, execution boundaries and legacy compatibility/migration |
| [Complete Rips subsystem](design/rips.md) | Rips target, original GUDHI/Ripser comparison, API sketches and current acceptance matrix; use guides/rustdoc for callable APIs |
| [Rips acceptance audit](design/rips-acceptance.md) | R1-R10 evidence, resource boundaries and local/hosted validation distinction |
| [Rips implementation scope](design/rips-implementation.md) | Stage-specific new directories, source moves, code/tooling changes and review units |
| [Kernel design](design/kernel.md) | Design rationale, extension boundaries, and proposed capability gates |
| [GUDHI C++ study](research/gudhi-cpp.md) | Pinned upstream source map and reading plan; excludes Python wrappers |
| [Benchmarks](../benches/README.md) | Suite-specific protocols and maintained comparisons with measured revisions and evidence status |
| [Rips comparison](../benches/reports/rips-comparison.md) | Measured native correctness and performance; updated in place on reruns |
| [Performance reporting rules](../benches/reporting.md) | Comparability, sampling, memory, source retention and report template |
| [Changelog](../CHANGELOG.md) | Completed user-visible changes |

For future capabilities, use the GUDHI study as evidence, the kernel design to
evaluate boundaries, and the roadmap to select the next operation. A source map
does not establish feature parity, and historical measurements do not describe
the performance of unmeasured code.

## Organization and maintenance

| Directory | Owns |
| --- | --- |
| `guides/` | Task-oriented usage of implemented capabilities |
| `reference/` | Mathematical contracts, derivations, and bibliography |
| `development/` | Current architecture, coding conventions, and validation strategy |
| `design/` | Design decisions, implementation records, future direction and acceptance gates |
| `research/` | Upstream investigations tied to a source revision and scope |

Add a document to the directory matching its purpose and link it from this index.
Use descriptive capability names such as `guides/rips.md`; add subdirectories
when a subject has multiple documents that need their own navigation. Do not
create empty sections for unimplemented capabilities or duplicate the source tree
outside the architecture document.

Keep current behavior, proposals, and dated observations explicit. Link to the
document that owns a fact instead of maintaining parallel versions. Project
documentation is in English. Paths in prose and shell commands are relative to
the repository root unless stated otherwise; Markdown links resolve relative to
their document. See [documentation maintenance](../CONTRIBUTING.md#documentation-ownership)
for checks to run when adding or moving pages.
