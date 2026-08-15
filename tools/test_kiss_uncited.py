"""Gates for the uncited-test sweep's DECLARATION SCOPE.

The sweep decides which uncited tests are citation CANDIDATES. Its scope rule is
the whole tool: too narrow and real declarations are invisible, too wide and the
strongest bucket fills with rows no author ever declared. It had no test at all
until the first-test-in-file defect was found by reading a candidate that had
fourteen refs and one comment.

Every test here is a DISCRIMINATION control: it fails on the pre-fix behaviour
and passes on the fixed one. A scope test that passes under both scopes is
measuring the file, not the rule.
"""
import pathlib
import sys
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import kiss_uncited as ku  # noqa: E402

# A file whose FIRST test declares nothing in its own comment, sitting under an
# implementation body that names clauses. Pre-fix, the first test's declaration
# scope was src[0:start] and it inherited every one of them.
FIRST_TEST_FIXTURE = """\
//! Module doc comment for the widget helpers (KISS-OPS §6.99-0001).

/// Helper that implements the reduce path described by §6.99-0002.
fn helper() -> u32 {
    // the clamp policy of §6.99-0003 applies here
    7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_test_declares_nothing_itself() {
        assert_eq!(helper(), 7);
    }

    // a real header declaration for the NEXT test (§6.99-0004)
    #[test]
    fn second_test_has_a_header_declaration() {
        assert_eq!(helper(), 7);
    }
}
"""


class DeclarationScopeTest(unittest.TestCase):
    def _sweep(self, source):
        """Run the sweep over a throwaway conformance dir holding one .rs file."""
        with tempfile.TemporaryDirectory() as td:
            conf = pathlib.Path(td) / "conformance"
            conf.mkdir()
            (conf / "sample.rs").write_text(source, encoding="utf-8")
            spec = pathlib.Path(td) / "spec"
            spec.mkdir()
            rows = ku.sweep(str(spec), str(conf))
        return {r["test"]: r for r in rows}

    def test_first_test_in_file_does_not_inherit_the_file_above_it(self):
        """The first test has no previous test to bound a header's reach.

        Pre-fix this row carried §6.99-0001/-0002/-0003 — the module doc comment,
        a helper's doc comment, and a comment inside a function body. None were
        written about this test. This assertion FAILS on the pre-fix scope, which
        is the only reason it is worth having.
        """
        rows = self._sweep(FIRST_TEST_FIXTURE)
        first = rows["first_test_declares_nothing_itself"]
        self.assertEqual(
            first["refs"], [],
            "the first test in a file must not inherit clause refs from the "
            f"implementation body above it; got {first['refs']}",
        )
        self.assertEqual(
            first["bucket"], "unclear",
            "with no declaration of its own the first test is a row for a human, "
            "not a citation candidate",
        )

    def test_header_declaration_still_reaches_the_next_test(self):
        """The narrow fallback must not cost the behaviour the tool exists for.

        A header separated by a blank line is invisible to a contiguous run, and
        recovering those rows is why the declaration scope is wider than the
        citation scope. Narrowing the FIRST test must leave that intact — this is
        the control against 'fixing' the defect by disabling the feature.
        """
        rows = self._sweep(FIRST_TEST_FIXTURE)
        second = rows["second_test_has_a_header_declaration"]
        self.assertEqual(second["refs"], ["§6.99-0004"])
        self.assertEqual(second["bucket"], "declared")

    def test_a_first_test_with_its_own_comment_still_declares(self):
        """The fallback is narrow, not empty.

        A first test that names a clause in its OWN contiguous comment run keeps
        its declaration. Without this, the fix would be indistinguishable from
        dropping first-in-file rows entirely.
        """
        src = FIRST_TEST_FIXTURE.replace(
            "    #[test]\n    fn first_test_declares_nothing_itself",
            "    // this test asserts §6.99-0007\n    #[test]\n"
            "    fn first_test_declares_nothing_itself",
            1,
        )
        rows = self._sweep(src)
        self.assertEqual(rows["first_test_declares_nothing_itself"]["refs"], ["§6.99-0007"])

    def test_fixture_actually_exercises_the_first_test_path(self):
        """Convention 9: assert the setup this file depends on is real.

        If the fixture's first `#[test]` were not preceded by clause-naming
        implementation text, every assertion above would pass vacuously on any
        scope rule.
        """
        head = FIRST_TEST_FIXTURE.split("#[test]", 1)[0]
        for ref in ("§6.99-0001", "§6.99-0002", "§6.99-0003"):
            self.assertIn(ref, head, "fixture must name clauses ABOVE the first test")


if __name__ == "__main__":
    unittest.main()
