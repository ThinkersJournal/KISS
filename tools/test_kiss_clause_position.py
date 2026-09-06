#!/usr/bin/env python3
"""Born-red for kiss_clause_position — the position check MUST flag a clause moved out of its
section, and MUST stay clean with the clause in place.

⚠️ An advisory that has never fired is indistinguishable from one that CANNOT fire. #486 exists
because six gates passed on a clause placed past the appendices; this test proves the new check
reddens on exactly that defect (a real clause moved past a non-numbered heading) and not otherwise.

unittest.TestCase (not bare `assert`) so Bandit B101 has nothing to flag — the repo's dominant
test style and the #468 lesson applied up front.
"""
import sys
import os
import io
import tempfile
import pathlib
import contextlib
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kiss_clause_position import scan, main as position_main  # noqa: E402

REAL_CLAUSE = "KISS-CONTRACT-6.6-0001"  # a real §6.6 clause, in place on main


def _spec(name):
    return pathlib.Path(__file__).resolve().parent.parent / "spec" / name


class ClausePositionTests(unittest.TestCase):
    def test_real_clause_is_clean_in_place(self):
        """Positive control: the real clause is correctly placed on main, and the parser sees the
        whole population (a low count would mean the bullet form drifted and the check went blind)."""
        offenders, total = scan(_spec("contract.md"))
        self.assertGreater(total, 100, f"parsed only {total} clause defs — the bullet form changed")
        self.assertNotIn(
            REAL_CLAUSE, [cid for _, cid, _, _ in offenders],
            f"{REAL_CLAUSE} is correctly placed on main; the control must find it clean",
        )

    def test_position_check_reddens_on_a_clause_past_its_section(self):
        """Born-red: move a real clause past a non-numbered heading (the #486 defect) and require
        the check to flag it, naming current-section None."""
        moved = (
            _spec("contract.md").read_text(encoding="utf-8")
            + f"\n\n## Appendix Z (synthetic)\n\n- **{REAL_CLAUSE}** — moved out of its section\n"
        )
        f = tempfile.NamedTemporaryFile("w", suffix=".md", delete=False, encoding="utf-8")
        f.write(moved)
        f.close()
        try:
            offenders, _ = scan(pathlib.Path(f.name))
            flagged = [cur for _, cid, _, cur in offenders if cid == REAL_CLAUSE]
            self.assertTrue(
                flagged,
                "the position check did NOT flag a clause moved past the appendices — it cannot fire",
            )
            self.assertIsNone(
                flagged[-1],
                f"a clause past a non-numbered heading must report current-section None, got {flagged[-1]!r}",
            )
        finally:
            os.unlink(f.name)

    def test_main_is_clean_on_the_live_spec(self):
        """The lint exits 0 on the current tree (the whole spec/ is correctly positioned)."""
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc = position_main()
        self.assertEqual(rc, 0, buf.getvalue())
        self.assertIn("RESULT: CLEAN", buf.getvalue())


if __name__ == "__main__":
    unittest.main()
