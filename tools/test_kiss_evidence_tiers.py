"""Controls for the evidence-strength tiers (#278): NAMED / CITED / PROVEN.

The single `backed` count spans three strengths of evidence (convention 15): a name
coincidence (NAMED), a human-asserted citation (CITED, deliberateness), and a
mutation-verified backing (PROVEN, aboutness). `compute_evidence_tiers` partitions
`backed` into NAMED + CITED and carves PROVEN as a subset of CITED. These controls pin
the invariants the report rests on — the partition, and a NON-EMPTY PROVEN inside
CITED — and drive the real function main() calls, not a copy.

Run: python tools/test_kiss_evidence_tiers.py
"""
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)


# ----------------------------------------------------------- invariants --

def test_named_and_cited_partition_backed():
    """NAMED and CITED are disjoint and together cover exactly `backed`."""
    backed = {"KISS-OPS-6.5-0001", "KISS-OPS-6.5-0002", "KISS-OPS-6.5-0003"}
    cited = {"KISS-OPS-6.5-0002": {"t_b"}, "KISS-OPS-6.5-0003": {"t_c"}}
    named, cited_set, _proven = kt.compute_evidence_tiers(backed, cited)
    assert named == {"KISS-OPS-6.5-0001"}, f"named wrong: {named!r}"
    assert cited_set == {"KISS-OPS-6.5-0002", "KISS-OPS-6.5-0003"}, f"cited wrong: {cited_set!r}"
    assert named | cited_set == backed, "named union cited != backed"
    assert named & cited_set == set(), "named intersect cited != empty"


def test_a_backed_uncited_clause_is_named_not_cited():
    """The #278 population: a §9 name matching a real fn with NO citation is a name
    coincidence — NAMED, and must not be counted as an asserted backing."""
    backed = {"KISS-OPS-6.5-0009"}
    cited = {}                                    # nobody cited it in a backing form
    named, cited_set, _ = kt.compute_evidence_tiers(backed, cited)
    assert named == {"KISS-OPS-6.5-0009"} and cited_set == set(), (
        f"a name coincidence was miscounted as cited: named={named!r} cited={cited_set!r}")


def test_proven_is_a_subset_of_cited():
    """PROVEN ⊆ CITED: a proof presupposes the citation. A proven clause that IS cited is
    kept; a 'proven' clause that is NOT cited is dropped, so the invariant holds by
    construction — recording a proof for a NAMED-only clause requires migrating it to CITED
    first, never asserting proof over a bare name."""
    backed = {"KISS-OPS-6.5-0004", "KISS-OPS-6.5-0005"}
    cited = {"KISS-OPS-6.5-0004": {"t_d"}}        # only 0004 is cited
    proven = {"KISS-OPS-6.5-0004", "KISS-OPS-6.5-0005"}  # claim both proven
    named, cited_set, proven_set = kt.compute_evidence_tiers(backed, cited, proven=proven)
    assert proven_set == {"KISS-OPS-6.5-0004"}, (
        f"proven leaked a non-cited clause (subset-of broken): {proven_set!r}")
    assert proven_set <= cited_set, "proven_set not-subset-of cited_set"


def test_proven_defaults_empty():
    """No proof record → PROVEN is empty. The honest current state (#278): the only tier
    that is actual evidence has no number until the reserved mechanism lands."""
    backed = {"KISS-OPS-6.5-0006"}
    cited = {"KISS-OPS-6.5-0006": {"t_e"}}
    _named, _cited, proven_set = kt.compute_evidence_tiers(backed, cited, proven=None)
    assert proven_set == set(), f"proven should default empty, got {proven_set!r}"


# --------------------------------------------------------------- live --

def _load_live():
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
    backed = {c for c, t in clause_test.items() if t in harness or c in cited}
    # The PROVEN set the live tree actually carries. `collect_proven` is the function
    # that PRODUCES this argument, and omitting it was the defect (#405): the default
    # is None -> empty, so every downstream assertion about PROVEN passed by default.
    proven_map, _violations = kt.collect_proven(harness)
    return backed, cited, set(proven_map)


def test_live_tiers_partition_the_backed_set():
    """On the real tree: NAMED + CITED partition backed, PROVEN is non-empty and no
    recorded proof was silently dropped. Reddens if a future change makes the tiers overlap, leaves a backed clause
    in neither, or lets a proof exist for a clause nothing cites.

    ⚠️ THE LIVE PROVEN SET IS PASSED, NOT DEFAULTED (#405). This call previously omitted
    the third argument, so `proven_set` was the empty default and BOTH assertions about
    it were vacuous: `empty <= anything` holds, and `empty == set()` compares the default
    to itself. The docstring advertised a live PROVEN check the code did not perform."""
    backed, cited, proven = _load_live()
    named, cited_set, proven_set = kt.compute_evidence_tiers(backed, cited, proven=proven)
    assert named | cited_set == backed, "live: named union cited != backed"
    assert named & cited_set == set(), "live: named intersect cited overlap"
    # ⚠️ NON-EMPTINESS GUARDS THE SUBSET CHECK. `proven_set <= cited_set` is satisfied by
    # an EMPTY proven set, so without this the subset assertion silently reverts to the
    # defect above the moment PROVEN is empty for any reason.
    #
    # If PROVEN is ever legitimately empty -- every proof retired under the sanctioned
    # drop rule -- this assertion is a PROMPT, not a false alarm: the subset check has
    # gone vacuous and must be REPLACED, never merely deleted.
    assert proven_set, (
        "live: PROVEN is EMPTY, so the drop check below compares two empty sets and "
        "asserts nothing. If the emptiness is real (all proofs retired), replace this "
        "control rather than delete it -- a set comparison over two empty sets is "
        "check-shaped and reports success.")
    # ⚠️ NOT `proven_set <= cited_set`. That direction is UNFALSIFIABLE here:
    # `compute_evidence_tiers` drops any proven element not in cited_set, so the subset
    # holds "by construction rather than by the caller's discipline" (its own docstring).
    # Asserting it would be a tautology wearing a control's clothes -- verified: injecting
    # a proof for a clause nothing cites leaves that assertion GREEN.
    #
    # The falsifiable neighbour is the DROP: a `// Proven:` marker for an uncited clause
    # vanishes silently at this layer, so the tree's proof record and the counted set
    # would disagree with nothing reporting it. That is what this asserts.
    dropped = proven - proven_set
    assert not dropped, (
        f"live: {len(dropped)} recorded proof(s) were SILENTLY DROPPED because their clause "
        f"is not CITED -- e.g. {sorted(dropped)[:3]}. A `// Proven:` marker for an uncited "
        f"clause disappears here without reporting; migrate the clause to CITED or remove "
        f"the marker.")
    # NO COUNT IS ASSERTED HERE. The number is COVERAGE_FLOOR.tsv's ratchet dimension;
    # a hand-written count in a second place is the count-valued-floor defect (two
    # parties each legitimately moving it leave a figure neither holds).


def main():
    tests = [v for k, v in sorted(globals().items())
             if k.startswith("test_") and callable(v)]
    for t in tests:
        t()
    print(f"ok - {len(tests)} controls pass: NAMED+CITED partition backed, and the LIVE "
          f"PROVEN set is non-empty and no recorded proof was silently dropped")
    return 0


if __name__ == "__main__":
    sys.exit(main())
