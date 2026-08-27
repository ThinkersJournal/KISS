"""Controls for the pinned-tolerated-red assertion (#343).

`strict` is `continue-on-error` BY DESIGN and should stay that way — a gate that can never
go green, including for the commits that would fix it, gets bypassed by habit. The defect is
one layer up: `continue-on-error` makes the run's conclusion SUCCESS while it carries a red
job, so **"main is green" and "strict is failing on main" are simultaneously true**, and a
tolerated failure is indistinguishable from a new one at every surface above the log.

    A PERMANENTLY-TOLERATED RED IS THE SAME OBJECT AS A DELETED EXCLUSION LIST UNLESS
    SOMETHING PINS WHAT IT IS TOLERATING.

`--assert-known-red` pins it, and it fails in BOTH directions: a different red is news, and
so is no red at all — the tolerance going stale should be removed in the change that earned
it, exactly as a stale coverage floor is lowered in the PR that earns the improvement.

Run: python tools/test_kiss_known_red.py
"""
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
TOOL = str(HERE / "kiss_trace.py")

CID, TESTNAME = "KISS-OPS-6.0-0042", "test_ops_fixture_backed"
STEMS = ["umbrella", "announce", "classify", "ops", "grammar", "contract",
         "synth", "consume", "emit", "conform"]


def run(*args):
    r = subprocess.run([sys.executable, TOOL, *args], capture_output=True, timeout=300,
                       env={**os.environ, "PYTHONIOENCODING": "utf-8"})
    out = r.stdout.decode("utf-8", "replace")
    err = r.stderr.decode("utf-8", "replace")
    # STDERR IS KEPT (#347 review). A harness that discards it asserts on a strictly
    # smaller artefact than the one CI produces -- and the failures it will be used to
    # diagnose are exactly the ones where a Python traceback IS the message.
    return r.returncode, out + (("\n--- stderr ---\n" + err) if err.strip() else "")


def all_backed_tree():
    """A fixture suite where every clause HAS a test — so `strict` passes."""
    root = pathlib.Path(tempfile.mkdtemp())
    spec, conf = root / "spec", root / "conformance"
    spec.mkdir()
    conf.mkdir()
    for stem in STEMS:
        body = "## 9. Traceability\n\n| Clause | Test |\n|---|---|\n"
        if stem == "ops":
            body = ("## 6.0 Fixture\n\n- **%s** — A fixture clause. An implementation MUST do "
                    "the fixture thing. *Test:* `%s`.\n\n## 9. Traceability\n\n"
                    "| Clause | Test |\n|---|---|\n| %s | `%s` |\n"
                    % (CID, TESTNAME, CID, TESTNAME))
        (spec / (stem + ".md")).write_text(body, encoding="utf-8")
    (conf / "fixture_tests.rs").write_text(
        "#[test]\nfn %s() { assert_eq!(1 + 1, 2); }\n" % TESTNAME, encoding="utf-8")
    (conf / "UNBACKED.tsv").write_text("# fixture ledger\n", encoding="utf-8")
    return str(spec), str(conf)


class KnownRedTest(unittest.TestCase):
    def test_the_tolerated_red_exits_zero(self):
        """The live tree: `strict` fails for the documented reason and no other."""
        rc, out = run("--assert-known-red")
        self.assertEqual(rc, 0, out[-500:])
        self.assertIn("FAILURE REASONS: strict_untested", out)
        self.assertIn("KNOWN RED", out)

    def test_a_DIFFERENT_red_exits_one(self):
        """The whole point. A clause naming a test that does not exist is a `doc` failure —
        which today looks identical to the tolerated state at the run level."""
        spec = HERE.parent / "spec" / "conform.md"
        orig = spec.read_text(encoding="utf-8")
        anchor = "- **KISS-CONFORM-6.5-0010**"
        self.assertIn(anchor, orig, "seed anchor missing — refusing to report a result")
        seeded = orig.replace(anchor,
                              "- **KISS-CONFORM-6.5-0099** — a SEEDED clause naming a test "
                              "that does not exist.\n"
                              "  *Test:* `test_conform_no_such_test_anywhere`.\n" + anchor, 1)
        self.assertNotEqual(seeded, orig, "seed did not apply")
        try:
            spec.write_text(seeded, encoding="utf-8")
            rc, out = run("--assert-known-red")
        finally:
            spec.write_text(orig, encoding="utf-8")
            self.assertEqual(spec.read_text(encoding="utf-8"), orig, "spec restore FAILED")
        self.assertEqual(rc, 1, "a NEW failure was accepted as the tolerated one:\n" + out[-500:])
        self.assertIn("A DIFFERENT RED", out)

    def test_NO_red_also_exits_one(self):
        """Fails in BOTH directions, and this arm is the one people forget.

        If `strict` starts passing, the `continue-on-error` tolerance is STALE and should be
        removed in the change that earned it — the same rule as a coverage floor lowered in
        the PR that earns the improvement. Green is not the expected state here, and a check
        that accepted it would let the tolerance outlive its reason indefinitely.
        """
        spec, conf = all_backed_tree()
        try:
            rc, out = run("--assert-known-red", "--spec-dir", spec, "--conformance-dir", conf)
            self.assertEqual(rc, 1, "a PASSING strict was accepted as the tolerated red:\n" + out[-400:])
            self.assertIn("TOLERANCE IS STALE", out)
        finally:
            # cleaned even when an assertion raises (#347 review) -- a failing run is
            # exactly when the tree is most likely to be left behind, and least likely
            # to be noticed.
            shutil.rmtree(pathlib.Path(spec).parent, ignore_errors=True)

    def test_every_failure_site_names_itself(self):
        """The assertion is only as good as its coverage of the failure sites.

        An untagged `any_fail = True` is INVISIBLE to the reason set — so a new failure class
        added later would be silently absorbed into the tolerated red, which is precisely the
        defect this exists to close, reintroduced one edit at a time.
        """
        src = (HERE / "kiss_trace.py").read_text(encoding="utf-8")
        code = "\n".join(l for l in src.splitlines() if not l.lstrip().startswith("#"))
        self.assertEqual(
            re.findall(r"any_fail\s*=\s*True", code), [],
            "an untagged failure site is back — it cannot appear in the reason set, so "
            "`--assert-known-red` would absorb it into the tolerated state. Use "
            "`note_fail(\"<tag>\")`.")
        self.assertGreaterEqual(len(re.findall(r"note_fail\(", code)), 14,
                                "failure sites went missing — the reason set has gone blind")

    def test_an_INCONCLUSIVE_is_reported_not_silent(self):
        """A DECLINE is a third state, not the absence of a reason (#347 review).

        Without this, `--why-red` printed `<none>` while the tool exited 2 because the
        ratchet DECLINED to answer — the instrument built to tell one red from another
        having a state in which it explains nothing. That is the decline-vs-failure
        collapse this PR exists to close, in the reporting half of the same PR.
        """
        rc, out = run("--why-red", "--ratchet")
        self.assertEqual(rc, 0, out[-300:])
        # ASSERT THE REASON-SET LINE ITSELF, not merely that the word appears (#347).
        # The first version checked `assertIn("inconclusive", out)` and the explanatory
        # line -- both of which survive dropping the decline from the set: the word is
        # already in the pre-existing `RESULT: INCONCLUSIVE` text, and the explanation
        # prints off `inconclusive` directly. Mutation-proven vacuous, then fixed.
        reasons = next(l for l in out.splitlines() if "FAILURE REASONS:" in l)
        self.assertIn("inconclusive", reasons,
                      "a DECLINE is missing from the reason set: " + reasons)
        self.assertIn("DECLINE, not a failure", out)

    def test_why_red_reports_without_gating(self):
        """The diagnostic arm exits 0 even on the live (red) tree, so it can be read anywhere."""
        rc, out = run("--why-red")
        self.assertEqual(rc, 0, out[-300:])
        self.assertIn("FAILURE REASONS", out)


if __name__ == "__main__":
    unittest.main()
