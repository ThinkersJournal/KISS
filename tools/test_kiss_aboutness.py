"""Controls for the NAMED-tier aboutness signal (#246).

`kiss_trace` binds a clause to a test BY NAME in both directions and reads no assertion, so
a clause can name a test that exists, compiles and passes while checking something else --
and be counted as backed. Four such instances surfaced in a single PR (#242).

    THE HEADLINE FIGURE COUNTS CLAUSES WHOSE NAMED TEST EXISTS, NOT CLAUSES WHOSE NAMED
    TEST CHECKS THEM.

This does not close that -- nothing short of reading assertions can. It MEASURES it: for
each NAMED clause, does the named test's text even mention a distinctive term the clause
names? That is the weakest evidence of aboutness there is, and still strictly more than a
name match.

WHAT THESE CONTROLS PROTECT, in order of how badly each would fail silently:

1. THE CLAUSE'S OWN `*Test:*` NAME MUST BE EXCLUDED FROM ITS TERMS. That name is backticked
   in the clause and is present in the test by construction, so leaving it in makes every
   clause overlap. Measured on the live tree:

       test name EXCLUDED   hit=148  miss=15
       test name INCLUDED   hit=241  miss=0     <- reports nothing, looks like it works

   A signal that cannot come out negative is not a signal, and this one fails in the
   direction that looks like good news.

2. NO TERMS IS UNMEASURABLE, NOT CLEAN. A clause naming no backticked identifier cannot be
   assessed on this axis, and folding it into the "mentions a term" bucket would inflate
   the reassuring number with cases nobody measured.

3. THE SIGNAL DE-CREDITS NOTHING. Zero overlap does not prove a test is off-subject; a
   recognizer's silence is not a finding. The buckets must partition the measured set so
   no clause is dropped or double-counted, and no coverage number may move.

Run: python tools/test_kiss_aboutness.py
"""
import os
import pathlib
import re
import subprocess
import sys
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import kiss_trace as kt  # noqa: E402


class ClauseTermsTest(unittest.TestCase):
    def test_the_clauses_OWN_test_name_is_excluded(self):
        """Control 1 — the one that decides whether this measures anything at all.

        `*Test:* `test_ops_bf16_layout`` is backticked, so without the exclusion the term
        set contains the test's own name, which the test body contains by construction.
        Every clause would then "mention a term it names" and the miss bucket would be
        empty — a clean-looking report from a measure that ranges over nothing.
        """
        block = ("- **KISS-OPS-6.16-0003** — `bf16` MUST be encoded as 1-sign / 8-exp.\n"
                 "  *Test:* `test_ops_bf16_layout`.\n")
        terms = kt.clause_terms(block)
        self.assertIn("bf16", terms)
        self.assertNotIn("test_ops_bf16_layout", terms,
                         "the clause's own named test leaked into its terms — every clause "
                         "will now overlap and the signal reports nothing")

    def test_a_clause_naming_no_distinctive_term_yields_NOTHING(self):
        """Control 2. Such a clause is UNMEASURABLE on this axis.

        The report keeps it in its own bucket for exactly this reason: counting it as a hit
        would pad the reassuring number with cases no one assessed.
        """
        block = ("- **KISS-OPS-6.0-0001** — An implementation MUST be deterministic.\n"
                 "  *Test:* `test_ops_determinism`.\n")
        self.assertEqual(kt.clause_terms(block), set())

    def test_stopwords_and_clause_ids_carry_no_signal(self):
        """A term present in most tests separates nothing.

        Without the stop list, `` `test` `` or `` `assert_eq` `` would match almost any body
        and the overlap count would be inflated by tokens that discriminate nothing — the
        same silent inflation as control 2, arriving through vocabulary instead of bucketing.
        """
        block = ("- **KISS-OPS-9.9-9999** — see `KISS-OPS-6.1-0001`, use `assert_eq` in a "
                 "`test` with `vec` and `u32`.\n  *Test:* `test_x`.\n")
        self.assertEqual(kt.clause_terms(block), set())

    def test_a_multiword_backtick_span_yields_each_identifier(self):
        """`` `structure_key` `` and `` `f8e4m3fn / f8e5m2` `` must both contribute.

        A span-level match would miss a test that names one of a pair, which is the common
        case in this corpus.
        """
        block = "- **X** — `f8e4m3fn / f8e5m2` and `structure_key`.\n  *Test:* `test_x`.\n"
        self.assertEqual(kt.clause_terms(block), {"f8e4m3fn", "f8e5m2", "structure_key"})


class ReportTest(unittest.TestCase):
    """The shipped path. Every control above drives `clause_terms` directly, so all of them
    would pass against a report that never called it (#344's lesson)."""

    @classmethod
    def setUpClass(cls):
        r = subprocess.run([sys.executable, str(HERE / "kiss_trace.py"), "--report"],
                           capture_output=True, timeout=900,
                           env={**os.environ, "PYTHONIOENCODING": "utf-8"})
        cls.out = r.stdout.decode("utf-8", "replace") + r.stderr.decode("utf-8", "replace")

    def _n(self, pattern):
        m = re.search(pattern, self.out)
        self.assertIsNotNone(m, "report line missing: %s" % pattern)
        return int(m.group(1))

    def test_the_report_prints_the_three_buckets(self):
        hit = self._n(r"(\d+) the named test's text mentions a term")
        miss = self._n(r"(\d+) it mentions NONE")
        unmeas = self._n(r"(\d+) the clause names no distinctive term")
        self.assertGreater(hit + miss + unmeas, 0)
        # EVERY bucket must be reachable, not just this one. The first version asserted
        # only `miss > 0`, and a mutation that folded UNMEASURABLE into the hit bucket
        # SURVIVED the whole sweep: the sum is unchanged, `miss` is untouched, and the
        # reassuring number silently absorbs 78 clauses nobody assessed.
        #
        # The asymmetry was the defect. A bucket whose count no control pins can be emptied
        # into another one for free, and the direction that pays is always into the bucket
        # that reads as good news.
        #
        # Each of these going to zero is a real event: it means the corpus changed shape, or
        # a bucket stopped being populated. Both deserve a human look rather than a green run.
        self.assertGreater(miss, 0,
                           "no NAMED clause lacks overlap — either the corpus is perfect or "
                           "the exclusion broke and every clause now matches")
        self.assertGreater(unmeas, 0,
                           "no clause is UNMEASURABLE — most likely that bucket was folded "
                           "into the hit count, which inflates the reassuring number with "
                           "cases nobody assessed")
        self.assertGreater(hit, 0, "no clause overlaps — the term extractor is returning "
                                   "nothing and the whole measure is vacuous")

    def test_the_buckets_PARTITION_the_named_tier(self):
        """No clause dropped, none double-counted, and none invented.

        The three buckets must sum to at most the NAMED tier — a sum ABOVE it would mean a
        clause counted twice, which would make the reassuring bucket the one that grew.
        """
        named = self._n(r"aboutness signal over those (\d+) NAMED")
        total = (self._n(r"(\d+) the named test's text mentions a term")
                 + self._n(r"(\d+) it mentions NONE")
                 + self._n(r"(\d+) the clause names no distinctive term"))
        self.assertLessEqual(total, named,
                             "the buckets exceed the tier they partition: %d > %d"
                             % (total, named))

    def test_the_section_says_it_does_not_gate(self):
        """The wording is load-bearing, not decoration.

        A population offered without that caveat gets read as a defect list, and the first
        thing someone does with a defect list is de-credit from it — on a recognizer's
        silence, which is the one move #187 rules out.
        """
        self.assertIn("REPORTS, never gates", self.out)
        self.assertIn("de-credits nothing", self.out)
        self.assertIn("UNMEASURABLE, not clean", self.out)


if __name__ == "__main__":
    unittest.main()
