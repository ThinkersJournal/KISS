#!/usr/bin/env python3
"""Discrimination test for the corpus op-coverage ratchet (#459).

The ratchet must red for a coverage LOSS and for a DOMAIN move, and — the load-bearing part —
its MESSAGE must say WHICH. kiss_trace's `_untested_rose` shipped the wrong reason for a year
because every test pinned the VERDICT (`v == "regression"`) and none pinned the string; a
verdict can be right for a reason that is false. So these assert the message text, not just the
verdict. Written as a pytest-collectable `test_*` (CI requires every tools/test_*.py collect one).
"""
import sys, os, io, tempfile, contextlib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kiss_ops import (  # noqa: E402
    classify_corpus_coverage,
    read_corpus_floor,
    corpus_coverage_ratchet,
)


def _write_tsv(text):
    f = tempfile.NamedTemporaryFile("w", suffix=".tsv", delete=False, encoding="utf-8")
    f.write(text)
    f.close()
    return f.name


def _value_error(fn):
    """Return the ValueError message fn() raises, or None if it raised no ValueError."""
    try:
        fn()
    except ValueError as e:
        return str(e)
    return None


def test_corpus_coverage_ratchet_verdicts_and_reasons():
    # AT FLOOR — the only green.
    ok, v, lines = classify_corpus_coverage(1, 106, 1, 106)
    assert ok and v == "at-floor", (v, lines)
    assert any("1/106" in l for l in lines), "at-floor must report the fraction"

    # REGRESSION — declared dropped below the floor (numerator down).
    ok, v, lines = classify_corpus_coverage(2, 106, 1, 106)
    msg = " ".join(lines)
    assert not ok and v == "regression", v
    assert "BACKWARD" in msg, msg
    assert "DOMAIN moved" not in msg, "a loss must not be reported as a domain move"

    # STALE / PROGRESS — declared grew: bump the floor, not a regression.
    ok, v, lines = classify_corpus_coverage(1, 106, 2, 106)
    msg = " ".join(lines)
    assert not ok and v == "stale", v
    assert "PROGRESS" in msg and "bump" in msg, msg

    # DENOMINATOR MOVED — declared held, total grew (ops.md added an op): NOT a regression.
    ok, v, lines = classify_corpus_coverage(1, 106, 1, 107)
    msg = " ".join(lines)
    assert not ok and v == "denominator-moved", v
    assert "NOT a coverage regression" in msg, msg
    assert "BACKWARD" not in msg, msg

    # ⚠️ THE #445 DISCRIMINATOR: the two FAILING-but-different cases (a real loss vs a domain move
    # that shifts the fraction) must NOT share a message — a reader acts on the reason, not the
    # verdict, and both are red.
    _, _, reg = classify_corpus_coverage(2, 106, 1, 106)
    _, _, den = classify_corpus_coverage(1, 106, 1, 107)
    assert " ".join(reg) != " ".join(den), "loss and domain-move must give DIFFERENT reasons"


def test_read_corpus_floor_parses_valid_tsv():
    """Control: comments and blank lines are ignored, key/int pairs parsed."""
    p = _write_tsv("# the burn-down floor\ndeclared\t1\n\ntotal\t106\n")
    try:
        assert read_corpus_floor(p) == {"declared": 1, "total": 106}
    finally:
        os.unlink(p)


def test_read_corpus_floor_names_the_offending_line_on_malformed_input():
    """A hand-editable TSV: a dropped tab or a non-integer value must raise an error that NAMES
    the line, not a bare `int('')` / `int('many')` crash that leaves the editor guessing. #468."""
    # A tab replaced by a space — the classic hand-edit — must name the line and its number.
    p = _write_tsv("declared 1\n")
    try:
        msg = _value_error(lambda: read_corpus_floor(p))
        assert msg is not None, "a line without a tab must raise ValueError"
        assert "declared 1" in msg and ":2:" not in msg and ":1:" in msg, msg
    finally:
        os.unlink(p)
    # A non-integer value must name the KEY (not just repeat int()'s opaque message).
    p = _write_tsv("declared\tmany\n")
    try:
        msg = _value_error(lambda: read_corpus_floor(p))
        assert msg is not None and "declared" in msg and "many" in msg, msg
    finally:
        os.unlink(p)


def test_corpus_coverage_ratchet_is_fatal_and_names_a_missing_key():
    """A floor missing `declared`/`total` must produce a clear FATAL + rc 1, not a bare KeyError
    traceback out of classify_corpus_coverage. #468."""
    spec_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "spec")
    p = _write_tsv("declared\t1\n")  # `total` absent
    try:
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc = corpus_coverage_ratchet(spec_dir, p)
        out = buf.getvalue()
        assert rc == 1, out
        assert "FATAL" in out and "total" in out, out
    finally:
        os.unlink(p)


if __name__ == "__main__":
    test_corpus_coverage_ratchet_verdicts_and_reasons()
    test_read_corpus_floor_parses_valid_tsv()
    test_read_corpus_floor_names_the_offending_line_on_malformed_input()
    test_corpus_coverage_ratchet_is_fatal_and_names_a_missing_key()
    print("RESULT: CLEAN")
