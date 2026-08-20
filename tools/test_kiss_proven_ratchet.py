"""Fixture controls for the 4th ratchet dimension — the PROVEN gate (#278, step 2b).

PROVEN starts at 0 and the live tree reports 0 -> 0 forever, so the live ratchet can never
exercise the drop arm — the same shape as the tier itself, and the reason #295 needed a
positive control. These drive `classify_proven` DIRECTLY with a fixture: a non-zero floor and
a seeded drop, so the gate's arms are proven rather than asserted (#279).

The ruling (architect, #278 2b): a proof drop is GREEN iff its proving TEST no longer exists
in the harness; a marker removed while the test SURVIVES is a regression, always. The gate
asks about the proving TEST, not whether any count moved.

Run: python tools/test_kiss_proven_ratchet.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt  # noqa: E402

C = "KISS-OPS-6.5-0001"
T = "prove_it"


def _verdict(floor_proven, live_map, harness, base=None):
    v, _lines = kt.classify_proven(floor_proven, live_map, set(harness), base_proven_map=base)
    return v


def test_at_floor_is_green():
    assert _verdict(0, {}, set()) == "proven_at_floor"
    assert _verdict(1, {C: [T]}, {T}) == "proven_at_floor"


def test_a_new_proof_is_improved_bump_the_floor():
    """live > floor: a proof earned. Not silently green — the floor must be bumped up, like a
    `stale` improvement, so green stays AT the floor."""
    assert _verdict(0, {C: [T]}, {T}) == "proven_improved"


def test_a_drop_whose_test_is_GONE_is_a_legitimate_retirement():
    """The proving test was removed, so the testimony is void — green (retirement). The floor
    is bumped DOWN in the same PR."""
    v = _verdict(1, {}, harness=set(), base={C: [T]})   # T not in harness
    assert v == "proven_retired", f"a test-gone drop was not a retirement: {v!r}"


def test_a_drop_whose_test_SURVIVES_is_a_regression():
    """Marker removed while the proving test still stands: a stale proof. RED, always — this
    is the exact case the architect's gate turns on, and it differs from the retirement above
    ONLY by whether the test is still in the harness."""
    v = _verdict(1, {}, harness={T}, base={C: [T]})     # T still present
    assert v == "proven_regression", f"a marker-gone-test-survives drop was not a regression: {v!r}"


def test_the_retirement_and_regression_differ_only_by_test_existence():
    """Nail the discriminator: identical floor/live/base, the ONLY difference is whether the
    proving test exists — and that flips retirement to regression. A gate keyed on 'did a count
    move' (the false positive the architect caught) could not tell these apart."""
    base = {C: [T]}
    assert _verdict(1, {}, harness=set(), base=base) == "proven_retired"
    assert _verdict(1, {}, harness={T}, base=base) == "proven_regression"


def test_a_drop_with_no_base_is_fail_closed():
    """live < floor and no base proven map: which proof dropped and whether its test survives
    cannot be told, so it is refused (never passed) — the deferred-base-reader path."""
    v = _verdict(1, {}, harness={T}, base=None)
    assert v == "proven_uncharacterized", f"an undeterminable drop was not fail-closed: {v!r}"


def test_a_multi_test_proof_reds_if_ANY_proving_test_survives():
    """A clause proven by two tests, dropped: green only if BOTH tests are gone. If one
    survives without the marker, that surviving test's testimony is stale — regression."""
    base = {C: [T, "other_prover"]}
    assert _verdict(1, {}, harness=set(), base=base) == "proven_retired"          # both gone
    assert _verdict(1, {}, harness={"other_prover"}, base=base) == "proven_regression"  # one survives


def main():
    tests = [v for k, v in sorted(globals().items())
             if k.startswith("test_") and callable(v)]
    for t in tests:
        t()
    print(f"ok - {len(tests)} controls pass: the PROVEN gate greens a test-gone retirement, "
          f"reds a marker-gone-test-survives loss, and fail-closes an undeterminable drop")
    return 0


if __name__ == "__main__":
    sys.exit(main())
