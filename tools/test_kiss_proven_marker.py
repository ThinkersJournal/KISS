"""Controls for the `// Proven:` marker parser and collector (#278, step 2).

A `// Proven:` marker is TESTIMONY that a seeded mutation of a clause's obligation was
shown to redden a test — aboutness, the strongest evidence tier. It is not a live re-run,
so it must carry the mutation SUBJECT (convention 15) and a resolvable REF (16(a)), and it
earns the PROVEN tier only where the SAME test also BACKS the clause (PROVEN subset-of
CITED). Everything else is fail-closed: a malformed marker, or a proof over an unbacked
clause, is FLAGGED, never counted — the tier is worth having only if nothing enters it
unearned.

These drive the real `_proven_markers` / `collect_proven` that main() calls.
Run: python tools/test_kiss_proven_marker.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
CID = "KISS-OPS-6.5-0001"


def _test(clauses=(), proven=None, malformed=()):
    """One synthetic discovered test, shaped like discover_tests output."""
    return {"clauses": set(clauses), "proven": dict(proven or {}),
            "proven_malformed": list(malformed)}


# ----------------------------------------------------------------- parser --

def test_wellformed_marker_parses_subject_and_ref():
    well, bad = kt._proven_markers(
        f"// Backs: {CID}\n// Proven: {CID} (subject: impl; ref: PR#291)")
    assert well == {CID: ("impl", "PR#291")}, f"well-formed not parsed: {well!r}"
    assert bad == [], f"well-formed wrongly flagged malformed: {bad!r}"


def test_spec_text_subject_is_accepted():
    well, bad = kt._proven_markers(f"// Proven: {CID} (subject: spec-text; ref: 5060252)")
    assert well == {CID: ("spec-text", "5060252")}, f"spec-text subject rejected: {well!r}"


def test_marker_without_subject_or_ref_is_malformed():
    well, bad = kt._proven_markers(f"// Proven: {CID}")
    assert well == {} and bad == [CID], f"bare Proven not flagged: well={well!r} bad={bad!r}"


def test_marker_missing_ref_is_malformed():
    well, bad = kt._proven_markers(f"// Proven: {CID} (subject: impl)")
    assert well == {} and bad == [CID], f"missing-ref not flagged: well={well!r} bad={bad!r}"


def test_marker_with_invalid_subject_is_malformed():
    """subject must be impl|spec-text — a free-text subject does not parse and is flagged."""
    well, bad = kt._proven_markers(f"// Proven: {CID} (subject: vibes; ref: x)")
    assert well == {} and bad == [CID], f"bad-subject not flagged: well={well!r} bad={bad!r}"


def test_a_wellformed_marker_does_not_mask_a_malformed_one_for_the_same_clause():
    """FAIL-CLOSED REGARDLESS (Copilot #295): a stray bare `// Proven: X` alongside a correct
    one for the SAME X must still be flagged AND must poison X's credit — good evidence cannot
    mask bad, the exact class the PROVEN tier exists to expose. BORN RED before the span-based
    detection: the earlier id-subtraction let the well-formed marker hide the stray."""
    scope = f"// Proven: {CID} (subject: impl; ref: PR#291)\n// Proven: {CID}"
    well, bad = kt._proven_markers(scope)
    assert bad == [CID], f"stray marker masked by a sibling well-formed one: {bad!r}"
    assert well == {}, f"a clause with a malformed marker kept its credit: {well!r}"


# -------------------------------------------------------------- collector --

def test_proven_maps_the_clause_to_its_proving_test():
    """A well-formed marker in a test that BACKS the clause earns PROVEN, no violation — and
    collect_proven returns the clause MAPPED TO ITS PROVING TEST, not a bare set. The 4th
    ratchet dimension's drop gate needs that identity (a proof drop is green iff its proving
    test is gone), so the map, not the set, is the contract."""
    harness = {"t": _test(clauses=[CID], proven={CID: ("impl", "PR#291")})}
    pmap, viol = kt.collect_proven(harness)
    assert pmap == {CID: ["t"]}, f"proving-test identity not mapped: {pmap!r}"
    assert viol == [], f"unexpected violation: {viol!r}"


def test_proof_without_a_backing_is_a_violation():
    """A marker in a test that does NOT back the clause is proof over nothing — flagged,
    not counted (PROVEN subset-of CITED)."""
    harness = {"t": _test(clauses=[], proven={CID: ("impl", "PR#291")})}
    pmap, viol = kt.collect_proven(harness)
    assert pmap == {}, f"proof without a backing was credited: {pmap!r}"
    assert len(viol) == 1 and "does NOT back" in viol[0], f"not flagged: {viol!r}"


def test_malformed_marker_is_a_violation_and_uncounted():
    harness = {"t": _test(clauses=[CID], malformed=[CID])}
    pmap, viol = kt.collect_proven(harness)
    assert pmap == {}, f"malformed marker was counted: {pmap!r}"
    assert len(viol) == 1 and "malformed" in viol[0], f"not flagged: {viol!r}"


def test_clean_tree_has_no_proven_and_no_violations():
    harness = {"t": _test(clauses=[CID])}          # backs, but no proof marker
    pmap, viol = kt.collect_proven(harness)
    assert pmap == {} and viol == [], f"clean tree not clean: {pmap!r} {viol!r}"


# -------------------------------------------------------------------- live --

def _live_harness():
    return kt.discover_tests(os.path.join(ROOT, "conformance"))


def test_live_tree_is_armed_and_empty():
    """The floor starts at 0: no `// Proven:` markers exist yet, so PROVEN is empty and there
    are no malformed markers to flag. Reddens the instant a marker is added wrong (missing
    subject/ref, or over an unbacked clause) — the fail-closed guard the burndown rests on."""
    pmap, viol = kt.collect_proven(_live_harness())
    assert viol == [], f"live tree has proven-marker violations: {viol}"
    assert pmap == {}, f"live PROVEN is non-empty ({sorted(pmap)}) — expected 0 (armed, empty)"


def main():
    tests = [v for k, v in sorted(globals().items())
             if k.startswith("test_") and callable(v)]
    for t in tests:
        t()
    print(f"ok - {len(tests)} controls pass: the `// Proven:` marker parses subject+ref, "
          f"fail-closes on malformed/unbacked, and the live tree is armed and empty")
    return 0


if __name__ == "__main__":
    sys.exit(main())
