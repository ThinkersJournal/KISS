#!/usr/bin/env python3
"""The conformance README's coverage figures are BOUND to the artifacts that maintain them.

`conformance/README.md` carried three claims that had aged apart from the tree in the same
directory: `31 of 855 normative clauses`, `130 tests pass by default`, and `8 of 9
sub-standards are at 0.0%`. The ratcheted floor file beside it said `harness 380`. One is
maintained by CI on every merge; the other by whoever last remembered.

    A NUMBER IN A README WITH NO GENERATOR BEHIND IT DOES NOT FAIL -- IT AGES.

An abandoned branch (`salvage/docs-coverage-refresh`, 2026-07-23) carries a third set --
`114 / 13.2%` -- reached by regenerating the figures BY HAND. It is now wrong the same way
by the same mechanism, which is the argument for binding rather than refreshing.

THE METRIC DECISION IS THE DELIVERABLE, and the two numbers were NOT the same metric:

    README `31 of 855`   -- forward only. Its own sentence said the rest "name a conformance
                            test that does not exist", which is the NAMED tier.
    floor  `harness 380` -- NAMED + CITED: a clause is backed if the §9 name resolves OR
                            some test carries a backing-form citation for it.

Binding to `harness` and rewriting the sentence, because the section asks "how much of the
spec is actually executable" and `harness` answers that question, is what the floor ratchets,
and is the number CI already defends. The narrower NAMED figure is kept alongside rather than
dropped, so the finer claim is not silently lost.

WHAT IS BOUND AND WHAT IS ONLY MEASURED, stated because the difference matters: the clause
figures and the per-sub-standard claim are recomputed here and MUST match. The
"tests pass by default" count cannot be reached without executing the suite, so this lint
binds the STATIC count of discovered test fns and the README says which is which.

Run: python tools/kiss_readme_coverage.py
"""
import bisect
from collections import Counter
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kiss_trace import read_ledger  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
README = os.path.join(ROOT, "conformance", "README.md")
FLOOR = os.path.join(ROOT, "conformance", "COVERAGE_FLOOR.tsv")

# THE MARKER IS A POINTER, NOT A COPY (#350 review). It follows the VISIBLE figure and
# names which key that figure is; the lint reads the number a READER sees.
#
# The first version put the value inside the comment -- `<!-- bound:harness=380 -->` --
# so the bound value and the visible value were TWO OBJECTS, and only one of them is what
# a reader is misled by. Editing the prose to `999 of 932` and leaving the comment alone
# reported CLEAN: the guard was INVARIANT UNDER THE EXACT DRIFT IT EXISTS TO CATCH.
# Verified by seeding it, not by reading. With one number there is no shadow to agree with.
RE_BOUND = re.compile(r"(?P<val>[0-9]+)\s*<!--\s*bound:(?P<key>[a-z_]+)\s*-->")


def floor_values(path=FLOOR):
    out = {}
    if not os.path.exists(path):
        return None
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            parts = line.strip().split("\t")
            if len(parts) >= 2 and parts[1].strip().isdigit():
                out[parts[0].strip()] = int(parts[1].strip())
    return out


def derived():
    """Figures recomputed from the tool that CI already runs, never from prose."""
    r = subprocess.run([sys.executable, os.path.join(HERE, "kiss_trace.py")],
                       capture_output=True, timeout=600)
    text = r.stdout.decode("utf-8", "replace")
    err = r.stderr.decode("utf-8", "replace")
    # FAIL CLOSED on a generator that did not produce an answer (#350 review). kiss_trace
    # exits 1 when the tree has violations -- that is normal and its figures are still
    # printed -- but a CRASH or a kill leaves partial output that parses to nothing, and the
    # lint would then report "DRIFT: actual None" for every figure. A MALFORMED INSTRUMENT
    # PRODUCING SOMETHING THAT PARSES AS AN ANSWER, and the failure wears the costume of a
    # real finding, which is the expensive direction to be wrong in.
    if "normative clauses" not in text:
        raise RuntimeError(
            "kiss_trace.py produced no figures (exit %d). This is a GENERATOR FAILURE, not "
            "README drift -- do not update the README from it.\n--- stderr ---\n%s"
            % (r.returncode, err[-800:] or "<empty>"))
    out = {}
    m = re.search(r"(\d+) normative clauses", text)
    if m:
        out["clauses"] = int(m.group(1))
    m = re.search(r"(\d+) executable test fns found", text)
    if m:
        out["test_fns"] = int(m.group(1))
    m = re.search(r"(\d+)/(\d+) clauses \([\d.]+%\) are backed", text)
    if m:
        out["harness"] = int(m.group(1))
    m = re.search(r"(\d+) executable tests cite no clause", text)
    if m:
        out["uncited_tests"] = int(m.group(1))
    m = re.search(r"(\d+) NAMED", text)
    if m:
        out["named"] = int(m.group(1))
    # the ledger's own row counts, so the README's breakdown cannot drift from it either
    try:
        led = read_ledger(os.path.join(ROOT, "conformance", "UNBACKED.tsv"))
        out["untested_rows"] = sum(1 for p in led.values() if p["category"] == "untested")
    except Exception:
        pass
    # sub-standards with ZERO traced clauses — the `8 of 9 at 0.0%` claim
    zero = len([1 for ln in text.splitlines()
                if re.search(r"\[FAIL\]\s+[A-Z]+\s+0/\d+", ln)])
    out["zero_coverage_subs"] = zero
    return out


def occurrences(text):
    """[(key, value, line)] for EVERY marker — a list, never a dict (#359).

    This was `{m.group("key"): int(m.group("val")) for m in ...}`, and a dict comprehension
    SILENTLY DROPS every repeat: with the same key present twice, only the LAST occurrence
    was ever compared. A duplicate marker LOOKED BOUND AND WAS NOT.

    That is #350's review finding arriving through a second door. There the bound value and
    the visible value were two objects because the value sat inside the comment; here they
    diverge by MULTIPLICITY. Both fail in the same direction — the lint passes — which is
    the direction nobody investigates.

    A repeated key is NOT itself an error. A figure may honestly appear twice, and banning
    that would push authors back to unbound prose, which is the disease this tool exists to
    treat. Comparing EVERY occurrence against the tree makes agreement the requirement
    instead of uniqueness.
    """
    # Newline offsets precomputed once and bisected, rather than re-counting from the start
    # of the file for every match (#361 review). Measured on the live README -- 12 KB, 7
    # markers -- the difference is 0.28 ms/call and irrelevant; at 200x the file and 200x
    # the markers it is 907 ms vs 54 ms. The shape is quadratic, the cost today is not, and
    # bisect is no harder to read, so this removes it before it is ever reached.
    nl = [m.start() for m in re.finditer("\n", text)]
    return [(m.group("key"), int(m.group("val")), bisect.bisect_left(nl, m.start()) + 1)
            for m in RE_BOUND.finditer(text)]


def repeated_keys(claimed):
    """How many KEYS appear more than once — NOT how many extra occurrences there are.

    `len(claimed) - len(set(keys))` is the occurrence surplus: one key appearing three times
    gives 2, while exactly ONE key repeats. The summary line says "repeated key(s)", so that
    arithmetic ranged over something other than what the sentence claimed (#361 review).

    THE SAME ERROR THIS FILE EXISTS TO CATCH, ONE LEVEL IN: the number was right about a
    construct nobody had named, and the prose beside it named a different one. It is a
    function rather than an inline expression so a control can assert it.
    """
    return sum(1 for n in Counter(k for k, _v, _ln in claimed).values() if n > 1)


def check(readme=None, actual=None):
    """[(key, readme_value, actual, line)] for every bound OCCURRENCE that disagrees.

    `readme` / `actual` are injection points for the controls ONLY. The shipped path passes
    neither, so `main()` exercises the real README and the real recompute — a control that
    only ever drove the injected form would prove the comparison and not the tool (#344).
    """
    with open(readme or README, encoding="utf-8") as fh:
        text = fh.read()
    claimed = occurrences(text)
    if not claimed:
        return None, [], {}          # nothing bound: a vacuity, reported separately
    actual = dict(actual) if actual is not None else derived()
    fl = floor_values()
    if fl:
        actual["floor_harness"] = fl.get("harness")
        actual["floor_untested"] = fl.get("untested")
    # (key, LINE) -- not the bare tuple, whose second element is the VALUE, which would
    # order two copies of one key by their numbers instead of by where they appear.
    ordered = sorted(claimed, key=lambda t: (t[0], t[2]))
    bad = [(k, v, actual.get(k), ln) for k, v, ln in ordered if actual.get(k) != v]
    return bad, claimed, actual


def main():
    # `--emit-coverage` FIRST, and cheaply. kiss_trace.py's discover_lint_coverage runs
    # EVERY sibling `kiss_*.py --emit-coverage` to collect lint-enforced clauses -- so a
    # tool that ignores the flag runs its whole main() instead. This one spawns kiss_trace,
    # which spawns this tool again: measured, it took kiss_trace from ~51s to ~200s on the
    # branch that added it, bounded only by discover_lint_coverage's 120s subprocess timeout.
    #
    # The docstring there says a lint lacking the flag "simply contributes no coverage",
    # which is true of the COVERAGE and not of the COST -- the contract assumed lints are
    # cheap to invoke, and this is the first one that is not. Emitting nothing is correct
    # here: this lint enforces no normative clause, it binds a README to the tree.
    if "--emit-coverage" in sys.argv:
        return 0

    try:
        bad, claimed, actual = check()
    except RuntimeError as exc:
        print("KISS conformance README — coverage figures bound to their generators")
        print("=" * 68)
        print("  UNVERIFIED: the generator did not answer, so the README could not be")
        print("  checked. This is NOT drift and the figures below are not evidence:")
        for ln in str(exc).splitlines():
            print("          " + ln)
        print("-" * 68)
        print("  RESULT: VIOLATIONS FOUND")
        return 1
    print("KISS conformance README — coverage figures bound to their generators")
    print("=" * 68)
    if bad is None:
        print("  VACUITY: the README carries NO bound figures. A lint over an empty set")
        print("  passes for the wrong reason — the markers were removed or never added.")
        print("-" * 68)
        print("  RESULT: VIOLATIONS FOUND")
        return 1
    seen = len({k for k, _v, _ln in claimed})
    # KEYS that repeat, not EXTRA OCCURRENCES (#361 review). `len(claimed) - seen`
    # is the occurrence surplus: one key appearing three times gives 2, while exactly
    # ONE key repeats -- the sentence said "repeated key(s)" and the arithmetic
    # ranged over something else. The construct-vs-count error this PR is about,
    # one level in: the arithmetic was right and its subject was not what the
    # sentence claimed.
    dup = repeated_keys(claimed)
    print(f"  {len(claimed):3d} figure(s) bound ({seen} distinct); recomputed from "
          f"kiss_trace.py + COVERAGE_FLOOR.tsv")
    # EVERY occurrence is printed, with its line, not one row per key (#359). A per-key
    # summary is what hid the defect: two markers collapsed into one row and the reader
    # could not see that a second copy existed at all, let alone that it disagreed.
    for k, v, ln in sorted(claimed, key=lambda t: (t[0], t[2])):
        mark = "ok " if v == actual.get(k) else "DRIFT"
        print(f"     [{mark}] {k:22s} L{ln:<4d} README {v:>5}   actual {actual.get(k)}")
    if dup:
        print(f"  ({dup} repeated key(s) — legal, and every copy above was compared.)")
    if bad:
        print("-" * 68)
        print("  DRIFT: a README figure no longer matches what the tree reports. These")
        print("  numbers age silently — nothing else in the repository fails when they do:")
        for k, was, now, ln in bad:
            print(f"          line {ln}: {k} — README says {was}, actual is {now}")
        print("  Update the README (the number AND its `<!-- bound:… -->` marker together).")
    print("-" * 68)
    print("  RESULT: VIOLATIONS FOUND" if bad else "  RESULT: CLEAN")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
