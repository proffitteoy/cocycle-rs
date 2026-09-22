"""Apply existing documentation/source checks to the collaboration directory."""
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import check_docs
import check_source


def main():
    paths = sorted(path for path in (ROOT / "collaboration").rglob("*")
                   if path.is_file() and path.suffix in {".md", ".txt", ".py"})
    errors = [error for path in paths for error in check_source.check(path)]
    if errors:
        print("\n".join(errors), file=sys.stderr)
    check_docs.PATTERNS = (*check_docs.PATTERNS, "collaboration/**/*.md")
    docs_status = check_docs.main()
    if not errors:
        print(f"Collaboration source checks passed: {len(paths)} files.")
    return int(bool(errors) or bool(docs_status))


if __name__ == "__main__":
    raise SystemExit(main())
