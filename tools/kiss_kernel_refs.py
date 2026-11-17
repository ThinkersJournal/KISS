#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""
kiss_kernel_refs.py — the references to KISS-Kernel cannot rot (KISS-KERNEL-6.1-0001; board 450 option A).

CireSnave ruled board 450 "A...but only as long as the necessary references are kept in place so readers know when
recommendations are available." This lint is that condition, mechanised. It reads three places and fails when they
disagree:

  spec/kernel.md  §2.2   the reserved/published section map (`| 6.N | ...` rows): the sections a pointer may name.
  spec/ops.md     §6.22  the "Kernel-governed ops" table: op | KISS-Kernel sections | Ops clauses carrying the pointer.
  spec/ops.md     clauses  each carries `*Kernel-governed:* KISS-KERNEL-6.N[, KISS-KERNEL-6.M]` where it fixes an op's
                           concurrent behaviour.

Violations (each exits 1):
  * a table op that is not a KISS-Ops op;
  * a table section that is not in kernel.md §2.2;
  * a table row naming an Ops clause that carries no pointer, or whose pointer omits one of the row's sections
    ("a governed op with no pointer");
  * a pointer in an Ops clause that no table row lists, or that names a section absent from §2.2.

`kernel_governed(spec_dir)` is imported by tools/kiss_op_facts.py to emit the `kernel_rules` field of op_facts.json, so the
field is generated from the same table the lint checks.

Usage:
  python tools/kiss_kernel_refs.py          # check; exit 0 clean, 1 violations
"""
from __future__ import annotations

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

PTR = re.compile(r"\*Kernel-governed:\*\s*((?:KISS-KERNEL-\d+\.\d+(?:,\s*)?)+)")
SEC = re.compile(r"KISS-KERNEL-(\d+\.\d+)")
CLAUSE = re.compile(r"^- \*\*(KISS-OPS-\d+\.\d+-\d{4}[a-z]?)\*\*", re.M)


def _read(spec_dir, name):
    return open(os.path.join(spec_dir, name), encoding="utf-8").read().replace("\r\n", "\n")


def kernel_sections(spec_dir):
    """The section numbers (`6.N`) kernel.md §2.2 sets aside, reserved or published."""
    text = _read(spec_dir, "kernel.md")
    start = text.index("### 2.2")
    end = text.index("\n### ", start + 1) if "\n### " in text[start + 1:] else text.index("\n## ", start)
    return {m.group(1) for m in re.finditer(r"^\| (6\.\d+) \|", text[start:end], re.M)}


def governed_table(spec_dir):
    """{op: (sections, clauses)} from ops.md §6.22, in table order."""
    text = _read(spec_dir, "ops.md")
    start = text.index("### 6.22 Kernel-governed ops")
    end = text.index("\n## ", start)
    rows = {}
    for m in re.finditer(r"^\| `([a-z0-9_]+)` \| ([^|]+) \| ([^|]+) \|$", text[start:end], re.M):
        secs = [s.strip() for s in m.group(2).split(",") if s.strip()]
        clauses = re.findall(r"KISS-OPS-\d+\.\d+-\d{4}[a-z]?", m.group(3))
        rows[m.group(1)] = (secs, clauses)
    return rows


def pointers(spec_dir):
    """{ops clause id: set of `6.N`} for every `*Kernel-governed:*` pointer, by the clause that contains it."""
    text = _read(spec_dir, "ops.md")
    heads = [(m.start(), m.group(1)) for m in CLAUSE.finditer(text)]
    out = {}
    for m in PTR.finditer(text):
        owner = [cid for pos, cid in heads if pos < m.start()]
        if not owner:
            continue
        out.setdefault(owner[-1], set()).update(SEC.findall(m.group(1)))
    return out


def kernel_governed(spec_dir):
    """{op: ["KISS-KERNEL-6.4", ...]} for the `kernel_rules` field of op_facts.json."""
    return {op: [f"KISS-KERNEL-{s}" for s in secs] for op, (secs, _c) in governed_table(spec_dir).items()}


def check(spec_dir, all_ops):
    problems = []
    known = kernel_sections(spec_dir)
    table = governed_table(spec_dir)
    ptr = pointers(spec_dir)
    if not known:
        problems.append("kernel.md §2.2 lists no reserved section (the `| 6.N |` rows moved)")
    if not table:
        problems.append("ops.md §6.22 has no rows (the Kernel-governed ops table moved or emptied)")
    listed = set()
    for op, (secs, clauses) in table.items():
        if op not in all_ops:
            problems.append(f"§6.22: `{op}` is not a KISS-Ops op")
        for s in secs:
            if s not in known:
                problems.append(f"§6.22: `{op}` names KISS-Kernel {s}, which kernel.md §2.2 does not list")
        if not clauses:
            problems.append(f"§6.22: `{op}` names no Ops clause carrying the pointer")
        for c in clauses:
            listed.add(c)
            if c not in ptr:
                problems.append(f"{c}: listed for `{op}` in §6.22 but carries no `Kernel-governed:` pointer (a governed op with no pointer)")
            elif not set(secs) <= ptr[c]:
                miss = sorted(set(secs) - ptr[c])
                problems.append(f"{c}: its pointer omits {miss}, which §6.22 lists for `{op}`")
    for c, secs in sorted(ptr.items()):
        if c not in listed:
            problems.append(f"{c}: carries a `Kernel-governed:` pointer but no §6.22 row lists it")
        for s in sorted(secs - known):
            problems.append(f"{c}: pointer names KISS-Kernel {s}, which kernel.md §2.2 does not list")
    return problems


def main():
    import kiss_ops as K
    spec_dir = os.path.join(os.path.dirname(HERE), "spec")
    problems = check(spec_dir, set(K.build_manifest(spec_dir)["all_ops"]))
    print("KISS-Kernel references (KISS-KERNEL-6.1-0001)")
    print("=" * 68)
    t = governed_table(spec_dir)
    print(f"  governed ops: {', '.join(t) or '(none)'}; reserved sections: {', '.join(sorted(kernel_sections(spec_dir)))}")
    for p in problems:
        print(f"  VIOLATION: {p}")
    print("  RESULT: " + ("CLEAN" if not problems else "VIOLATIONS FOUND"))
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
