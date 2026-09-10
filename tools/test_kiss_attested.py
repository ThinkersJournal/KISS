"""Genuineness controls for the `attested` ledger category (#505).

The category exists because 20 clauses across seven sub-standards DECLARE THEMSELVES
checklist gates -- their `*Test:*` names a human AUDIT procedure, not a function -- and
the ledger recorded all 20 as bare `untested`, indistinguishable from "nobody wrote it
yet". The burn-down target of 0 was therefore unreachable by construction, and nothing
said so.

    A DERIVED CATEGORY IS ONLY WORTH HAVING IF IT CANNOT BE FORGED.

Moving 20 clauses out of `untested` has the SAME COUNT SIGNATURE as silently dropping 20
clauses, and the ratchet cannot tell them apart -- it sees `untested 501 -> 481` and asks
for a floor bump either way. These are the gate that discriminates:

  * `attested` in the ledger must equal the set the SPEC declares -- both directions, so
    neither a hand-added row nor a spec clause the ledger forgot can pass.
  * a spec that stops calling a clause a checklist gate must drop the category, with
    nobody needing to remember (the drift this closes, running in reverse).

Run: python tools/test_kiss_attested.py
"""
import os
import pathlib
import sys
import unittest

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(HERE))

import kiss_trace as kt  # noqa: E402


def _spec_declared():
    """The clause ids whose OWN BLOCK declares a checklist gate, read from the spec."""
    found = set()
    for stem in kt.SPECS:
        res = kt.DocResult(stem)
        kt.parse(os.path.join(ROOT, "spec", stem + ".md"), res)
        found |= res.attested
    return found


def _ledger_attested():
    path = os.path.join(ROOT, "conformance", "UNBACKED.tsv")
    out = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            if line.startswith("#"):
                continue
            parts = line.rstrip("\n").split("\t")
            if len(parts) >= 3 and parts[2] == "attested":
                out.add(parts[0])
    return out


class AttestedIsDerivedNotCurated(unittest.TestCase):

    def test_ledger_attested_equals_spec_declared(self):
        """Both directions. A one-way check would let the ledger keep a row for a clause
        the spec no longer declares -- which is the exact drift this category closes."""
        spec, ledger = _spec_declared(), _ledger_attested()
        self.assertEqual(
            spec, ledger,
            "the ledger's `attested` set and the spec's declared checklist gates "
            f"disagree.\n  in spec, not ledger: {sorted(spec - ledger)}\n"
            f"  in ledger, not spec: {sorted(ledger - spec)}")

    def test_the_set_is_not_empty(self):
        """A vacuous equality passes when BOTH sides are empty, and an empty recognizer
        would report a clean gate while measuring nothing (#456's shape). The count is
        not pinned -- that would fail on every legitimate spec edit -- but zero is not a
        state this category can honestly be in while the spec still says the words."""
        self.assertGreater(len(_spec_declared()), 0,
                           "no clause declares a checklist gate; the recognizer is dead "
                           "or the spec convention changed")

    def test_recognizer_matches_the_real_declaring_form(self):
        """BORN-RED CONTROL. The recognizer is a regex over spec prose, and a regex that
        silently matches nothing is this repo's most-repeated defect -- this one shipped
        with `\b` mis-escaped to a literal backspace and derived 0 while parsing fine."""
        self.assertTrue(kt.RE_CHECKLIST_GATE.search("(checklist gate; AUDIT-signed)"))
        self.assertTrue(kt.RE_CHECKLIST_GATE.search(
            "(checklist gate; signed by the AUDIT role, not DESIGN)"))
        self.assertIsNone(kt.RE_CHECKLIST_GATE.search("a checklist gate"),
                          "the form is parenthesised; bare prose must NOT match")
        self.assertIsNone(kt.RE_CHECKLIST_GATE.search("(checklist)"))

    def test_attested_clauses_are_unbacked(self):
        """A checklist gate whose named test EXISTS is not a checklist gate any more --
        either the spec label is stale or a real test arrived. Either way a human must
        look, so this fails rather than silently preferring one reading."""
        harness = kt.discover_tests(os.path.join(ROOT, "conformance"))
        bad = []
        for stem in kt.SPECS:
            res = kt.DocResult(stem)
            kt.parse(os.path.join(ROOT, "spec", stem + ".md"), res)
            for cid, _ln, tests in res.body:
                if cid in res.attested:
                    for t in tests:
                        if t in harness:
                            bad.append((cid, t))
        self.assertEqual(bad, [], f"clauses labelled checklist gates have a real test: {bad}")


class AttestedIsAccountedFor(unittest.TestCase):

    def test_category_is_in_the_vocabulary(self):
        self.assertIn("attested", kt.CATEGORIES)

    def test_category_appears_in_the_ledger_header(self):
        """A category absent from the header is undocumented for every future reader."""
        self.assertIn("attested", kt.LEDGER_HEADER)

    def test_attested_is_not_counted_as_the_real_gap(self):
        """The point of the category: these can never be burned down, so counting them in
        the number that `must reach 0` makes the target unreachable by construction."""
        self.assertEqual(kt.untested_count({"attested": ["X"], "untested": []}), 0)
        self.assertEqual(kt.untested_count({"untested": ["X"], "attested": ["Y"]}), 1)


if __name__ == "__main__":
    unittest.main(verbosity=2)
