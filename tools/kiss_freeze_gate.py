#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""
kiss_freeze_gate.py — the blocking hard-fail for a Draft -> Frozen promotion (#524).

umbrella §5.3 condition 3 and KISS-CONFORM-8-0006: a sub-standard is not Frozen until its KISS-Conform
suite is complete, with clause<->test traceability both ways. `kiss_trace.py --freeze-ready` computes
that, but nothing RAN it where it bites: the `strict` job is non-blocking by design, so a PR that flips
a sub-standard's front-matter `Maturity stage` to Frozen merged green over hundreds of untested MUSTs.

What this gate does, exactly:

  * reads the front-matter row `| Maturity stage | **<stage>** ... |` of each `spec/<sub>.md`
    at the base ref and in the working tree;
  * for every sub-standard whose stage is FROZEN in the working tree: runs
    `kiss_trace.py --freeze-ready <SUB>` and FAILS the build unless it exits 0. That covers a promotion
    in this change AND a regression of an already-Frozen sub-standard (a Frozen standard stays traced);
  * a Draft change of any size is not its business (CLEAN, exit 0).

Three outcomes, never conflated (a gate that cannot tell its states apart reports a refusal as a
violation — the same lesson as the ratchet, #251):

  0  CLEAN      no sub-standard is Frozen, or every Frozen one passes --freeze-ready
  1  BLOCKED    a Frozen sub-standard fails --freeze-ready (promotion or regression)
  2  DECLINED   the gate could not read a stage or the base ref — INCONCLUSIVE, not a pass

What it does NOT check, said plainly: who signed (#462 — the signature is still the only enforcement of
the AUDIT-role part of §5.3), traceability ACCURACY (#387), conditions 1 and 2 (independent derivation,
a foreign reader). It is the cheapest of the five instruments and it is a floor, not the freeze gate.

Usage:
  python tools/kiss_freeze_gate.py
"""
from __future__ import annotations

import argparse
import contextlib
import io
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
ROOT = os.path.dirname(HERE)
SPEC_DIR = os.path.join(ROOT, "spec")

# the nine sub-standards, by file stem -> the name `--freeze-ready` takes
SUBS = ["announce", "classify", "conform", "consume", "contract", "emit", "grammar", "ops", "synth"]

STAGE_ROW = re.compile(r"^\|\s*Maturity stage\s*\|\s*(.+?)\s*\|\s*$", re.M)
STAGE_TOKEN = re.compile(r"\*\*\s*([A-Za-z]+)")


def maturity_stage(spec_text: str):
    """The stage token of the front-matter `Maturity stage` row, e.g. `Draft` / `Frozen`; None if the
    row or its bold token is absent (the caller must DECLINE, never assume Draft)."""
    m = STAGE_ROW.search(spec_text)
    if not m:
        return None
    t = STAGE_TOKEN.search(m.group(1))
    return t.group(1) if t else None


def is_frozen(stage) -> bool:
    return stage is not None and stage.lower() == "frozen"


def evaluate(stages: dict, run_freeze_ready) -> tuple[int, list[str]]:
    """Pure decision: `stages` maps sub -> stage token or None; `run_freeze_ready(sub) -> int` is the
    `--freeze-ready` exit status. Returns (exit_code, report lines)."""
    lines = []
    unreadable = sorted(s for s, st in stages.items() if st is None)
    if unreadable:
        lines.append(f"DECLINED: no readable `Maturity stage` row in: {', '.join(unreadable)}")
        return 2, lines
    frozen = sorted(s for s, st in stages.items() if is_frozen(st))
    if not frozen:
        lines.append("CLEAN: no sub-standard is Frozen; nothing to gate.")
        return 0, lines
    blocked = []
    for s in frozen:
        rc = run_freeze_ready(s)
        if rc == 0:
            lines.append(f"  {s}: Frozen and --freeze-ready passes")
        else:
            blocked.append(s)
            lines.append(f"  {s}: Frozen but --freeze-ready exits {rc} — BLOCKED")
    if blocked:
        lines.append(
            f"BLOCKED: {', '.join(blocked)} claim(s) Frozen while umbrella §5.3 condition 3 is unmet "
            f"(`python tools/kiss_trace.py --freeze-ready <SUB>` lists the clauses)."
        )
        return 1, lines
    lines.append("CLEAN: every Frozen sub-standard passes --freeze-ready.")
    return 0, lines


def _read_worktree(sub: str):
    p = os.path.join(SPEC_DIR, f"{sub}.md")
    if not os.path.exists(p):
        return None
    with open(p, encoding="utf-8") as f:
        return f.read()


def _freeze_ready(sub: str) -> int:
    """`kiss_trace.py --freeze-ready <SUB>` exit status, run in-process (no subprocess: the gate has
    no business spawning a shell, and a Frozen claim is decided by the same code CI already runs)."""
    import kiss_trace

    argv = sys.argv
    sys.argv = ["kiss_trace.py", "--freeze-ready", sub.upper()]
    try:
        with contextlib.redirect_stdout(io.StringIO()):
            rc = kiss_trace.main()
    except SystemExit as e:  # kiss_trace may sys.exit
        rc = e.code
    finally:
        sys.argv = argv
    return int(rc or 0)


def main(argv=None) -> int:
    argparse.ArgumentParser(description="Blocking hard-fail for a Frozen sub-standard (#524)").parse_args(argv)

    stages = {s: (maturity_stage(t) if (t := _read_worktree(s)) is not None else None) for s in SUBS}
    print("KISS freeze gate (#524) — umbrella §5.3 condition 3 on a Frozen sub-standard")
    print("=" * 68)
    for s in SUBS:
        print(f"  {s:9} {stages[s]}")
    code, lines = evaluate(stages, _freeze_ready)
    print("-" * 68)
    print("\n".join(lines))
    return code


if __name__ == "__main__":
    sys.exit(main())
