"""Mutate every named constant and report the ones NO test reds on (#409 Phase 1).

Found the #422 class in one run: three live shape-expression wire tags — `Add`, `Sub`, `Mul` —
whose byte values can be changed to anything with zero tests red anywhere. On a
byte-deterministic freeze surface that is silent wire divergence, not a red anyone interprets.

SCOPE, ruled and deliberately narrow. This detects a constant nothing pins. It does NOT detect
the #417 class — an assertion that cannot fail because a neighbouring assertion's precondition
guarantees its subject — because that is a semantic relation between assertions rather than a
shape. Those stay adversarial.

⚠️ WHAT THIS TOOL CANNOT SEE, stated because a sweep that reports a clean number invites the
reader to conclude more than it measured: a detector can find a constant that nothing pins. It
cannot find a constant that IS pinned, by a test that asserts something TRUE ABOUT A SMALLER SET
than the reader assumes.

FOUR VERDICTS, kept distinct because the third fails silent and reads exactly like the second:

    GUARDED       mutation applied, at least one test reds        -> the constant is pinned
    UNGUARDED     mutation applied, NOTHING reds                  -> the finding
    NOT-APPLIED   the edit did not land where it was aimed        -> NOT a result
    INVALID       the mutated tree does not compile               -> NOT a result

⚠️ AND UNGUARDED SPLITS IN TWO. A constant that is reserved and never emitted cannot be pinned by
any test, and counting it as a defect inflates the rate — `TAG_REDUCE` did exactly that in the
Phase 0 sample, by a third. Declare those in `conformance/UNPINNABLE_CONSTANTS.tsv` with a reason;
the sweep reports them separately and never as findings.

Run: python tools/kiss_constant_sweep.py --only shape_expr
"""
import argparse
import io
import os
import re
import subprocess
import sys
import time

if not __debug__:
    raise SystemExit(
        "refusing to run under -O/PYTHONOPTIMIZE: `assert` is stripped, so these "
        "checks would report success having verified nothing")

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
SRC_DIR = os.path.join(ROOT, "conformance", "src")
UNPINNABLE = os.path.join(ROOT, "conformance", "UNPINNABLE_CONSTANTS.tsv")

RE_CONST = re.compile(r"^\s*pub const (\w+)\s*:\s*([^=]+?)\s*=\s*(.+?);\s*(?://.*)?$")


def discover(only=None):
    """Every `pub const` as (relpath, line_index, name, type, value). Located by LINE,
    never by name: five constant names in this tree are defined in more than one file
    (`SCHEMA_VERSION`, `MAX_RANK`, `MAX_OPERANDS`, `MAGIC`, `MAX_STRUCTURE_KEY_LEN`), so a
    name-keyed edit can silently mutate the wrong one and report a verdict about a
    constant it never touched."""
    out = []
    for dirpath, _dirs, files in os.walk(SRC_DIR):
        for fn in sorted(files):
            if not fn.endswith(".rs"):
                continue
            rel = os.path.relpath(os.path.join(dirpath, fn), ROOT).replace("\\", "/")
            if only and only not in rel:
                continue
            lines = io.open(os.path.join(dirpath, fn), encoding="utf-8").read().split("\n")
            for i, line in enumerate(lines):
                m = RE_CONST.match(line)
                if m:
                    out.append((rel, i, m.group(1), m.group(2).strip(), m.group(3).strip()))
    return out


def alternative(value, ty):
    """A DIFFERENT literal of the same type, or None if this constant is not mutable
    by a rule simple enough to be trustworthy. Returning None is a refusal, recorded as
    unmeasured -- never guessed into a mutation whose validity we cannot argue."""
    v = value.strip()
    if re.fullmatch(r"0x[0-9A-Fa-f]+", v):
        n = int(v, 16)
        alt = (n ^ 0x40) & 0xFF if n <= 0xFF else n ^ 0x4000
        return None if alt == n else ("0x%02X" % alt if n <= 0xFF else "0x%X" % alt)
    if re.fullmatch(r"\d+", v):
        return str(int(v) + 7)
    if re.fullmatch(r'"[^"\\]*"', v):
        return '"%s_MUTATED"' % v[1:-1]
    return None


def load_unpinnable():
    """{(relpath, name): reason} -- constants declared impossible to pin, with a reason."""
    out = {}
    if not os.path.exists(UNPINNABLE):
        return out
    for line in io.open(UNPINNABLE, encoding="utf-8").read().split("\n"):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) >= 3:
            out[(parts[0].strip(), parts[1].strip())] = parts[2].strip()
    return out


def apply_and_verify(path, idx, old_line, new_line):
    """Write the mutation and PROVE it landed at the intended line.

    ⚠️ A harness that reports NO-OP when the edit never applied is indistinguishable from
    one reporting a genuine survivor -- measured during Phase 0, where a `replace` matched a
    different occurrence and a KILL was recorded as a NO-OP. So the edit is verified in
    place, and the rest of the file is verified UNCHANGED, before any verdict is believed.
    """
    full = os.path.join(ROOT, path)
    before = io.open(full, encoding="utf-8").read()
    lines = before.split("\n")
    if lines[idx] != old_line:
        return None, "the target line moved under us"
    lines[idx] = new_line
    after = "\n".join(lines)
    io.open(full, "w", encoding="utf-8", newline="").write(after)
    check = io.open(full, encoding="utf-8").read().split("\n")
    if check[idx] != new_line:
        return before, "the edit did not land at line %d" % (idx + 1)
    if len(check) != len(lines):
        return before, "the file changed length"
    for j, (a, b) in enumerate(zip(check, before.split("\n"))):
        if j != idx and a != b:
            return before, "line %d changed too -- the edit was not isolated" % (j + 1)
    return before, None


def run_suite(timeout):
    r = subprocess.run(["cargo", "test"], capture_output=True, text=True,
                       cwd=os.path.join(ROOT, "conformance"), timeout=timeout)
    out = r.stdout + r.stderr
    if "error[E" in out or "could not compile" in out:
        return "INVALID", []
    red = sorted(set(re.findall(r"^---- (\S+) stdout ----", out, re.M)))
    return ("GUARDED" if red else "UNGUARDED"), red


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--only", help="substring of the source path to restrict the sweep to")
    ap.add_argument("--timeout", type=int, default=1800)
    ap.add_argument("--limit", type=int, default=0, help="stop after N constants (0 = all)")
    args = ap.parse_args(argv)

    consts = discover(args.only)
    unpinnable = load_unpinnable()
    print("-" * 74)
    print("  CONSTANT SWEEP (#409 Phase 1)")
    print("  scope             %s" % (args.only or "conformance/src (all)"))
    print("  constants found   %d" % len(consts))
    print("  declared unpinnable %d" % len(unpinnable))

    todo, unmeasured = [], []
    for rel, idx, name, ty, val in consts:
        alt = alternative(val, ty)
        if alt is None:
            unmeasured.append((rel, name, val, "no trustworthy mutation rule for this literal"))
        else:
            todo.append((rel, idx, name, ty, val, alt))
    if args.limit:
        todo = todo[:args.limit]

    print("  mutable           %d" % len(todo))
    print("  UNMEASURED        %d  (reported, never counted as clean)" % len(unmeasured))
    print("-" * 74)

    results, t0 = [], time.time()
    for rel, idx, name, ty, val, alt in todo:
        full = os.path.join(ROOT, rel)
        old_line = io.open(full, encoding="utf-8").read().split("\n")[idx]
        new_line = old_line.replace("= " + val, "= " + alt, 1)
        if new_line == old_line:
            results.append((rel, name, val, "NOT-APPLIED", ["value not substitutable in place"]))
            continue
        before, err = apply_and_verify(rel, idx, old_line, new_line)
        try:
            if err:
                verdict, red = "NOT-APPLIED", [err]
            else:
                verdict, red = run_suite(args.timeout)
        finally:
            if before is not None:
                io.open(full, "w", encoding="utf-8", newline="").write(before)
        results.append((rel, name, val, verdict, red))
        print("  %-34s %-11s %s" % ("%s::%s" % (os.path.basename(rel), name), verdict,
                                    ("%d red" % len(red)) if verdict == "GUARDED" else
                                    (red[0][:44] if red else "")))

    guarded = [r for r in results if r[3] == "GUARDED"]
    invalid = [r for r in results if r[3] == "INVALID"]
    notapplied = [r for r in results if r[3] == "NOT-APPLIED"]
    ung = [r for r in results if r[3] == "UNGUARDED"]
    ung_declared = [r for r in ung if (r[0], r[1]) in unpinnable]
    ung_live = [r for r in ung if (r[0], r[1]) not in unpinnable]

    print("-" * 74)
    print("  GUARDED                       %d" % len(guarded))
    print("  UNGUARDED, declared unpinnable %d   (not findings -- reason on file)" % len(ung_declared))
    print("  UNGUARDED, LIVE                %d   <-- the findings" % len(ung_live))
    print("  NOT-APPLIED                    %d   (not a result)" % len(notapplied))
    print("  INVALID (would not compile)    %d   (not a result)" % len(invalid))
    print("  elapsed                        %.0fs" % (time.time() - t0))
    for rel, name, val, _v, _r in ung_live:
        print("      %s::%s = %s" % (rel, name, val))
    if unmeasured:
        print()
        print("  ⚠️ UNMEASURED (%d) -- a FLOOR, not a total. These were never driven to a" % len(unmeasured))
        print("     verdict, so a clean result above does not cover them:")
        for rel, name, val, why in unmeasured[:8]:
            print("      %s::%s = %s   (%s)" % (os.path.basename(rel), name, val[:28], why))
    return 1 if ung_live else 0


if __name__ == "__main__":
    sys.exit(main())
