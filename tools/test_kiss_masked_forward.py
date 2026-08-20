"""Controls for the MASKED FORWARD NAME detector and the DECLARED_SHARES allow-list (#286).

A clause's §9 `*Test:*` names its forward test. The `no such test` report catches a
fictional name ONLY for a clause that is otherwise unbacked — so a clause backed by
REVERSE citation (a real test cites it) can carry a dead forward name that nothing sees.
That is a "masked forward name." `compute_masked_forward` is the detector; once the rows
are true its live population is ZERO, so — per the #279 finding that a check which cannot
be seen to fire is indistinguishable from a broken one — this file is the ONLY evidence
the detector functions. `test_detector_fires_on_a_reverse_backed_fictional_name` is that
born-red instrument: a synthetic fixture the detector MUST flag.

The allow-list half exercises `classify_shared_test`: an injective forward name is the
default; a share is sanctioned only as a `test_conform_*` cross-standard deferral or an
EXACTLY-matching DECLARED_SHARES entry. An undeclared or superset share stays a violation.

These drive the real functions main() calls, not a re-implementation. Run:
    python tools/test_kiss_masked_forward.py
"""
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
SHARED_TEST = "test_classify_work_class_element_count"
SHARED_CLAUSES = frozenset({"KISS-CLASSIFY-6.5-0007", "KISS-CLASSIFY-6.5-0010"})


# ---------------------------------------------------------------- detector --

def test_detector_fires_on_a_reverse_backed_fictional_name():
    """BORN RED: a clause named `test_ghost` (in no harness) but cited by a real test is
    a masked forward name — backed, yet its §9 row resolves to nothing. This is the
    fixture that proves the detector CAN fire; without it a zero live population reads as
    a broken check (#279)."""
    clause_test = {"KISS-OPS-6.99-0001": "test_ghost"}      # §9 names a fiction
    harness = {"test_real_backing": {"clauses": {"KISS-OPS-6.99-0001"}}}
    cited = {"KISS-OPS-6.99-0001": {"test_real_backing"}}   # reverse-backed
    masked = kt.compute_masked_forward(clause_test, harness, cited)
    assert masked == {"KISS-OPS-6.99-0001": ("test_ghost", "test_real_backing")}, (
        f"detector failed to flag a reverse-backed fictional name: {masked!r}")


def test_detector_ignores_an_unbacked_fictional_name():
    """The 553 that must NEVER be flagged: a fictional name on an UNBACKED clause (uncited)
    is correct-by-design aspiration, not a masked row. Absent from harness AND uncited."""
    clause_test = {"KISS-OPS-6.99-0002": "test_aspirational"}
    harness = {"test_something_else": {"clauses": set()}}
    cited = {}                                              # nobody cites it
    masked = kt.compute_masked_forward(clause_test, harness, cited)
    assert masked == {}, f"detector wrongly flagged an unbacked aspirational name: {masked!r}"


def test_detector_ignores_a_real_forward_name():
    """A clause whose §9 name IS in the harness is forward-backed — never masked."""
    clause_test = {"KISS-OPS-6.99-0003": "test_real"}
    harness = {"test_real": {"clauses": {"KISS-OPS-6.99-0003"}}}
    cited = {"KISS-OPS-6.99-0003": {"test_real"}}
    masked = kt.compute_masked_forward(clause_test, harness, cited)
    assert masked == {}, f"detector wrongly flagged a forward-backed clause: {masked!r}"


# ------------------------------------------------------- allow-list (shares) --

def test_declared_share_exact_set_is_allowed():
    """The live DECLARED_SHARES entry, with its EXACT clause set, is a declared share."""
    kind = kt.classify_shared_test(SHARED_TEST, set(SHARED_CLAUSES), kt.DECLARED_SHARES)
    assert kind == "declared_share", f"exact declared share not allowed: {kind!r}"


def test_undeclared_same_standard_share_is_a_violation():
    """Two same-standard clauses on an UNDECLARED test reddens — omission is not sanction."""
    clauses = {"KISS-OPS-6.5-0001", "KISS-OPS-6.5-0002"}
    kind = kt.classify_shared_test("test_not_in_declared_shares", clauses, kt.DECLARED_SHARES)
    assert kind == "violation", f"undeclared share was not a violation: {kind!r}"


def test_superset_of_a_declared_share_is_a_violation():
    """A THIRD clause naming the declared test is a superset — NOT covered, so it reddens.
    The declared set must match the citing set exactly; drift must be added deliberately."""
    clauses = set(SHARED_CLAUSES) | {"KISS-CLASSIFY-6.5-0011"}
    kind = kt.classify_shared_test(SHARED_TEST, clauses, kt.DECLARED_SHARES)
    assert kind == "violation", f"a superset of a declared share was allowed: {kind!r}"


def test_cross_standard_conform_share_is_deferred():
    """The original sanctioned exception: a `test_conform_*` test cited by exactly one
    CONFORM clause plus a deferring sub-standard clause."""
    clauses = {"KISS-CONFORM-6.13-0006", "KISS-OPS-6.11-0004"}
    kind = kt.classify_shared_test("test_conform_reduce_mean", clauses, kt.DECLARED_SHARES)
    assert kind == "cross_standard", f"cross-standard conform share not deferred: {kind!r}"


def test_a_conform_test_naming_two_conform_clauses_is_a_violation():
    """The cross-standard arm requires exactly ONE CONFORM clause — two CONFORM clauses on
    a conform test is a same-standard collision, not a deferral."""
    clauses = {"KISS-CONFORM-6.13-0006", "KISS-CONFORM-6.13-0007"}
    kind = kt.classify_shared_test("test_conform_two", clauses, kt.DECLARED_SHARES)
    assert kind == "violation", f"a two-CONFORM conform share was deferred: {kind!r}"


# ------------------------------------------------------------- live guards --

def _load_live():
    """Build (clause_test, harness, cited) from the real spec/ + conformance/, exactly as
    main() does — so the guards below run the detector end-to-end against the tree."""
    spec_dir = os.path.join(ROOT, "spec")
    conf_dir = os.path.join(ROOT, "conformance")
    clause_test = {}
    for stem in kt.SPECS:
        res = kt.DocResult(stem)
        path = os.path.join(spec_dir, stem + ".md")
        if os.path.exists(path):
            kt.parse(path, res)
        for cid, t, _ln in res.matrix:
            clause_test[cid] = t
    harness = kt.discover_tests(conf_dir)
    cited = defaultdict(set)
    for tname, info in harness.items():
        for cid in info["clauses"]:
            cited[cid].add(tname)
    return clause_test, harness, cited


def test_live_tree_has_no_masked_forward():
    """After the #286 fix the live population is zero. This guard reddens the instant any
    clause reacquires a dead forward name under reverse backing — the ongoing enforcement
    the born-red control cannot give (it only proves the detector CAN fire)."""
    clause_test, harness, cited = _load_live()
    masked = kt.compute_masked_forward(clause_test, harness, cited)
    assert masked == {}, f"live tree carries {len(masked)} masked forward name(s): {sorted(masked)}"


def test_live_declared_share_matches_the_spec_rows():
    """The DECLARED_SHARES entry and the §9 matrices must agree: the clauses the spec rows
    actually assign to the shared test must equal the declared set. Catches a spec edit
    that repoints a row without updating the allow-list (or vice versa)."""
    clause_test, _harness, _cited = _load_live()
    rows_naming_it = {c for c, t in clause_test.items() if t == SHARED_TEST}
    declared = set(kt.DECLARED_SHARES[SHARED_TEST]["clauses"])
    assert rows_naming_it == declared, (
        f"§9 rows naming `{SHARED_TEST}` ({sorted(rows_naming_it)}) != "
        f"DECLARED_SHARES ({sorted(declared)})")


def main():
    tests = [v for k, v in sorted(globals().items())
             if k.startswith("test_") and callable(v)]
    for t in tests:
        t()
    print(f"ok - {len(tests)} controls pass: detector fires on a masked name, ignores the "
          f"aspirational 553, and the allow-list accepts only the exact declared share")
    return 0


if __name__ == "__main__":
    sys.exit(main())
