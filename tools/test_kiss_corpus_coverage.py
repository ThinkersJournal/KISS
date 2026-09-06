#!/usr/bin/env python3
"""Discrimination test for the corpus op-coverage ratchet (#459).

The ratchet must red for a coverage LOSS and for a DOMAIN move, and — the load-bearing part —
its MESSAGE must say WHICH. kiss_trace's `_untested_rose` shipped the wrong reason for a year
because every test pinned the VERDICT (`v == "regression"`) and none pinned the string; a
verdict can be right for a reason that is false. So these assert the message text, not just the
verdict. Written as a pytest-collectable `test_*` (CI requires every tools/test_*.py collect one).
"""
import sys, os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from kiss_ops import classify_corpus_coverage  # noqa: E402


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


if __name__ == "__main__":
    test_corpus_coverage_ratchet_verdicts_and_reasons()
    print("RESULT: CLEAN")
