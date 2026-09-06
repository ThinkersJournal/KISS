"""Discrimination controls for the `Backs:` ↔ §9 matrix cross-check (`kiss_backs_matrix.py`, #479).

Every positive assertion is paired with the case it must NOT flag, because a checker that flags
everything and a checker that flags nothing both produce a number somebody will act on.

The three that matter:

  * ⚠️ BORN-RED AGAINST THE REAL HISTORICAL DEFECT. #475 shipped a test whose docstring cited
    `KISS-CONTRACT-6.6-0006` while the clause it was written for was `-0009`. Both were already
    backed, so the ledger was BYTE-IDENTICAL either way and `kiss_cites` produced identical output
    at exit 0. That exact shape is reproduced here and must be flagged.
  * THE THREE LEGITIMATE SHAPES MUST STAY SILENT — agreeing citation, forward-only (no citation),
    reverse-only (no matrix row). A tool that flagged any of them would flag the convention.
  * ⚠️ AN EMPTY POPULATION IS A REFUSAL (exit 2), NOT A CLEAN ZERO. If no test is both
    matrix-bound and citing, the comparator has nothing to range over and "0 candidates" would
    report success having checked nothing.

Run: python tools/test_kiss_backs_matrix.py   (also collected by pytest)
"""
import os
import sys

if not __debug__:
    raise SystemExit(
        "refusing to run under -O/PYTHONOPTIMIZE: `assert` is stripped, so these "
        "controls would report success having verified nothing")

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_backs_matrix as bm  # noqa: E402


# ------------------------------------------------------- born-red: the real defect --

def test_the_475_miscitation_shape_is_flagged():
    """⚠️ The exact shape that shipped: a test bound to -0009 citing -0006, both already backed.

    Byte-identical ledger, identical ratchet verdict, `kiss_cites` exit 0 — and this flags it.
    """
    tests = {"t": {"clauses": {"KISS-CONTRACT-6.6-0006"}, "file": "x.rs"}}
    matrix = {"t": {"KISS-CONTRACT-6.6-0009"}}
    got = bm.disagreements(tests, matrix)
    assert len(got) == 1, got
    name, cited, bound = got[0]
    assert name == "t"
    assert cited == ["KISS-CONTRACT-6.6-0006"]
    assert bound == ["KISS-CONTRACT-6.6-0009"]


# ------------------------------------------- the legitimate shapes must stay silent --

def test_an_agreeing_citation_is_not_flagged():
    tests = {"t": {"clauses": {"KISS-A-1.1-0001"}}}
    matrix = {"t": {"KISS-A-1.1-0001"}}
    assert bm.disagreements(tests, matrix) == []


def test_a_citation_that_is_a_SUPERSET_of_the_matrix_is_not_flagged():
    """A test may back several clauses; naming its matrix clause among them is agreement."""
    tests = {"t": {"clauses": {"KISS-A-1.1-0001", "KISS-B-2.2-0002"}}}
    matrix = {"t": {"KISS-A-1.1-0001"}}
    assert bm.disagreements(tests, matrix) == []


def test_forward_only_is_out_of_scope():
    """Matrix-bound, cites nothing: credit comes from the NAME. Nothing to disagree with."""
    tests = {"t": {"clauses": set()}}
    matrix = {"t": {"KISS-A-1.1-0001"}}
    assert bm.disagreements(tests, matrix) == []


def test_reverse_only_is_out_of_scope():
    """Cites a clause, no §9 row names this test: `kiss_cites` owns that direction."""
    tests = {"t": {"clauses": {"KISS-A-1.1-0001"}}}
    assert bm.disagreements(tests, {}) == []


# --------------------------------------------- the real tree is scanned, not stubbed --

def test_the_real_scan_finds_a_nontrivial_population():
    """⚠️ THE SCAN SIZE IS THE CONTROL. A recognizer that sees almost nothing produces a clean
    zero indistinguishable from a clean tree — the shape that turned "55 of 111 ledger rows name
    missing tests" into "0 of 111" once the test recognizer was widened."""
    tests = bm.kt.discover_tests(os.path.join(bm.ROOT, "conformance"))
    matrix = bm.matrix_rows()
    assert len(tests) > 300, f"only {len(tests)} tests discovered — the recognizer is too narrow"
    assert len(matrix) > 300, f"only {len(matrix)} §9 matrix rows — the parser is too narrow"
    both = [t for t in tests if t in matrix and tests[t].get("clauses")]
    assert len(both) > 20, (
        f"only {len(both)} tests are both matrix-bound and citing; the comparator would have "
        f"almost nothing to range over and a zero would mean little")


def test_the_two_recognizers_are_kiss_traces_own():
    """⚠️ NOT A SECOND PARSER. If this tool recognized a different set of tests or citations than
    `kiss_trace`, a 'disagreement' could be between two PARSERS rather than two RECORDS."""
    assert bm.kt.discover_tests is not None
    assert bm.kt.RE_MATRIX is not None
    # the module must not define its own clause/test regexes
    src = open(os.path.join(bm.HERE, "kiss_backs_matrix.py"), encoding="utf-8").read()
    assert "re.compile" not in src, (
        "kiss_backs_matrix must reuse kiss_trace's recognizers, not define its own")


def main():
    tests = [v for k, v in sorted(globals().items())
             if k.startswith("test_") and callable(v)]
    for t in tests:
        t()
    print(f"ok - {len(tests)} controls pass: the #475 shape is flagged, the three legitimate "
          f"shapes are not, the scan population is non-trivial, and no second parser exists")
    return 0


if __name__ == "__main__":
    sys.exit(main())
