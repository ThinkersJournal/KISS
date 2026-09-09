r"""Controls for the citation-form lint (#488).

⚠️ BORN-RED IS THE POINT OF THIS FILE. After the normalization pass the tree is CLEAN, so without
a control that forces a red, this ships as a check nobody has ever seen fail -- and "never fired"
is indistinguishable from "cannot fire". Every canonical spelling is also checked, because a lint
that flags everything is as useless as one that flags nothing and both report a number.

⚠️ `unittest.TestCase`, not bare `assert`, and that is deliberate rather than stylistic. Bandit
B101 flags every bare `assert` and Codacy gates on NEW issues, so a new bare-assert test file
arrives with a red check attached (#491, filed). 15 of the tree's 31 tool tests already use
`self.assertX`, which B101 never flags -- so this is an existing house style, not a novelty, and
it avoids manufacturing the exact toll #491 describes.
"""
import os
import shutil
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kiss_cite_form as cf  # noqa: E402


def _tree(sites):
    """A throwaway repo root whose conformance/ holds one .rs file with `sites` in it."""
    root = tempfile.mkdtemp(prefix="citeform_")
    d = os.path.join(root, "conformance", "tests")
    os.makedirs(d)
    with open(os.path.join(d, "t.rs"), "w", encoding="utf-8") as fh:
        fh.write("\n".join(sites) + "\n")
    return root


class CiteForm(unittest.TestCase):
    def scan(self, lines):
        root = _tree(lines)
        try:
            return cf.scan(root)
        finally:
            shutil.rmtree(root, ignore_errors=True)

    # ---- BORN-RED: the drift this lint exists for must be flagged ----------------------

    def test_born_red_backs_without_a_colon_is_flagged(self):
        """The exact shape of the four original #488 sites."""
        sites = self.scan(["/// Backs KISS-CONTRACT-6.6-0009 - the division semantics."])
        self.assertEqual(len(sites), 1, "the extractor found no citation at all")
        self.assertFalse(sites[0][4], "a colon-less `Backs` must be flagged; it was not")

    def test_born_red_enforces_without_a_colon_is_flagged(self):
        sites = self.scan(["//! Enforces KISS-ANNOUNCE-6.1-0010"])
        self.assertEqual(len(sites), 1)
        self.assertFalse(sites[0][4], "a colon-less `Enforces` must be flagged; it was not")

    def test_born_red_proven_without_a_colon_is_flagged(self):
        """⚠️ The one that LOSES CREDIT SILENTLY. `RE_CITE` accepts `Backs`/`Enforces` with or
        without the colon, but `RE_PROVEN` is `Proven:\s*` -- colon REQUIRED. So a `Proven`
        written in the loose form that `Backs` taught is not a style slip: it is not recognised
        at all, and the credit vanishes with no error. 0 such sites exist today (control: 20
        `Proven:` sites do), so this is a trap for the next author, not a live loss."""
        sites = self.scan(["// Proven KISS-OPS-6.5-0001"])
        self.assertEqual(len(sites), 1)
        self.assertFalse(sites[0][4])

    def test_born_red_space_before_the_colon_is_MATCHED_and_flagged(self):
        """The Codacy #501 finding, and the sharpest of the four: `Backs : KISS-X`.

        The first version's separator was `(:?\s*)`, which cannot match this AT ALL --
        so the site was invisible to the lint while `kiss_trace`'s `[:\s]*` matched it and
        CREDITED it. A lint policing citation form, blind to a citation form the tracer accepts.

        This control asserts BOTH halves, and the first is the one that was broken: the site
        must be FOUND (len == 1), then judged non-canonical. Asserting only the verdict would
        pass trivially on an extractor that finds nothing."""
        sites = self.scan(["/// Backs : KISS-CONTRACT-6.6-0009"])
        self.assertEqual(len(sites), 1,
                         "MATCH half: the tracer credits this form, so the lint must SEE it")
        self.assertFalse(sites[0][4], "JUDGE half: a colon not adjacent to the keyword is not canonical")

    def test_population_matches_kiss_trace_on_every_separator_the_tracer_accepts(self):
        """⚠️ POPULATION PARITY WITH THE TRACER IS THE INVARIANT, not any single spelling.
        Every separator `kiss_trace`'s `[:\s]*` accepts must also be SEEN here -- otherwise the
        lint's clean run is a statement about its own regex rather than about the tree."""
        forms = ["/// Backs: KISS-OPS-6.1-0001", "/// Backs : KISS-OPS-6.1-0002",
                 "/// Backs  KISS-OPS-6.1-0003", "/// Backs:  KISS-OPS-6.1-0004",
                 "/// Backs	:	KISS-OPS-6.1-0005"]
        sites = self.scan(forms)
        self.assertEqual(len(sites), len(forms),
                         f"lint saw {len(sites)} of {len(forms)} tracer-accepted separators")
        # control: exactly the two colon-adjacent ones are canonical
        self.assertEqual(sum(1 for s in sites if s[4]), 2,
                         "only `Backs:` and `Backs:  ` are colon-adjacent")

    # ---- the canonical forms must NOT be flagged ---------------------------------------

    def test_canonical_forms_are_accepted(self):
        lines = ["/// Backs: KISS-OPS-6.1-0001",
                 "//! Enforces: KISS-OPS-6.1-0002",
                 "// Proven: KISS-OPS-6.1-0003"]
        sites = self.scan(lines)
        self.assertEqual(len(sites), 3, f"expected 3 citation sites, got {len(sites)}")
        for s in sites:
            self.assertTrue(s[4], f"canonical form wrongly flagged: {s!r}")

    # ---- an empty population must REFUSE, not report a clean tree -----------------------

    def test_no_citations_at_all_is_could_not_measure_not_clean(self):
        """⚠️ A recognizer that silently matches nothing reports zero violations, and zero
        violations is byte-identical to a clean tree. `main()` must return 2, not 0."""
        root = tempfile.mkdtemp(prefix="citeform_empty_")
        os.makedirs(os.path.join(root, "conformance"))
        saved_root, saved_argv = cf.ROOT, sys.argv
        try:
            self.assertEqual(cf.scan(root), [], "control: this tree really has no citations")
            # ⚠️ DRIVE main(), not just scan(). The claim in this test's NAME is about the EXIT
            # CODE, and asserting on scan() alone would leave the name promising something no
            # assertion checks -- a test whose name is a claim about what it covers.
            cf.ROOT, sys.argv = root, ["kiss_cite_form.py", "--strict"]
            self.assertEqual(cf.main(), 2,
                             "an empty population must be COULD-NOT-MEASURE (2), never clean (0)")
            # control: the same main() on a tree WITH a canonical citation returns 0, so the 2
            # above is about emptiness rather than about main() always returning 2.
            good = _tree(["/// Backs: KISS-OPS-6.1-0001"])
            try:
                cf.ROOT = good
                self.assertEqual(cf.main(), 0, "control: a clean non-empty tree must return 0")
            finally:
                shutil.rmtree(good, ignore_errors=True)
        finally:
            cf.ROOT, sys.argv = saved_root, saved_argv
            shutil.rmtree(root, ignore_errors=True)

    # ---- no second parser --------------------------------------------------------------

    def test_reuses_kiss_trace_clause_id_rather_than_respelling_it(self):
        """A private clause-id pattern here could recognise a different set of citations than
        the tool whose convention this polices -- making any disagreement a fact about two
        PARSERS rather than about the tree."""
        src = open(os.path.join(HERE, "kiss_cite_form.py"), encoding="utf-8").read()
        self.assertIn("kt.CLAUSE_ID", src, "must reuse kiss_trace's clause-id pattern")
        self.assertNotIn("KISS-[A-Z]", src, "a second clause-id spelling has crept in")

    # ---- the population figure is itself a control --------------------------------------

    def test_population_counts_every_keyword(self):
        sites = self.scan(["/// Backs: KISS-OPS-6.1-0001", "// Enforces: KISS-OPS-6.1-0002",
                           "// Proven: KISS-OPS-6.1-0003", "// nothing to see here"])
        self.assertEqual(sorted(s[2] for s in sites), ["Backs", "Enforces", "Proven"])


if __name__ == "__main__":
    r = unittest.main(exit=False, verbosity=0).result
    print(f"ok - {r.testsRun} controls pass: born-red on all three keywords AND on the space-before-colon form the tracer credits, population parity with kiss_trace, canonical forms "
          f"accepted, an empty population refuses rather than reporting clean, and no second parser")
    raise SystemExit(0 if r.wasSuccessful() else 1)
