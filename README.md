![Cocycle — topology, computed in Rust](assets/banner.svg)

[![CI](https://github.com/huangbogeng/cocycle-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/huangbogeng/cocycle-rs/actions/workflows/ci.yml)
[![Rust 1.91+](https://img.shields.io/badge/Rust-1.91%2B-102D32?style=flat-square)](Cargo.toml)
[![License: MIT](https://img.shields.io/badge/license-MIT-087D70?style=flat-square)](LICENSE)

**Find structure across scales.** Cocycle turns point clouds and pairwise
dissimilarities into persistence diagrams: which components and loops appear,
and how long they last. A general-purpose TDA library in pure Rust, with no runtime
dependencies and no unsafe code.

- **Rips persistence** — dimension-generic ordinary persistence over prime fields,
  with a specialized F₂ H₀/H₁ path. Euclidean points or borrowed matrix layouts.
- **Inspectable construction** — exact threshold graphs, supplied weighted flag
  filtrations, and frozen simplicial complexes with boundary/cofacet queries.
- **Sparse approximation** — deterministic sampling, modified edges and higher-simplex
  blockers, with explicit metric hypotheses and approximation provenance.
- **Optional representatives** — owned persistent cycle bases and query-scale dual
  cocycles, associated with intervals and original vertex IDs.
- **Meaningful results** — owned diagrams preserve multiplicity and distinguish
  finite deaths, essential classes, and right-censored intervals.
- **Useful summaries** — finite lifetimes, persistence entropy in nats, and Betti
  curves, computed directly from a diagram.

![A square's Rips filtration: points, a loop at edge length 1, and filled triangles at sqrt(2).](assets/filtration.svg)

## Quick start

**Rust 1.91+ · Pre-release.** Not yet published on crates.io; use the Git dependency:

```toml
[dependencies]
cocycle = { git = "https://github.com/huangbogeng/cocycle-rs", branch = "main" }
```

```rust
use cocycle::descriptors::betti_curve;
use cocycle::geometry::PointCloudView;
use cocycle::filtration::RipsBuilder;
use cocycle::persistence::PersistenceExt;

fn main() -> cocycle::Result<()> {
    let square = [0., 0., 1., 0., 1., 1., 0., 1.];
    let points = PointCloudView::new(&square, 4, 2)?;
    let result = RipsBuilder::from_points(points).persistence().compute()?;
    let diagram = result.diagram();
    assert_eq!(betti_curve(diagram, 1, &[0., 1., 2.])?, [0, 1, 0]);
    Ok(())
}
```

The square's H₁ interval is `[1, sqrt(2))`. Scales are **edge lengths**; a class
surviving an incomplete cutoff is censored, not dead. Your application's
`Cargo.lock` pins the resolved Git commit. See the [user guide](docs/guides/rips.md)
for cutoffs, input layouts, and result semantics.

## Explore

[Documentation](docs/README.md) · [Rips guide](docs/guides/rips.md) · [Graph construction](docs/guides/rips-construction.md) · [Mathematics](docs/reference/mathematics.md) ·
[Architecture](docs/development/architecture.md) · [Benchmarks](benches/README.md) ·
[Rips comparison](benches/reports/rips-comparison.md) · [Contributing](CONTRIBUTING.md) · [Roadmap](docs/design/roadmap.md)

Build the API reference with `cargo doc --no-deps --open`, or run the example with
`cargo run --locked --example square`. Dimension-generic prime-field persistence and
explicit complex queries are demonstrated by `cargo run --example rips_sphere`.
[Prime fields and representative bases](docs/guides/rips-representatives.md) are
demonstrated by `cargo run --example rips_representatives`.
[Sparse Rips approximation](docs/guides/sparse-rips.md), including metric hypotheses
and sampling provenance, is demonstrated by `cargo run --example sparse_rips`.
Work and memory depend on the input and reduction fill-in. Current comparisons
use native GUDHI and upstream Ripser C++; see the maintained
[Rips comparison](benches/reports/rips-comparison.md) for tested scope, correctness
and performance observations. Generated measurements and logs stay outside source Git.

[Issue tracker](https://github.com/huangbogeng/cocycle-rs/issues) ·
Code and original artwork are [MIT licensed](LICENSE).
