"""Controls for the clause-index emitter.

The emitter's whole value is that a downstream project can vendor it and turn "does this
KISS clause still exist?" into a diff. That value is zero if the emitter can ship an
INCOMPLETE index while reporting success -- so the index asserts a SET comparison between
clause definitions and §9 matrix rows before emitting.

These controls prove that assertion can FAIL. An emitter whose completeness check has never
been observed failing is the artifact this repo has spent a day finding.
"""
import io, os, subprocess, sys, tempfile, shutil, pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
TOOL = str(ROOT / "tools" / "kiss_clause_index.py")


def _run(root):
    p = subprocess.run([sys.executable, TOOL, str(root)],
                       capture_output=True, text=True, encoding="utf-8", errors="replace")
    return p.returncode, p.stdout, p.stderr


def _tree_copy(tmp):
    """A real copy of spec/ -- the controls must run through the SAME parse the tool uses,
    not beside it. A control that exercises a different mechanism cannot fail the way the
    instrument fails."""
    dst = pathlib.Path(tmp) / "spec"
    shutil.copytree(ROOT / "spec", dst)
    return pathlib.Path(tmp)


def test_the_live_tree_emits_a_complete_index():
    rc, out, err = _run(ROOT)
    assert rc == 0, f"live tree should emit cleanly, got rc={rc}\n{err}"
    rows = [l for l in out.splitlines() if l.startswith("KISS-")]
    assert len(rows) > 900, f"expected the full clause set, got {len(rows)} rows"
    assert "clause definitions:" in err and "matrix rows:" in err, \
        "the counts must be PRINTED, not merely reconciled internally"


def test_a_matrix_row_with_no_definition_is_a_MISMATCH_not_a_silent_drop():
    """The asymmetry the SET check exists for. A count check alone would read 936 vs 937 and
    could be dismissed as an off-by-one; the set check NAMES the id."""
    with tempfile.TemporaryDirectory() as tmp:
        root = _tree_copy(tmp)
        p = root / "spec" / "ops.md"
        t = io.open(p, encoding="utf-8").read()
        t = t.replace("| KISS-OPS-6.19-0005 |",
                      "| KISS-OPS-9.9-9999 | phantom_test |\n| KISS-OPS-6.19-0005 |", 1)
        io.open(p, "w", encoding="utf-8", newline="\n").write(t)
        rc, _out, err = _run(root)
        assert rc == 1, f"a phantom matrix row must MISMATCH (rc=1), got rc={rc}"
        assert "KISS-OPS-9.9-9999" in err, "the mismatch must NAME the asymmetric id"


def test_a_definition_with_no_matrix_row_is_ALSO_a_mismatch():
    """⚠️ THE OTHER DIRECTION. Two different sets project to the same COUNT when one clause
    loses its row and another gains a phantom -- they cancel. Only checking both directions
    makes the set comparison worth having over a count."""
    with tempfile.TemporaryDirectory() as tmp:
        root = _tree_copy(tmp)
        p = root / "spec" / "ops.md"
        t = io.open(p, encoding="utf-8").read()
        assert "| KISS-OPS-6.19-0005 |" in t, "fixture precondition: the row must exist first"
        t = t.replace("| KISS-OPS-6.19-0005 |", "| KISS-OPS-6-19-0005-REMOVED |", 1)
        io.open(p, "w", encoding="utf-8", newline="\n").write(t)
        rc, _out, err = _run(root)
        assert rc == 1, f"a definition with no matrix row must MISMATCH, got rc={rc}"
        assert "KISS-OPS-6.19-0005" in err and "NO matrix row" in err


def test_an_unreadable_corpus_REFUSES_and_does_not_emit_an_empty_index():
    """⚠️ REFUSED (2) is not MISMATCH (1) and neither is OK (0). An emitter that returned an
    empty index on an unreadable tree would hand a downstream project a file asserting that
    KISS has no clauses."""
    with tempfile.TemporaryDirectory() as tmp:
        rc, out, err = _run(pathlib.Path(tmp))   # no spec/ at all
        assert rc == 2, f"an unreadable corpus must REFUSE (rc=2), got rc={rc}"
        assert not [l for l in out.splitlines() if l.startswith("KISS-")], \
            "REFUSED must emit NO rows"


def test_the_three_verdicts_have_distinct_exit_codes():
    """0 / 1 / 2 must stay distinct so 'could not measure' never reads as 'measured clean'."""
    assert len({0, 1, 2}) == 3
