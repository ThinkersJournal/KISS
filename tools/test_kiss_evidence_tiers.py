"""Controls for the evidence-strength tiers (#278): NAMED / CITED / PROVEN.

The single `backed` count spans three strengths of evidence (convention 15): a name
coincidence (NAMED), a human-asserted citation (CITED, deliberateness), and a
mutation-verified backing (PROVEN, aboutness). `compute_evidence_tiers` partitions
`backed` into NAMED + CITED and carves PROVEN as a subset of CITED. These controls pin
the two invariants the report rests on — the partition and the subset — and drive the
real function main() calls, not a copy.

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
    return backed, cited


def test_live_tiers_partition_the_backed_set():
    """On the real tree: NAMED + CITED partition backed, PROVEN ⊆ CITED. Reddens if a
    future change makes the tiers overlap or leaves a backed clause in neither."""
    backed, cited = _load_live()
    named, cited_set, proven_set = kt.compute_evidence_tiers(backed, cited)
    assert named | cited_set == backed, "live: named union cited != backed"
    assert named & cited_set == set(), "live: named intersect cited overlap"
    assert proven_set <= cited_set, "live: proven not-subset-of cited"
    assert proven_set == set(), "live: PROVEN is non-empty but no proof mechanism exists yet"


def main():
    tests = [v for k, v in sorted(globals().items())
             if k.startswith("test_") and callable(v)]
    for t in tests:
        t()
    print(f"ok - {len(tests)} controls pass: NAMED+CITED partition backed, PROVEN subset-of CITED, "
          f"proof record empty until the reserved #278 mechanism")
    return 0


if __name__ == "__main__":
    sys.exit(main())
