"""Discrimination controls for the RFC clause-id allocation gate (#342).

`dup_scan` catches an id defined twice in one document. Ids are ALSO allocated in `rfcs/`,
and nothing reconciled the two — so an id drafted in an in-tree RFC and not yet landed was
invisible to the very gate that exists to prevent reuse.

THE GATE LANDS GREEN AGAINST THE LIVE TREE, because no id currently overlaps. That is
precisely the case where a new check most easily joins the population it polices: a gate
written against a tree that cannot violate it has never been shown to fire. So the red is
SEEDED, and it is reported before the green.

Run: python tools/test_kiss_rfc_allocations.py
"""
import pathlib
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import kiss_trace as kt  # noqa: E402

ALLOC = "- **KISS-CONFORM-6.5-0011** — the RFC's clause, drafted and not yet landed.\n"


def rfc_tree(**files):
    d = tempfile.mkdtemp()
    rfcs = pathlib.Path(d) / "rfcs"
    rfcs.mkdir()
    for name, body in files.items():
        (rfcs / (name + ".md")).write_text(body, encoding="utf-8")
    return str(rfcs)


class RfcAllocationTest(unittest.TestCase):
    def test_a_spec_id_colliding_with_an_RFC_allocation_is_caught(self):
        """#332's live case: an ordinal the RFC holds, minted in spec/ for a DIFFERENT
        obligation. Every pre-existing gate passed it — absent from spec/ so append-only was
        satisfied, and it had a test so the trace gate was satisfied."""
        msgs = kt.check_rfc_collisions({"KISS-CONFORM-6.5-0011"}, rfc_tree(draft=ALLOC))
        self.assertEqual(len(msgs), 1, msgs)
        self.assertIn("KISS-CONFORM-6.5-0011", msgs[0])
        self.assertIn("still allocated", msgs[0])

    def test_an_uncollided_spec_id_is_quiet(self):
        """Control. Without it a check that reports everything would pass the case above."""
        self.assertEqual(kt.check_rfc_collisions({"KISS-CONFORM-6.5-0009"},
                                                 rfc_tree(draft=ALLOC)), [])

    def test_a_deliberate_RELEASE_clears_the_allocation(self):
        """A drafted id is released DELIBERATELY, never by absence.

        The same rule as an exclusion list asserted empty rather than deleted: if the RFC's
        clause is the one that landed, that is an edit someone makes on purpose. Deleting
        the RFC block would also clear it — which is why the release is a positive marker
        and not merely the absence of an allocation.
        """
        body = ALLOC + "\n**ALLOCATION LANDED:** KISS-CONFORM-6.5-0011 (merged as-is).\n"
        self.assertEqual(kt.check_rfc_collisions({"KISS-CONFORM-6.5-0011"},
                                                 rfc_tree(draft=body)), [])

    def test_a_release_of_a_DIFFERENT_id_does_not_clear_this_one(self):
        """Paired with the above, or 'release' would degenerate to 'any marker anywhere'."""
        body = ALLOC + "\n**ALLOCATION LANDED:** KISS-CONFORM-6.5-0016.\n"
        self.assertEqual(len(kt.check_rfc_collisions({"KISS-CONFORM-6.5-0011"},
                                                     rfc_tree(draft=body))), 1)

    def test_a_MENTION_in_an_RFC_reserves_nothing(self):
        """An allocation is a drafted clause BLOCK; a bare mention is a CITATION.

        Measured on the live tree: 32 ids are mentioned across `rfcs/` and only 10 are
        allocated. Keying on mentions would reserve every id any RFC discusses — including
        landed clauses it cites — and the gate would red on correct prose.
        """
        body = "This RFC discusses KISS-CONFORM-6.5-0007 at length, citing it repeatedly.\n"
        self.assertEqual(kt.check_rfc_collisions({"KISS-CONFORM-6.5-0007"},
                                                 rfc_tree(draft=body)), [])

    def test_PROSE_containing_the_marker_does_not_release(self):
        """A gate that can be disabled by documentation about the gate (#344 review).

        Un-anchored, a quoted example or an explanatory paragraph carrying the marker
        mid-sentence turns the allocation off. This repo writes a great deal of prose
        about its own gates, so the hole is not theoretical.
        """
        body = (ALLOC + "\nThe marker looks like this: write **ALLOCATION LANDED:** "
                "KISS-CONFORM-6.5-0011 in the RFC when it lands.\n")
        self.assertEqual(len(kt.check_rfc_collisions({"KISS-CONFORM-6.5-0011"},
                                                     rfc_tree(draft=body))), 1,
                         "prose containing the marker released a live allocation")

    def test_the_allocation_parser_IS_the_production_one(self):
        """Not merely equivalent — the same object.

        A parallel copy drifts: the first draft's copy accepted non-4-digit ordinals
        while CLAUSE_ID requires four, so `allocated` and `defined` were different sets
        and a collision between them was invisible in the direction the gate exists for.
        """
        self.assertIs(kt.RE_RFC_ALLOC, kt.RE_DEF)

    def test_a_missing_rfcs_directory_is_not_a_violation(self):
        """A checkout without `rfcs/` reports nothing rather than crashing or blocking."""
        self.assertEqual(kt.check_rfc_collisions({"KISS-CONFORM-6.5-0011"},
                                                 str(pathlib.Path(tempfile.mkdtemp()) / "nope")),
                         [])

    def test_the_live_tree_has_no_collision(self):
        """The real tree, and the reason the red above had to be seeded.

        10 ids are allocated in `rfcs/`; none is defined in `spec/`. If this ever reds, an
        RFC-held ordinal has been minted — read the message, it names the RFC and line.
        """
        root = HERE.parent
        # THE PRODUCTION PARSER (#344 review). The first draft extracted with the gate's
        # own allocation regex, so the control could pass while the gate was broken --
        # it was not exercising the code that ships. `RE_DEF` is what defines a clause
        # everywhere else in this tool, and `RE_RFC_ALLOC` is now an alias of it.
        spec_ids = set()
        for p in (root / "spec").rglob("*.md"):
            spec_ids |= set(kt.RE_DEF.findall(p.read_text(encoding="utf-8")))
        self.assertEqual(kt.check_rfc_collisions(spec_ids, str(root / "rfcs")), [])
        allocs, _released = kt.rfc_allocations(str(root / "rfcs"))
        self.assertTrue(allocs, "no RFC allocations found — the parse has gone blind")


if __name__ == "__main__":
    unittest.main()
