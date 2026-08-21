"""Discrimination controls for the comparator-blindness lint (§6.8-0012).

Every case must FAIL on the defect and PASS without it. A control that passes in
both states is measuring the fixture, not the rule -- which is the failure this
lint exists to catch, so it would be a poor joke to commit it here.

Run: python tools/test_kiss_comparators.py
"""
import pathlib
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import kiss_comparators as kc  # noqa: E402

DECLARED = """\
- **KISS-CONFORM-9.9-0001** - The **example** comparator MUST be byte-identical.
  **Normalizes:** nothing.
  *Test:* `test_example`.
"""

UNDECLARED = """\
- **KISS-CONFORM-9.9-0002** - The **other** comparator MUST compare within a tolerance.
  *Test:* `test_other`.
"""

NO_COMPARATOR = """\
- **KISS-CONFORM-9.9-0003** - An unrelated obligation about matrix serialization.
  *Test:* `test_unrelated`.
"""


def scan_text(body):
    with tempfile.TemporaryDirectory() as td:
        p = pathlib.Path(td) / "conform.md"
        p.write_text(body, encoding="utf-8")
        return kc.scan(str(p))


class BlindnessLintTest(unittest.TestCase):
    def test_undeclared_comparator_is_caught(self):
        """The whole point: a comparison relation declaring nothing is a violation."""
        declared, undeclared, excluded, stale = scan_text(DECLARED + UNDECLARED)
        self.assertEqual(undeclared, ["KISS-CONFORM-9.9-0002"])
        self.assertEqual(declared, ["KISS-CONFORM-9.9-0001"])

    def test_declared_comparator_passes(self):
        """Control. Without this, a lint hardcoded to fail would pass the case above."""
        declared, undeclared, _, _ = scan_text(DECLARED)
        self.assertEqual(declared, ["KISS-CONFORM-9.9-0001"])
        self.assertEqual(undeclared, [])

    def test_a_clause_naming_no_comparator_is_out_of_scope(self):
        """The detector is over-broad by design; it must still not be UNBOUNDED.

        If every clause were in scope, the exclusion list would have to enumerate the
        whole document and would stop being reviewable.
        """
        declared, undeclared, excluded, _ = scan_text(NO_COMPARATOR)
        self.assertEqual((declared, undeclared, excluded), ([], [], []))

    def test_stale_exclusion_is_caught(self):
        """An exclusion for a clause no longer in scope exempts the WRONG clause later.

        This fired twice while the lint was being written -- both times on entries the
        author had assumed rather than measured -- which is the argument for keeping it.
        """
        kc.SELECTS_ONLY["KISS-CONFORM-9.9-0404"] = "does not exist"
        try:
            _, _, _, stale = scan_text(DECLARED)
            self.assertIn("KISS-CONFORM-9.9-0404", stale)
        finally:
            del kc.SELECTS_ONLY["KISS-CONFORM-9.9-0404"]

    def test_excluded_clause_is_not_a_violation(self):
        """An explicit exclusion suppresses the finding -- and only an explicit one."""
        kc.SELECTS_ONLY["KISS-CONFORM-9.9-0002"] = "test fixture"
        try:
            _, undeclared, excluded, _ = scan_text(DECLARED + UNDECLARED)
            self.assertEqual(undeclared, [])
            self.assertEqual(excluded, ["KISS-CONFORM-9.9-0002"])
        finally:
            del kc.SELECTS_ONLY["KISS-CONFORM-9.9-0002"]

    def test_the_real_document_is_clean(self):
        """Convention 9: assert the thing under test is actually in the state claimed.

        Without this the fixture cases could all pass against a spec/conform.md that
        violates the clause the lint enforces.
        """
        declared, undeclared, _, stale = kc.scan()
        self.assertEqual(undeclared, [], f"spec/conform.md has undeclared comparators: {undeclared}")
        self.assertEqual(stale, [], f"stale exclusions in SELECTS_ONLY: {stale}")
        self.assertGreaterEqual(len(declared), 6, "the real document must carry declarations")


class DeclarationCompletenessTest(unittest.TestCase):
    """§6.8-0012's own declaration must name every bucketed component of the key (#274).

    The defect this closes was a check TRUE AS FAR AS IT WENT: the declaration named
    semantics-blindness and stopped, omitting six bucketed dimensions. Such a check has
    no naturally failing case, so the control below supplies one -- A COMPLETENESS CHECK
    WITH NOTHING MISSING IS UNTESTED BY CONSTRUCTION.
    """

    CODEC = """
code_enum!(Alpha { A1 = "a1", A2 = "a2" });
code_enum!(Beta { B1 = "b1", B2 = "b2" });
code_enum!(MathPrecision { Stable = "st", ReducedMantissa = "rm" });
"""

    def _codec(self, td):
        p = pathlib.Path(td) / "structure_key.rs"
        p.write_text(self.CODEC, encoding="utf-8")
        return str(p)

    def test_an_omitted_bucket_is_caught(self):
        """BORN RED: a derivation deliberately absent from the declaration."""
        with tempfile.TemporaryDirectory() as td:
            body = "declares `a1` only, and says nothing about the other alphabet."
            self.assertEqual(kc.undeclared_buckets(body, self._codec(td)), ["Beta"])

    def test_a_complete_declaration_passes(self):
        """The paired control. Without it, 'always report missing' passes the test above."""
        with tempfile.TemporaryDirectory() as td:
            body = "declares `a1` and `b2` — both alphabets named by a token they collapse onto."
            self.assertEqual(kc.undeclared_buckets(body, self._codec(td)), [])

    def test_a_non_bucket_alphabet_is_not_demanded(self):
        """MathPrecision is a declared attribute, not a bucketing of a continuum.

        Demanding it would make the clause claim a blindness the key does not have —
        the false-disclaimer direction, which is worse than the omission this catches.
        """
        with tempfile.TemporaryDirectory() as td:
            body = "declares `a1` and `b2` and never mentions st/rm."
            self.assertNotIn("MathPrecision", kc.undeclared_buckets(body, self._codec(td)))

    def test_presence_is_by_TOKEN_not_by_type_name(self):
        """Naming the type without a token does not tell the reader WHICH values collide."""
        with tempfile.TemporaryDirectory() as td:
            body = "mentions Alpha and Beta by name, with no token codes at all."
            self.assertEqual(kc.undeclared_buckets(body, self._codec(td)), ["Alpha", "Beta"])

    def test_unreadable_codec_declines_rather_than_passes(self):
        """An unreadable population is not an empty one (#213/#267 discipline)."""
        self.assertIsNone(kc.undeclared_buckets("anything", "/nonexistent/structure_key.rs"))

    def test_the_real_declaration_is_complete(self):
        """Convention 9: the document must actually be in the state the fixtures assume."""
        doc = pathlib.Path(kc.DOC).read_text(encoding="utf-8")
        body = next(b for c, b in kc.clause_bodies(doc) if c == "KISS-CONFORM-6.8-0012")
        self.assertEqual(kc.undeclared_buckets(body), [],
                         "§6.8-0012 omits a bucketed component of the key")


# --- §6.8-0013 completeness: a declared dimension must be bound to an exhibit -----------
#
# The defect (#283): -0013 makes each DECLARED dimension falsifiable and leaves the
# COMPLETENESS of the declaration exactly as unfalsifiable as before. The architect
# demonstrated it by appending a SEVENTH dimension with no exhibit anywhere -- lint CLEAN,
# ratchet CLEAN, test passes, nothing fires. These controls are that demonstration, kept.

DECL_SIX = """\
- **KISS-CONFORM-9.9-0004** - The **example** byte-match compares tokens.
  Its declaration is **`Normalizes:`** the computation, and each bucketed component
  collapses a range onto one token: extent divisibility above its ceiling (`d16`), vector
  width above its widest (`v8`), and element count within a work class
  (`warp`/`block`/`grid`).

  > *Informative.* An aside naming a token pair (`ix32`/`ix64`) that is NOT a declared
  > dimension of this relation.
  *Test:* `test_example`.
"""

# The architect's seventh dimension, appended to the declaring paragraph with no exhibit.
DECL_SEVEN = DECL_SIX.replace(
    "and element count within a work class\n  (`warp`/`block`/`grid`).",
    "element count within a work class (`warp`/`block`/`grid`), and contraction extent\n"
    "  within a size class (`t`/`s`/`m`/`l`).",
)

EXHIBITS_THREE = """\
// EXHIBITS `d16` -- extent divisibility
// EXHIBITS `v8` -- vector width
// EXHIBITS `warp`/`block`/`grid` -- element count within a work class
"""


def unbound_with(clause_body, rs_text):
    with tempfile.TemporaryDirectory() as td:
        p = pathlib.Path(td) / "declared_blindness.rs"
        p.write_text(rs_text, encoding="utf-8")
        return kc.unbound_dimensions(clause_body, str(p))


class DeclarationCompletenessTest(unittest.TestCase):
    def test_seed_applied(self):
        """Vacuity guard, and it earns its place.

        If the seventh dimension never landed in the fixture, the caught-case below would
        pass because there was nothing to catch -- a green from a fixture that tests
        nothing. Assert the seed applied before asserting what it does.
        """
        self.assertNotEqual(DECL_SEVEN, DECL_SIX, "the seventh dimension was not seeded")
        self.assertEqual(len(kc.declared_dimensions(DECL_SIX)), 3)
        self.assertEqual(len(kc.declared_dimensions(DECL_SEVEN)), 4)

    def test_declared_but_unexhibited_dimension_is_caught(self):
        """#283 itself: a dimension appended to the declaration with no exhibit."""
        unbound = unbound_with(DECL_SEVEN, EXHIBITS_THREE)
        self.assertEqual(unbound, [("t", "s", "m", "l")])

    def test_fully_bound_declaration_passes(self):
        """Control. Without it, a check hardcoded to report SOMETHING would pass above."""
        self.assertEqual(unbound_with(DECL_SIX, EXHIBITS_THREE), [])

    def test_unexhibited_record_binds_the_dimension(self):
        """§6.8-0013 allows `unexhibited` -- but only as a record something can READ.

        The record that existed when this was written lived in a `//!` prose paragraph.
        "MUST be recorded, never omitted" is prose nothing reads; this is the half that
        makes it an artifact.
        """
        rs = EXHIBITS_THREE + "//! UNEXHIBITED `t`/`s`/`m`/`l` -- no derivation in this codec\n"
        self.assertEqual(unbound_with(DECL_SEVEN, rs), [])

    def test_informative_aside_is_not_a_declared_dimension(self):
        """Scope control: the parse must not manufacture dimensions from prose.

        `ix32`/`ix64` appears in the informative block of the fixture and is not declared.
        Counting it would demand an exhibit for a blindness the relation does not claim --
        failing closed, but on a fiction, which trains readers to ignore the check.
        """
        self.assertNotIn(("ix32", "ix64"), kc.declared_dimensions(DECL_SIX))

    def test_unreadable_exhibit_file_fails_closed(self):
        """An unreadable artifact must refuse the green, not silently bind nothing."""
        self.assertIsNone(kc.unbound_dimensions(DECL_SIX, "/nonexistent/declared_blindness.rs"))

    def test_live_declaration_is_fully_bound(self):
        """The real tree, not a fixture: every dimension §6.8-0012 declares is bound.

        This started RED -- all six unbound, because no machine-readable marker existed.
        """
        doc = pathlib.Path(kc.DOC).read_text(encoding="utf-8")
        body = next(b for c, b in kc.clause_bodies(doc) if c == "KISS-CONFORM-6.8-0012")
        self.assertEqual(len(kc.declared_dimensions(body)), 6)
        self.assertEqual(kc.unbound_dimensions(body), [])


if __name__ == "__main__":
    unittest.main()
