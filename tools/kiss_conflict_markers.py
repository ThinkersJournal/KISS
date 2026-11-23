#!/usr/bin/env python3
"""Fail on unresolved merge-conflict markers in any tracked file (board 476 follow-up).

origin/main carried a committed `<<<<<<< HEAD` / `=======` / `>>>>>>> origin/main` block in
conformance/README.md (introduced by 4ac8803): every other gate read through it, because
`kiss_readme_coverage.py` compares each `N<!-- bound:key -->` it finds against the tree and
both halves of the block carried the same figure (it reports such repeats as "legal, every copy
was compared"). Nothing looked for the markers themselves.

Rule: a line at column 0 that is `<<<<<<<` or `>>>>>>>` (followed by a space or end of line) is
a marker. A column-0 `=======` is a marker ONLY inside an open `<<<<<<<` region or when the file
also has a `<<<<<<<`/`>>>>>>>` line: a lone `=======` is a legitimate Markdown setext / RST
underline and is allowed.
Files are enumerated with `git ls-files -z` (never a disk walk: other lanes' worktrees live
under C:/Projects). Binary files (a NUL byte) are skipped.

Run: python tools/kiss_conflict_markers.py        exit 0 clean, 1 markers found.
"""
from __future__ import annotations

import subprocess
import sys

OPEN, CLOSE, MID = b"<<<<<<<", b">>>>>>>", b"======="


def _is(line: bytes, tok: bytes) -> bool:
    line = line.rstrip(b"\r")
    return line == tok or line.startswith(tok + b" ")


def find_markers(data: bytes) -> list[tuple[int, bytes]]:
    """Return [(1-based line, kind)] for every conflict marker in `data`."""
    lines = data.split(b"\n")
    has_edge = any(_is(l, OPEN) or _is(l, CLOSE) for l in lines)
    hits = []
    for n, l in enumerate(lines, 1):
        if _is(l, OPEN) or _is(l, CLOSE):
            hits.append((n, l.rstrip(b"\r")[:7]))
        elif has_edge and l.rstrip(b"\r") == MID:
            hits.append((n, MID))
    return hits


def tracked_files() -> list[str]:
    out = subprocess.run(["git", "ls-files", "-z"], capture_output=True, check=True).stdout
    return [p for p in out.decode("utf-8", "surrogateescape").split("\0") if p]


def main() -> int:
    files = tracked_files()
    bad = 0
    for p in files:
        try:
            data = open(p, "rb").read()
        except OSError:
            continue  # tracked but absent from the worktree (sparse/deleted): not ours to judge
        if b"\0" in data:
            continue
        for n, kind in find_markers(data):
            print(f"  [FAIL] {p}:{n}: conflict marker {kind.decode()}")
            bad += 1
    print(f"  scanned {len(files)} tracked file(s) via git ls-files; {bad} marker line(s).")
    print("  RESULT: " + ("CONFLICT MARKERS FOUND" if bad else "CLEAN"))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
