"""Run focused contributor checks from a source checkout, without native tools."""

import argparse
import json
from pathlib import Path
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def run(command, **kwargs):
    print("+ " + shlex.join(map(str, command)), flush=True)
    return subprocess.run(command, cwd=ROOT, check=True, text=True, **kwargs)


def library_artifact():
    # Ask Cargo for the actual path so CARGO_TARGET_DIR and platform-specific
    # paths do not require a second copy of Cargo's configuration logic.
    result = run(["cargo", "build", "--locked", "--lib", "--message-format=json-render-diagnostics"],
                 stdout=subprocess.PIPE)
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if (message.get("reason") == "compiler-artifact"
                and message.get("target", {}).get("name") == "cocycle"):
            for filename in message.get("filenames", []):
                if filename.endswith(".rlib"):
                    return Path(filename)
    raise ValueError("Cargo did not report the cocycle library artifact")


DOMAINS = {
    "diagram-analysis": {
        "sources": ("src/descriptors", "src/diagram"),
        "tests": ("descriptors", "contracts"),
        "example": "diagram_analysis",
        "guides": ("docs/development/diagram-analysis.md",),
    },
    "complex-construction": {
        "sources": ("src/complex", "src/filtration/simplicial"),
        "tests": ("filtered_complex",),
        "example": "complex_construction",
        "guides": ("docs/development/complex-construction.md",
                   "docs/guides/filtered-complexes.md"),
    },
    "diagram-distances": {
        "sources": ("src/diagram_distances",),
        "tests": ("diagram_distances",),
        "example": "diagram_distances",
        "guides": ("docs/development/diagram-analysis.md",),
        "unit_filter": "diagram_distances::",
    },
    "persistence-reduction": {
        "sources": ("src/algebra", "src/persistence"),
        "tests": ("filtered_complex", "prime_fields", "rips_resources", "rips_api"),
        "example": "flag_persistence",
        "guides": ("docs/development/persistence-reduction.md",),
        "unit_filter": "persistence::",
    },
}


def check_algorithm(domain):
    selected = DOMAINS[domain]
    run([sys.executable, "tools/check_source.py"])
    run([sys.executable, "tools/check_docs.py"])
    sources = [path for directory in selected["sources"]
               for path in sorted((ROOT / directory).rglob("*.rs"))]
    sources += [ROOT / f"tests/{name}.rs" for name in selected["tests"]]
    sources.append(ROOT / f"examples/{selected['example']}.rs")
    run(["rustfmt", "--edition", "2024", "--check",
         *(str(path.relative_to(ROOT)) for path in sources)])
    targets = [arg for name in selected["tests"] for arg in ("--test", name)]
    targets += ["--example", selected["example"]]
    run(["cargo", "clippy", "--locked", "--all-features", "--lib", *targets,
         "--", "-D", "warnings"])
    # Explicit --example is necessary: ordinary cargo test does not run the
    # constructor's colocated algorithm tests.
    run(["cargo", "test", "--locked", "--all-features", *targets])
    if "unit_filter" in selected:
        run(["cargo", "test", "--locked", "--all-features", "--lib", selected["unit_filter"]])
    run(["cargo", "run", "--locked", "--example", selected["example"]])
    library = library_artifact()
    for guide in selected["guides"]:
        run(["rustdoc", "--edition", "2024", "-D", "warnings", "--test", guide,
             "--extern", f"cocycle={library}", "-L", f"dependency={library.parent / 'deps'}"])


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("domain", choices=DOMAINS)
    args = parser.parse_args(argv)
    try:
        check_algorithm(args.domain)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Algorithm checks failed: {error}", file=sys.stderr)
        return 1
    print(f"{args.domain} checks passed. Full maintainer/CI checks remain separate.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
