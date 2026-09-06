r"""Is every citation keyword written in its CANONICAL form? (#488)

⚠️ THE CONVENTION HAD NO READER, WHICH IS WHY THIS EXISTS. `kiss_trace` recognises citations with
`(?:Backs|Enforces)\b[:\s]*` -- the colon is OPTIONAL and matching is case-insensitive -- while
`Proven:` is recognised with the colon REQUIRED. So the tree carried two enforcement policies and
CONTRIBUTING.md convention 15 named the pair as "a `Backs:` / `Enforces` keyword", spelling one
with a colon and one without IN THE SAME PHRASE. An author following the documentation could
write either, and four `Backs` sites did.

⚠️ THIS LINT DOES NOT CREDIT OR DE-CREDIT ANYTHING, AND THAT SEPARATION IS THE POINT. The
obvious alternative -- tightening `[:\s]*` to `:\s*` -- would add a check and change coverage as
a SIDE EFFECT: sites written in the loose form would silently lose their backing, `untested`
would move, and the ratchet would report a number whose true cause was a regex edit, in a form
neither the harness nor the untested dimension can distinguish from a real regression or a real
arrival. DETECTION AND CREDITING MUST STAY SEPARATE INSTRUMENTS. The recognizer stays permissive
so nothing is ever de-credited by a spelling rule; this reports instead.

Run: python tools/kiss_cite_form.py [--strict]
"""
import argparse
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

# ⚠️ REUSE kiss_trace's CLAUSE_ID, never a second spelling of it. A private id pattern here could
# recognise a different set of citations than the tool whose convention this polices, and the
# disagreement would then be between two PARSERS rather than between the tree and its convention.
sys.path.insert(0, HERE)
import kiss_trace as kt  # noqa: E402

# The canonical form: every citation keyword is followed IMMEDIATELY by a colon.
#
# ⚠️ CHOSEN, NOT DISCOVERED, and the alternative is worth recording. The tree's three keywords had
# three near-unanimous but DIFFERENT habits (Backs: 54/58 with a colon, Enforces 79/79 without,
# Proven 20/20 with). A lint could have blessed each keyword's own majority -- but a convention
# whose rule cannot be stated in one sentence is exactly what produced this issue: nobody can
# derive an exception, so nobody follows it, and the fourth keyword (#498) would have arrived
# into the same fog. One rule, no exceptions, is the only form a new keyword can be spelled
# correctly against without being told.
# ⚠️ `Supplements` is RESERVED for #498 and is NOT yet a crediting keyword -- `kiss_trace`
# does not recognise it. It is listed here anyway so its FORM is policed from the moment it
# first appears, rather than after four instances have set a habit. That is the whole finding
# of #488 applied forward: the other three keywords got their reader late.
CANONICAL = ("Backs", "Enforces", "Proven", "Supplements")

# A keyword followed by anything other than a colon, then a clause id.
RE_BAD = kt.re.compile(
    r'\b(' + '|'.join(CANONICAL) + r')(\s+)(' + kt.CLAUSE_ID + r')')
RE_ANY = kt.re.compile(
    r'\b(' + '|'.join(CANONICAL) + r')(:?\s*)(' + kt.CLAUSE_ID + r')')


def scan(root):
    """Every citation-keyword site under conformance/, as (path, line_no, keyword, ok)."""
    sites = []
    conf = os.path.join(root, "conformance")
    for dirpath, _dirs, files in os.walk(conf):
        for fn in files:
            if not fn.endswith(".rs"):
                continue
            p = os.path.join(dirpath, fn)
            with open(p, encoding="utf-8") as fh:
                for i, line in enumerate(fh, 1):
                    for m in RE_ANY.finditer(line):
                        ok = m.group(2).startswith(":")
                        sites.append((os.path.relpath(p, root).replace("\\", "/"),
                                      i, m.group(1), m.group(3), ok))
    return sites


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--strict", action="store_true",
                    help="exit 1 on any non-canonical citation (CI uses this)")
    args = ap.parse_args()

    sites = scan(ROOT)
    bad = [s for s in sites if not s[4]]

    print("-" * 74)
    print("  CITATION KEYWORD FORM (#488)")
    # ⚠️ THE POPULATION IS THE CONTROL. A recognizer that silently matches nothing reports a
    # clean tree, and "no violations" is byte-identical to "no citations found". Printing the
    # scanned count makes a narrowed pattern show up as a SMALLER NUMBER rather than as a green
    # run -- the failure mode a pass/fail line cannot express.
    print(f"  citation sites scanned  {len(sites)}")
    for kw in CANONICAL:
        n = sum(1 for s in sites if s[2] == kw)
        print(f"    {kw + ':':<12} {n:>4}")
    print(f"  non-canonical           {len(bad)}")

    if not sites:
        print("-" * 74)
        print("  COULD NOT MEASURE: zero citation sites found under conformance/.")
        print("  That is not a clean tree -- the tree has hundreds. The extractor is broken,")
        print("  and reporting 0 violations here would be a FALSE CLEAN.")
        print("-" * 74)
        return 2

    if bad:
        print("-" * 74)
        for path, ln, kw, cid, _ in bad:
            print(f"  {path}:{ln}")
            print(f"      `{kw} {cid}`  ->  `{kw}: {cid}`")
        print("-" * 74)
        print("  NON-CANONICAL CITATION FORM. These are RECOGNISED today -- `kiss_trace` accepts")
        print("  `Backs`/`Enforces` with or without the colon -- so this costs no coverage and")
        print("  is not urgent. It is reported because the convention had no reader: the four")
        print("  original drifted sites were written by authors who had read it, and a keyword")
        print("  nobody checks is a keyword whose next spelling is a coin flip.")
        print("-" * 74)
        return 1 if args.strict else 0

    print("-" * 74)
    print(f"  CLEAN - all {len(sites)} citation sites use the canonical `Keyword:` form.")
    print("-" * 74)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
