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


class BoundaryMatchTest(unittest.TestCase):
    """#192. Exact-token equality was wrong in one direction and the direction is the point."""

    def test_a_term_spanning_whole_components_IS_a_mention(self):
        """`scatter` IS present in `scatter_oob_writes_are_skipped`.

        The report said "mentions NONE", which is FALSE as stated. Reverse-cited tests are
        named for what they test, so the clause's term is present but embedded -- and that
        is why this bit the CITED tier and not the NAMED one.
        """
        for term, tok in [("scatter", "scatter_oob_writes_are_skipped"),
                          ("index_select", "e2_index_select_u32"),
                          ("softmax", "e2_softmax_norm_axis"),
                          ("norm_axis", "e2_softmax_norm_axis")]:
            self.assertTrue(kt._mentions(term, {tok}), "%r should match %r" % (term, tok))

    def test_a_term_that_is_only_a_PREFIX_is_NOT_a_mention(self):
        """The over-match guard, stated because a later reader will assume it was overlooked.

        `sort` inside `assert_sorted` is a prefix of `sorted`, not a component. A bare
        substring rule admits it, and it admits it in the direction that SHRINKS the alarming
        bucket -- the direction the sink rule says mass flows toward.

        A rule anchored only at the left edge would also admit it. Both ends must align.
        """
        self.assertFalse(kt._mentions("sort", {"assert_sorted"}))
        self.assertFalse(kt._mentions("index", {"reindexed"}))
        self.assertFalse(kt._mentions("axis", {"axistype"}))
        # ...and the paired positive, or the control passes by matching nothing at all
        self.assertTrue(kt._mentions("sort", {"sort_stability"}))

    def test_a_GENUINE_miss_survives_the_boundary_fix(self):
        """THE BORN-RED FOR THE CORRECTION ITSELF.

        The fix moves rows OUT of the miss bucket. Nothing else here proves it cannot move a
        row that belongs there -- and every check that would notice is on the other side of
        the change. A clause whose terms appear nowhere in the test, as a component or
        otherwise, must still be a miss afterwards.
        """
        b, matched = kt.about_bucket({"f8e4m3fn", "oob_policy"}, {"test_ops_add_wraps", "value"})
        self.assertEqual(b, "miss")
        self.assertEqual(matched, set())

    def test_about_bucket_USES_the_boundary_matcher(self):
        """MUTATE THE CALL, NOT ONLY THE FUNCTION.

        The controls above drive `_mentions` directly, and every one of them passes against
        an `about_bucket` that still compares with `t in tokens` — proving the matcher works
        and not that anything calls it. Reverting `about_bucket` to exact equality survived
        the entire sweep until this case existed.
        """
        b, matched = kt.about_bucket({"index_select"}, {"e2_index_select_u32"})
        self.assertEqual(b, "hit", "about_bucket is not using the boundary matcher")
        self.assertEqual(matched, {"index_select"})

    def test_no_terms_is_unmeasurable_not_a_miss(self):
        b, _ = kt.about_bucket(set(), {"anything"})
        self.assertEqual(b, "unmeasurable")


class CitingTestResolutionTest(unittest.TestCase):
    def test_it_uses_the_CITING_test_not_the_section9_row(self):
        """The caveat that leads this increment, as a behavioural control.

        A CITED clause is backed by whichever test CITES it, frequently not the test the §9
        matrix names -- and reverse-only clauses have no §9-named test at all. Resolving to
        the §9 row fails in the REASSURING direction: a §9-named test usually does mention
        its clause's terms, so the miss bucket would quietly approach zero.
        """
        harness = {
            "test_spec_named_row": {"clauses": set(), "tokens": {"structure_key", "wrong"}},
            "the_test_that_cites": {"clauses": {"KISS-X-6.1-0001"}, "tokens": {"right"}},
        }
        toks = kt.citing_tokens("KISS-X-6.1-0001", harness)
        self.assertEqual(toks, {"right"},
                         "resolved to the wrong test — the §9 row's tokens leaked in")
        self.assertIsNone(kt.citing_tokens("KISS-X-9.9-9999", harness),
                          "a clause nothing cites must be None, not an empty set: empty would "
                          "read as UNMEASURABLE and be counted as assessed")

    def test_every_citing_test_contributes(self):
        """A clause cited by two tests is supported if EITHER mentions its terms."""
        harness = {
            "a": {"clauses": {"KISS-X-6.1-0001"}, "tokens": {"alpha"}},
            "b": {"clauses": {"KISS-X-6.1-0001"}, "tokens": {"beta"}},
        }
        self.assertEqual(kt.citing_tokens("KISS-X-6.1-0001", harness), {"alpha", "beta"})


class ReportTest(unittest.TestCase):
    """The shipped path. Every control above drives the helpers directly, so all of them
    would pass against a report that never called them (#344's lesson)."""

    @classmethod
    def setUpClass(cls):
        r = subprocess.run([sys.executable, str(HERE / "kiss_trace.py"), "--report"],
                           capture_output=True, timeout=900,
                           env={**os.environ, "PYTHONIOENCODING": "utf-8"})
        cls.out = r.stdout.decode("utf-8", "replace") + r.stderr.decode("utf-8", "replace")

    def tier(self, label):
        """(population, hit, miss, unmeasurable) for one tier's section."""
        pat = (r"aboutness over the (\d+) " + label + r".*?"
               r"(\d+) the [^\n]*text mentions a term[^\n]*\n"
               r"\s*(\d+) it mentions NONE[^\n]*\n"
               r"\s*(\d+) the clause names no distinctive term")
        m = re.search(pat, self.out, re.S)
        self.assertIsNotNone(m, "no %s aboutness section in the report" % label)
        return tuple(int(g) for g in m.groups())

    def test_BOTH_tiers_are_reported(self):
        """#246 measured NAMED only, and the substring defect was structurally invisible
        there — §9-named tests never embed a clause identifier as a component. An
        instrument's defects show only where its inputs differ from the ones it was built
        against, so both halves of the partition must be run."""
        self.assertIsNotNone(re.search(r"aboutness over the \d+ NAMED", self.out))
        self.assertIsNotNone(re.search(r"aboutness over the \d+ CITED", self.out))

    def test_every_bucket_is_reachable_in_BOTH_tiers(self):
        """#367's per-class pin, now over two tiers.

        A bucket whose count no control pins can be emptied into another for free, and the
        direction that pays is always the one that reads as good news. Pinning only the
        bucket you are worried about is what let a redistribution survive the whole sweep
        last time — and `sum <= total` cannot catch it, because that is exactly what a
        redistribution preserves.
        """
        for label in ("NAMED", "CITED"):
            pop, hit, miss, unmeas = self.tier(label)
            for name, n in (("hit", hit), ("miss", miss), ("unmeasurable", unmeas)):
                self.assertGreater(n, 0, "%s/%s is empty — most likely folded into another "
                                         "bucket, which inflates whichever number reads as "
                                         "good news" % (label, name))
            self.assertLessEqual(hit + miss + unmeas, pop,
                                 "%s buckets exceed the tier they partition" % label)

    def test_the_CITED_section_names_the_CITING_test(self):
        """The label is the contract. If it said "named test" a reader would assume the §9
        row was used, which is the resolution that fails in the reassuring direction."""
        m = re.search(r"aboutness over the \d+ CITED.*?text mentions a term", self.out, re.S)
        self.assertIn("CITING test", m.group(0))

    def test_the_CITED_numbers_match_an_INDEPENDENT_recompute(self):
        """The label control asserts WORDING; this asserts the RESOLUTION.

        Swapping the tool to resolve CITED clauses against their §9 row leaves the label
        untouched, so a wording check cannot see it — and it survived the sweep until this
        existed. Recomputing the buckets here from `citing_tokens` and comparing to the
        report's own numbers binds the printed figure to the resolution that produced it.

        A §9-row resolution would also SHRINK the population, since reverse-only clauses
        have no §9-named test at all — so the mismatch shows up in the totals, not only in
        the split.
        """
        harness = kt.discover_tests(str(HERE.parent / "conformance"))
        terms = {}
        for stem in ("umbrella", "announce", "classify", "ops", "grammar", "contract",
                     "synth", "consume", "emit", "conform"):
            path = HERE.parent / "spec" / (stem + ".md")
            if path.exists():
                r = kt.DocResult(stem)
                kt.parse(str(path), r)
                terms.update(r.terms)
        counts = {"hit": 0, "miss": 0, "unmeasurable": 0}
        for cid in {c for info in harness.values() for c in info["clauses"]}:
            toks = kt.citing_tokens(cid, harness)
            if toks is None or cid not in terms:
                continue
            counts[kt.about_bucket(terms[cid], toks)[0]] += 1
        _pop, hit, miss, unmeas = self.tier("CITED")
        self.assertEqual((hit, miss, unmeas),
                         (counts["hit"], counts["miss"], counts["unmeasurable"]),
                         "the report's CITED numbers do not match a recompute against the "
                         "CITING tests — the tool is resolving to a different test")

    def test_miss_rows_print_their_TERMS(self):
        """The answer to an ungroundable predicate is to EXPOSE, not classify.

        Three groundings for "is this term distinctive" were measured and all three failed,
        so the judgement moves to the reader — which requires the evidence be on the page.
        Without the terms the miss count is a number to trust; with them it is a list to
        triage.
        """
        self.assertRegex(self.out, r"KISS-[A-Z]+-[0-9.]+-\d+\w*\s+terms \[")

    def test_the_section_says_it_does_not_gate(self):
        self.assertIn("REPORTS, never gates", self.out)
        self.assertIn("de-credits nothing", self.out)
        self.assertIn("UNMEASURABLE, not clean", self.out)


if __name__ == "__main__":
    unittest.main()
