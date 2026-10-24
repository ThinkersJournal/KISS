#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""
kiss_io_discard.py — a lint for a filesystem read or enumeration whose failure is discarded (KISS #148).

KISS-CONFORM-6.5-0016 says an instrument's negative MUST be distinguishable from its failure to look.
The way that goes wrong in Rust is the SHORT form: a discovery step that can fail and silently
shrinks its result set instead of failing,

    let Ok(entries) = std::fs::read_dir(dir) else { return; };   // truncates silently
    let Ok(text)    = std::fs::read_to_string(&p) else { continue };
    std::fs::read_dir(d).ok()?                                   // an unreadable dir is "empty"
    entries.flatten()                                            // drops a failed DirEntry

Four instances were written in one afternoon, the fourth by the author of the clause forbidding it;
nothing in the language or the review path resists the short form, which is the argument for a lint.

WHAT IT FLAGS. A statement that contains a filesystem read (`read_dir`, `read_to_string`,
`fs::read(`, `File::open`) AND discards its `Result` by one of: `.ok()`, `.flatten()`,
`unwrap_or_default()`, `let Ok(..) =` / `if let Ok(..) =`. It does NOT judge whether the context is a
"discovery" one: that boundary cannot be drawn textually, so the line is drawn at the I/O call and
the exemption is explicit and visible instead.

THE EXEMPTION. A deliberate probe (absence IS the answer, e.g. "is a compiler installed here") puts
`io-discard-ok: <reason>` in a `//` comment on the statement's first line or within the three lines
above it. An empty reason does not exempt.

WHAT IT CANNOT SEE. A `Result` discarded across a function boundary (a helper returning `Option` that
swallows the error inside), `let _ = ..`, a `match` arm that ignores `Err`, non-`fs` I/O (network,
process), and any I/O reached through a path this textual match does not name. A clean run says no
statement has that exact shape; it does not say the code propagates its I/O errors.

Enumerates from the index (`git ls-files`, never a disk walk, so another lane's worktree is not
scanned) and prints which enumeration ran and how many files it read. Stdlib only.

Usage:
  python tools/kiss_io_discard.py                # lint conformance/ and tools/ (*.rs)
  python tools/kiss_io_discard.py --root DIR     # lint the index of another checkout
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys

IO_CALL = re.compile(r"\b(?:read_dir|read_to_string)\s*\(|\bfs::read\s*\(|\bFile::open\s*\(")
DISCARDS = (
    ("ok()", re.compile(r"\.ok\s*\(\s*\)")),
    ("flatten()", re.compile(r"\.flatten\s*\(\s*\)")),
    ("unwrap_or_default()", re.compile(r"\.unwrap_or_default\s*\(\s*\)")),
    ("let Ok(..) =", re.compile(r"\blet\s+Ok\s*\(")),
)
EXEMPT = re.compile(r"io-discard-ok:\s*\S")
EXEMPT_WINDOW = 3  # lines above the statement's first line that may carry the marker


def _blank_line_comments(line):
    """The line with a trailing `//` comment blanked, unless the `//` sits inside a string."""
    i = line.find("//")
    while i != -1:
        if line.count('"', 0, i) % 2 == 0:
            return line[:i]
        i = line.find("//", i + 2)
    return line


def scan_text(text):
    """[(line, form)] for every statement that reads the filesystem and discards the Result,
    unless a marked exemption covers it. `line` is where the I/O call is."""
    raw = text.split("\n")
    code = "\n".join(_blank_line_comments(l) for l in raw)
    found = []
    for m in IO_CALL.finditer(code):
        start = max(code.rfind(";", 0, m.start()), code.rfind("{", 0, m.start()), code.rfind("}", 0, m.start()))
        end = code.find(";", m.end())
        stmt = code[start + 1:(len(code) if end == -1 else end)]
        form = next((name for name, rx in DISCARDS if rx.search(stmt)), None)
        if form is None:
            continue
        first = code.count("\n", 0, start + 1 + (len(stmt) - len(stmt.lstrip())))
        window = raw[max(0, first - EXEMPT_WINDOW):first + 1]
        if any(EXEMPT.search(l) for l in window):
            continue
        found.append((code.count("\n", 0, m.start()) + 1, form))
    return found


def tracked_rust_files(root):
    out = subprocess.run(["git", "-C", root, "ls-files", "conformance", "tools"],
                         capture_output=True, text=True, check=True).stdout.split()
    return [f for f in out if f.endswith(".rs")]


def main():
    ap = argparse.ArgumentParser(description="KISS #148: a filesystem read whose failure is discarded")
    ap.add_argument("--root", default=os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    args = ap.parse_args()
    files = tracked_rust_files(args.root)
    print("KISS #148 silent I/O discard lint (KISS-CONFORM-6.5-0016)")
    print("=" * 68)
    print(f"  enumerated from the index (git ls-files): {len(files)} Rust file(s)")
    bad = []
    for f in files:
        with open(os.path.join(args.root, f), encoding="utf-8", errors="replace") as fh:
            bad += [(f, ln, form) for ln, form in scan_text(fh.read())]
    if bad:
        print(f"  VIOLATIONS — {len(bad)} discarded filesystem Result(s):")
        for f, ln, form in bad:
            print(f"    {f}:{ln}  {form}")
        print("  propagate the error (`?`), or mark a deliberate probe with `// io-discard-ok: <reason>`")
        return 1
    print("  RESULT: CLEAN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
