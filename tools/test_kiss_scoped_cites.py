"""Discrimination controls for the 16(e) bare-cite detector.

Every case must FAIL on the defect and PASS without it. The controls that matter most here
are the ones proving the detector stays QUIET: a lint that flags correct prose gets routed
around, and this one has two ways to do that — flagging a self-reference, and flagging a
bare cite deliberately preserved inside a quotation (#310).

Run: python tools/test_kiss_scoped_cites.py
"""
import pathlib
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import kiss_scoped_cites as sc  # noqa: E402

# A fixture document that DEFINES §6.4 and §3.1, so a cite to either is a self-reference.
DEFINER = "## 6. Spec\n\n### 6.4 Things\n\n### 3.1 Terms\n\n"


def scan_docs(**docs):
    """Run the real scanner over a fixture tree; returns (violations, judgement)."""
    with tempfile.TemporaryDirectory() as td:
        root = pathlib.Path(td)
        (root / "spec").mkdir()
        real_docs = sc.DOCS
        try:
            sc.DOCS = []
            for name, body in docs.items():
                rel = "spec/%s.md" % name
                (root / "spec" / (name + ".md")).write_text(body, encoding="utf-8")
                sc.DOCS = sc.DOCS + [rel]
            return sc.scan(str(root))
        finally:
            sc.DOCS = real_docs


class BareCiteTest(unittest.TestCase):
    def test_a_bare_cross_document_cite_is_a_violation(self):
        """The whole point. `conform.md` citing a section it does not define, unqualified."""
        v, _j = scan_docs(umbrella=DEFINER, conform="See §6.4 for the rule.\n")
        self.assertEqual([c for _d, _l, c in v], ["§6.4"])

    def test_a_self_reference_is_NOT_a_violation(self):
        """Control. Without it a detector that flags every § would pass the case above.

        A bare `§6.4` inside the document defining §6.4 is CORRECT — that is the whole
        reason the rule is scoped by defining document rather than being 'always qualify'.
        """
        v, _j = scan_docs(conform=DEFINER + "See §6.4 for the rule.\n")
        self.assertEqual(v, [])

    def test_every_qualified_form_passes(self):
        """The five binding forms, three of which were found by measuring the corpus.

        A rule written to the two forms that were in view when this tool was specified
        would have flagged 138 correct cites.
        """
        forms = {
            "adjacency": "KISS-Ops §6.4 governs.",
            "apposition": "owned by KISS-Ops (§6.4), spelled verbatim.",
            "markdown emphasis": "frozen at KISS-OPS **§6.4** for this version.",
            "possessive": "KISS-Emit's §6.4 correspondence table.",
            "line wrap": "pinned by KISS-Ops\n  §6.4 in the registry.",
        }
        for name, body in forms.items():
            with self.subTest(form=name):
                v, j = scan_docs(umbrella=DEFINER, conform=body + "\n")
                self.assertEqual(v, [], "%s should be QUALIFIED, got a violation" % name)
                self.assertEqual(j, [], "%s should be QUALIFIED, got a judgement row" % name)

    def test_coordination_is_qualified_but_only_within_the_delimited_group(self):
        """One qualifier distributed over a list resolves for every member (ruled).

        The paired half is the point: a coordinator CANNOT reach across a sentence
        boundary. An over-broad coordination model is worse than a narrow one, because
        over-qualifying is silent — it goes quiet on the ambiguities the lint exists to
        find, while a miss lands on the judgement list where a human sees it.
        """
        v, _j = scan_docs(umbrella=DEFINER,
                          conform="pinned at KISS-Ops §6.4 and §3.1 for this version.\n")
        self.assertEqual(v, [], "a coordinated list member should be qualified")

        v2, _j2 = scan_docs(umbrella=DEFINER,
                            conform="pinned at KISS-Ops §6.4. Separately, §3.1 applies.\n")
        self.assertEqual([c for _d, _l, c in v2], ["§3.1"],
                         "a coordinator must not reach across a full stop")

    def test_a_bare_cite_inside_a_quotation_is_EXEMPT(self):
        """#310 requires a paraphrase to preserve the other party's identifiers.

        Rewriting a cite inside quotation marks trades an unresolvable reference for an
        UNRELIABLE QUOTATION, which is worse. A lint that flags the correct fix punishes
        the correct behaviour and teaches everyone to ignore it. Measured at 7 instances
        in the corpus — not a corner case.
        """
        v, _j = scan_docs(umbrella=DEFINER,
                          conform='> quoting them: "its §6.4 reference vectors"\n')
        self.assertEqual(v, [], "a bare cite inside a blockquote must not flag")

    def test_336s_actual_note_passes(self):
        """The real artefact, not a fixture (demonstrated, not asserted).

        #336's fix DELIBERATELY preserves a bare `§6.7` inside a quotation and resolves it
        alongside. If this detector flagged that note it would be punishing the fix that
        motivated it.
        """
        cuda = HERE.parent / "spec" / "namespaces" / "cuda.md"
        text = cuda.read_text(encoding="utf-8")
        self.assertIn("its §6.7 reference vectors", text,
                      "the note this control is about is not in cuda.md — re-point it")
        v, _j = sc.scan()
        self.assertEqual([r for r in v if "cuda" in r[0]], [],
                         "#336's quoted bare §6.7 was flagged — the exemption is broken")

    def test_the_judgement_list_never_gates(self):
        """A case the syntactic model cannot decide is REPORTED, never failed.

        Otherwise the tool forces a human to satisfy a rule it cannot state.
        """
        v, j = scan_docs(umbrella=DEFINER,
                         conform="KISS-Ops owns the registry; the table in §6.4 lists it.\n")
        self.assertEqual(v, [], "a name-nearby case must not gate")
        self.assertTrue(j, "...but it must still be REPORTED")

    def test_the_live_corpus_has_no_violations(self):
        """The real tree. This lands born-red and is the assertion that turns it green."""
        v, _j = sc.scan()
        self.assertEqual(v, [], "bare cross-document cites remain: %r" % (v[:6],))


if __name__ == "__main__":
    unittest.main()
