# SPDX-License-Identifier: MIT OR Apache-2.0
"""Discrimination controls for the freeze gate (#524).

A gate that lands green against a tree that cannot violate it has never been shown to fire (the same
shape as #342's allocation gate). This tree is entirely Draft, so the green is vacuous: every
promotion case below is SEEDED, and the real `--freeze-ready` is run once on real data to show the
BLOCKED arm is not an artifact of the injected callable.

Run: python tools/test_kiss_freeze_gate.py
"""
import pathlib
import sys
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import kiss_freeze_gate as g  # noqa: E402

ROW = "| Maturity stage | **{}** (note) |\n"


class StageParseTest(unittest.TestCase):
    def test_reads_the_bold_stage_token(self):
        self.assertEqual(g.maturity_stage("x\n" + ROW.format("Draft") + "y"), "Draft")
        self.assertEqual(g.maturity_stage(ROW.format("Frozen")), "Frozen")

    def test_a_missing_row_is_None_not_Draft(self):
        """Control: assuming Draft for an unreadable row would let a promotion through silently."""
        self.assertIsNone(g.maturity_stage("| Title | KISS-Classify |\n"))
        self.assertIsNone(g.maturity_stage("| Maturity stage | not bold |\n"))

    def test_the_real_specs_all_have_a_readable_stage(self):
        for s in g.SUBS:
            st = g.maturity_stage(pathlib.Path(g.SPEC_DIR, f"{s}.md").read_text(encoding="utf-8"))
            self.assertIsNotNone(st, f"spec/{s}.md has no readable `Maturity stage` row")


class DecisionTest(unittest.TestCase):
    def stages(self, **over):
        d = {s: "Draft" for s in g.SUBS}
        d.update(over)
        return d

    def test_all_draft_is_clean_and_never_calls_freeze_ready(self):
        called = []
        code, _ = g.evaluate(self.stages(), lambda s: called.append(s) or 1)
        self.assertEqual(code, 0)
        self.assertEqual(called, [], "a Draft tree must not even run the check")

    def test_a_promotion_that_fails_freeze_ready_is_BLOCKED(self):
        """THE SEEDED RED: Classify flips to Frozen while --freeze-ready exits 1."""
        code, lines = g.evaluate(self.stages(classify="Frozen"), lambda s: 1)
        self.assertEqual(code, 1)
        self.assertIn("classify", "\n".join(lines))
        self.assertIn("BLOCKED", "\n".join(lines))

    def test_a_promotion_that_passes_is_clean(self):
        """Control: the gate is not 'any Frozen is red'."""
        code, _ = g.evaluate(self.stages(classify="Frozen"), lambda s: 0)
        self.assertEqual(code, 0)

    def test_only_the_failing_frozen_sub_is_named(self):
        code, lines = g.evaluate(self.stages(ops="Frozen", emit="Frozen"), lambda s: 0 if s == "ops" else 1)
        self.assertEqual(code, 1)
        blocked = [l for l in lines if l.startswith("BLOCKED")][0]
        self.assertIn("emit", blocked)
        self.assertNotIn("ops", blocked)

    def test_an_unreadable_stage_DECLINES_with_its_own_code(self):
        code, lines = g.evaluate(self.stages(synth=None), lambda s: 0)
        self.assertEqual(code, 2, "INCONCLUSIVE is not a pass and not a violation")
        self.assertIn("synth", lines[0])


class RealDataTest(unittest.TestCase):
    def test_the_blocked_arm_fires_on_the_real_traceability_data(self):
        """The injected callable proves the logic; this proves `--freeze-ready` really exits non-zero
        for a sub-standard on today's tree (0 of 9 satisfy condition 3), so a real promotion is red."""
        rc = g._freeze_ready("classify")
        self.assertNotEqual(rc, 0, "if this passes, --freeze-ready no longer gates and so neither does this")
        code, _ = g.evaluate({s: ("Frozen" if s == "classify" else "Draft") for s in g.SUBS}, g._freeze_ready)
        self.assertEqual(code, 1)

    def test_the_cli_is_clean_on_the_draft_tree(self):
        self.assertEqual(g.main([]), 0)


if __name__ == "__main__":
    unittest.main()
