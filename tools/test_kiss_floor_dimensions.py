"""The floor's DIMENSION SET is itself checked (#271).

The ratchet validated its NUMBERS and had no check over its own dimension set. A wrong
VALUE was compared and reddened; a WRONG KEY was never compared at all — so deleting the
`proven` row, or typing it `proen`, switched a blocking dimension off while the run
reported CLEAN, exit 0. Measured at origin/main @ 083917d before the fix.

THE DEFECT WAS A HARDCODED TUPLE, NOT A MISSING NAME. `proven` was wired at the call site
as `if "proven" in floor:` while the required-key guard still read
`("harness", "lint", "untested")` — a blocking dimension made opt-in from the very file it
constrains. Adding one name to a second hardcoded tuple would fix today and reproduce this
on the fifth dimension, so `test_every_dimension_is_load_bearing` loops over `DIMENSIONS`
and will fail for any future member that is not actually compared.

Run: python tools/test_kiss_floor_dimensions.py
"""
import pathlib
import re
import subprocess
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import kiss_trace as kt  # noqa: E402

LIVE = {"harness": 380, "lint": 33, "untested": 496, "proven": 0}


def floor_file(rows):
    """Write a floor TSV and read it back through the real parser."""
    d = tempfile.mkdtemp()
    p = pathlib.Path(d) / "COVERAGE_FLOOR.tsv"
    p.write_text("# key\tvalue\n" + "".join("%s\t%d\n" % r for r in rows), encoding="utf-8")
    return kt.read_floor(str(p))


def verdict_for(floor):
    """The ratchet's verdict for a floor, holding every live figure AT that floor.

    Any red is therefore attributable to the DIMENSION SET and nothing else.
    """
    v, _lines = kt.classify_ratchet(floor, LIVE, set(), set(), set(), disk_lint=set())
    return v


class DimensionSetTest(unittest.TestCase):
    def test_control_an_intact_floor_is_at_the_floor(self):
        """Without this, a guard hardcoded to report `incomplete` would pass every case below."""
        floor, problems = floor_file([(d, LIVE[d]) for d in kt.DIMENSIONS])
        self.assertEqual(problems, [])
        self.assertNotEqual(verdict_for(floor), "incomplete")

    def test_every_dimension_is_load_bearing(self):
        """THE GENERALIZING CASE: drop each dimension in turn; each must red.

        This is the test that does not have to be rewritten when a fifth dimension is
        added — it reads DIMENSIONS. A version enumerating the four by hand would pass
        forever while the fifth went unchecked, which is exactly how #271 arrived.

        SCOPE, narrowed after review: this proves each key is PRESENT-or-red. It does NOT
        prove the dimension's value is COMPARED — a dimension added to DIMENSIONS and never
        wired to a comparison passes here while gating nothing, the exact inverse of how
        `proven` arrived. That is `ValueIsComparedTest` below, which has to drive the tool
        end-to-end because `classify_ratchet` never compares `proven` at all.
        """
        for dropped in kt.DIMENSIONS:
            with self.subTest(dropped=dropped):
                rows = [(d, LIVE[d]) for d in kt.DIMENSIONS if d != dropped]
                floor, _ = floor_file(rows)
                self.assertNotIn(dropped, floor, "seed did not apply: row still present")
                self.assertEqual(
                    verdict_for(floor), "incomplete",
                    "dropping `%s` did not red the ratchet — that dimension is not "
                    "actually compared, it is decoration." % dropped)

    def test_a_typo_is_caught_as_missing_AND_unknown(self):
        """A typo leaves the right NUMBER of rows, so a reviewer sees a complete file.

        Both halves must fire: the dimension is missing (the gate is off) and the key is
        unknown (this is why). Reporting only the first would leave a reader hunting for a
        row that is visibly present.
        """
        rows = [(d, LIVE[d]) for d in kt.DIMENSIONS if d != "proven"] + [("proen", 0)]
        floor, problems = floor_file(rows)
        self.assertEqual(len(rows), len(kt.DIMENSIONS), "the file still LOOKS complete")
        self.assertEqual(verdict_for(floor), "incomplete")
        self.assertTrue(any("proen" in p and "unknown" in p for p in problems), problems)

    def test_duplicate_key_is_reported(self):
        """A duplicate leaves the right KEYS, so it is invisible to a key-set check."""
        rows = [(d, LIVE[d]) for d in kt.DIMENSIONS] + [("harness", 999)]
        floor, problems = floor_file(rows)
        self.assertTrue(any("duplicate" in p and "harness" in p for p in problems), problems)

    def test_the_parser_takes_the_LAST_occurrence(self):
        """Pinned because #271's own text said `first`, and the direction matters.

        Last-wins means an appended row silently overrides a reviewed one. Demonstrated
        here rather than asserted in a comment.
        """
        floor, _ = floor_file([(d, LIVE[d]) for d in kt.DIMENSIONS] + [("harness", 999)])
        self.assertEqual(floor["harness"], 999)

    def test_no_second_hardcoded_dimension_tuple_survives(self):
        """The fix is the CONSTANT, not the added name.

        Guards against a future edit re-introducing a literal dimension list beside
        DIMENSIONS — the shape that let a fourth dimension be added to one site and not
        the other.
        """
        src = (HERE / "kiss_trace.py").read_text(encoding="utf-8")
        # CODE ONLY. The first version of this test scanned the whole file and matched the
        # COMMENT above DIMENSIONS that quotes the old tuple to explain the bug — a pattern
        # matching prose and being read as code, which is the same defect as an arity
        # extractor matching the English article "a" in a sentence. Strip comment lines.
        code = "\n".join(l for l in src.splitlines() if not l.lstrip().startswith("#"))
        stale = re.findall(r'\(\s*"harness"\s*,\s*"lint"\s*,\s*"untested"\s*\)', code)
        self.assertEqual(stale, [], "a hardcoded dimension tuple is back beside DIMENSIONS")
        self.assertIn('DIMENSIONS = ("harness", "lint", "untested", "proven")', src)

    def test_that_scan_would_catch_a_real_reintroduction(self):
        """The scan above is only worth having if it fires — and it nearly did not.

        Stripping comments to fix a false positive is one edit away from stripping the
        signal too. This feeds the stripper a line of CODE carrying the old tuple and
        requires it to survive.
        """
        code_line = '    missing = [k for k in ("harness", "lint", "untested") if k not in floor]'
        kept = "\n".join(l for l in [code_line] if not l.lstrip().startswith("#"))
        self.assertTrue(re.search(r'\(\s*"harness"\s*,\s*"lint"\s*,\s*"untested"\s*\)', kept),
                        "the comment-stripper removed a real code occurrence")

    def test_the_live_repo_floor_is_well_formed(self):
        """The real file, not a fixture: no unknown keys, no duplicates, all dimensions present."""
        real = HERE.parent / "conformance" / "COVERAGE_FLOOR.tsv"
        floor, problems = kt.read_floor(str(real))
        self.assertEqual(problems, [])
        self.assertEqual(sorted(floor), sorted(kt.DIMENSIONS))


# --- PRESENCE IS NOT COMPARISON (#271 review) -----------------------------------------
#
# The drop arm above proves each dimension's KEY IS REQUIRED. It does not prove the
# dimension's VALUE IS COMPARED — a dimension added to DIMENSIONS but never wired to a
# comparison passes the drop arm while gating nothing, which is the exact inverse of how
# `proven` arrived. Worse, `verdict_for` reaches only `classify_ratchet`, and
# `classify_ratchet` never compares `proven` at all: that lives at the CALL SITE. So for
# the very dimension this change protects, the drop arm shows presence and never comparison.
#
# These arms therefore run the REAL TOOL END-TO-END against the repo, perturbing one floor
# value at a time. A fixture would answer the wrong question: `floor["lint"]` IS compared on
# the GIT-LESS path (kiss_trace.py:863) and is NOT compared when a --base-ref is supplied,
# so a git-less fixture would report a gate that the shipping path does not have.

# Dimensions whose FLOOR VALUE is NOT count-compared on the git path, with the measurement.
# EXACT-SET, so it fails closed in both directions: a NEW uncompared dimension reds here,
# and a dimension that STARTS being compared also reds, forcing a deliberate edit.
VALUE_NOT_COMPARED = {
    "lint": "measured: floor lint 33 -> 38 with live 33 exits 0 and prints `at the floor - "
            "... lint 33 ...`, i.e. the LIVE number, so the mismatch is invisible. On the git "
            "path `floor['lint']` is read only inside the git-less branch (:863) and as the "
            "`counts_at_floor` precondition (:907) — neither can produce a red. The lint "
            "dimension is gated as a SET against the base ledger, which is the real check; "
            "whether the COUNT should also red is a design question, raised on #271 rather "
            "than decided here.",
}

PERTURB = {"harness": +1, "lint": +1, "untested": -1, "proven": +1}


def ratchet_with_floor(text):
    """Run the REAL ratchet against the repo with a perturbed floor, then restore."""
    floor = pathlib.Path(HERE.parent / "conformance" / "COVERAGE_FLOOR.tsv")
    orig = floor.read_text(encoding="utf-8")
    try:
        floor.write_text(text, encoding="utf-8")
        r = subprocess.run([sys.executable, str(HERE / "kiss_trace.py"), "--ratchet",
                            "--base-ref", "origin/main"],
                           capture_output=True, text=True, timeout=300)
        return r.returncode, r.stdout
    finally:
        floor.write_text(orig, encoding="utf-8")
        assert floor.read_text(encoding="utf-8") == orig, "floor restore FAILED"


class ValueIsComparedTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.floor_text = (HERE.parent / "conformance" / "COVERAGE_FLOOR.tsv").read_text(
            encoding="utf-8")

    def _perturbed(self, dim, delta):
        tab = chr(9)
        cur = int([l.split(tab)[1] for l in self.floor_text.splitlines()
                   if l.split(tab)[0].strip() == dim][0])
        old, new = "%s%s%d" % (dim, tab, cur), "%s%s%d" % (dim, tab, cur + delta)
        text = self.floor_text.replace(old, new, 1)
        self.assertNotEqual(text, self.floor_text, "seed did not apply for %s" % dim)
        return text

    def test_control_the_unperturbed_floor_is_clean(self):
        """Without this every arm below passes against a tool that is simply always red.

        It also fails loudly on a stale base — `origin/main` not being an ancestor makes the
        ratchet return INCONCLUSIVE (exit 2), which is non-zero and would satisfy a careless
        `assertNotEqual(rc, 0)` in every arm below. Rebase, then re-run.
        """
        rc, out = ratchet_with_floor(self.floor_text)
        self.assertEqual(rc, 0, "control is not clean — rebase onto origin/main first:\n"
                                + out[-600:])

    def test_every_dimensions_value_is_compared(self):
        """Perturb each floor value by one; a compared dimension must go RED (exit 1).

        Exit 1 specifically, never merely non-zero: exit 2 is the ratchet's INCONCLUSIVE
        refusal and is reachable for reasons that have nothing to do with the perturbation.
        """
        for dim, delta in PERTURB.items():
            with self.subTest(dim=dim):
                rc, out = ratchet_with_floor(self._perturbed(dim, delta))
                if dim in VALUE_NOT_COMPARED:
                    self.assertEqual(
                        rc, 0,
                        "`%s` is recorded as NOT count-compared but the perturbation red — "
                        "the gap has been closed, so remove it from VALUE_NOT_COMPARED.\n%s"
                        % (dim, out[-400:]))
                else:
                    self.assertEqual(
                        rc, 1,
                        "perturbing `%s` by %+d did not red the ratchet (exit %d). Its key is "
                        "required but its VALUE is not compared — the dimension gates "
                        "nothing.\n%s" % (dim, delta, rc, out[-400:]))

    def test_the_uncompared_set_is_exact(self):
        """Fails closed both ways, and every entry carries its measurement."""
        self.assertTrue(set(VALUE_NOT_COMPARED) <= set(kt.DIMENSIONS))
        for dim, why in VALUE_NOT_COMPARED.items():
            self.assertIn("measured", why, "%s is excepted without a measurement" % dim)
        self.assertEqual(sorted(PERTURB), sorted(kt.DIMENSIONS),
                         "a dimension has no perturbation arm")


if __name__ == "__main__":
    unittest.main()
