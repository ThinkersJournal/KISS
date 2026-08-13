"""Discrimination controls for the citation audit (`kiss_cites.py`).

The audit reports on the integrity of every coverage number the project quotes,
so an audit that cannot fail is precisely the thing it exists to find. Per
evidence convention 9 and the §6.5-0011 principle: an instrument that has never
been shown to reject the wrong input supplies no evidence.

Each case below is a synthetic spec + harness pair exhibiting exactly ONE
citation shape, and asserts the audit sorts it into exactly one bucket:

  1. ASSERTION      clause ID inside `assert!` -> NOT a candidate
  2. ASSERT HELPER  clause ID as the first arg of a project `assert_*` helper
                    -> NOT a candidate (the shape a fixed macro list would miss)
  3. CODE AS DATA   clause ID bound as a spec-lookup key -> PRIMARY candidate
  4. COMMENT        affirmative comment citation -> sanctioned, not actionable
  5. CONTRASTIVE    comment wording that disclaims backing -> secondary candidate
  6. FORWARD        the clause's named test exists -> out of scope entirely

Case 1 and 2 are the CONTROLS. Without them an audit that flagged everything
would pass cases 3 and 5 and look correct.

Run: python tools/test_kiss_cites.py
"""
import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_cites as kc

SPEC = """# KISS-Probe

## 6. Specification

### 6.0 Cases

- **KISS-PROBE-6.0-0001** — the assertion case. *Test:* `probe_named_0001`.
- **KISS-PROBE-6.0-0002** — the helper case. *Test:* `probe_named_0002`.
- **KISS-PROBE-6.0-0003** — the data case. *Test:* `probe_named_0003`.
- **KISS-PROBE-6.0-0004** — the comment case. *Test:* `probe_named_0004`.
- **KISS-PROBE-6.0-0005** — the contrastive case. *Test:* `probe_named_0005`.
- **KISS-PROBE-6.0-0006** — the forward case. *Test:* `probe_named_0006`.

## 9. Conformance

| Clause | Test |
|---|---|
| KISS-PROBE-6.0-0001 | `probe_named_0001` |
| KISS-PROBE-6.0-0002 | `probe_named_0002` |
| KISS-PROBE-6.0-0003 | `probe_named_0003` |
| KISS-PROBE-6.0-0004 | `probe_named_0004` |
| KISS-PROBE-6.0-0005 | `probe_named_0005` |
| KISS-PROBE-6.0-0006 | `probe_named_0006` |
"""

# Every citing test is deliberately named something OTHER than the clause's
# `*Test:*` name, so backing can only arrive by the reverse direction — except
# case 6, which is named correctly and must therefore be forward-backed.
HARNESS = '''
#[test]
fn cites_in_an_assertion() {
    assert!(thing(), "KISS-PROBE-6.0-0001: the thing must hold");
}

#[test]
fn cites_via_a_project_helper() {
    assert_token("KISS-PROBE-6.0-0002", &k, "expected|token");
}

#[test]
fn cites_as_a_lookup_key() {
    let home = "KISS-PROBE-6.0-0003";
    let block = clause_block(&spec, home);
    assert!(block.contains("something"), "KISS-PROBE-6.0-0099: unrelated cite");
}

/// Enforces KISS-PROBE-6.0-0004 - the affirmative comment form.
#[test]
fn cites_in_an_affirmative_comment() {
    assert_eq!(1 + 1, 2);
}

/// This test does not enforce KISS-PROBE-6.0-0005; cross-references use it only.
#[test]
fn cites_in_a_contrastive_comment() {
    assert_eq!(1 + 1, 2);
}

#[test]
fn probe_named_0006() {
    assert!(other(), "KISS-PROBE-6.0-0006: forward-backed by name");
}
'''


def run_audit():
    with tempfile.TemporaryDirectory() as td:
        spec_dir = os.path.join(td, "spec")
        conf_dir = os.path.join(td, "conformance", "tests")
        os.makedirs(spec_dir)
        os.makedirs(conf_dir)
        open(os.path.join(spec_dir, "probe.md"), "w", encoding="utf-8").write(SPEC)
        open(os.path.join(conf_dir, "probe.rs"), "w", encoding="utf-8").write(HARNESS)
        saved = kc.kt.SPECS
        try:
            kc.kt.SPECS = ["probe"]
            return kc.audit(spec_dir, os.path.dirname(conf_dir))
        finally:
            kc.kt.SPECS = saved


def main():
    rows, st = run_audit()
    by_clause = {r["clause"]: r["bucket"] for r in rows}
    fails = []

    def expect(clause, bucket, why):
        got = by_clause.get(clause, "<not a candidate>")
        if got != bucket:
            fails.append(f"  {clause}: expected {bucket!r}, got {got!r} - {why}")

    # CONTROLS: these must NOT be flagged. An audit that flags everything passes
    # the positive cases below and is worthless; these are what refute it.
    if "KISS-PROBE-6.0-0001" in by_clause:
        fails.append("  KISS-PROBE-6.0-0001: an ASSERTION citation was flagged - "
                     "the audit does not discriminate, it flags everything")
    if "KISS-PROBE-6.0-0002" in by_clause:
        fails.append("  KISS-PROBE-6.0-0002: a project assert_* HELPER citation was "
                     "flagged - the assertion match is a fixed macro list, not a shape")

    # POSITIVES: the shapes the audit exists to surface.
    expect("KISS-PROBE-6.0-0003", "code_no_assertion",
           "a spec-lookup key is the shape that motivated this audit")
    expect("KISS-PROBE-6.0-0005", "comment_contrastive",
           "wording that disclaims backing must sort below an affirmative one")

    # SANCTIONED: reported, but never as an actionable candidate.
    expect("KISS-PROBE-6.0-0004", "comment_affirmative",
           "the documented comment form must not be presented as a defect")

    # SCOPE: a forward-backed clause is out of scope by construction.
    if "KISS-PROBE-6.0-0006" in by_clause:
        fails.append("  KISS-PROBE-6.0-0006: a FORWARD-backed clause was audited - "
                     "its credit comes from the test NAME, so mention-vs-backing "
                     "cannot arise and flagging it is a false positive")
    if st["forward"] != 1:
        fails.append(f"  forward count is {st['forward']}, expected 1")
    if st["reverse_only"] != 5:
        fails.append(f"  reverse_only count is {st['reverse_only']}, expected 5")

    if fails:
        print("FAIL - the citation audit does not discriminate:")
        print("\n".join(fails))
        return 1
    print("ok - citation audit discriminates all six citation shapes")
    print(f"     forward={st['forward']} reverse_only={st['reverse_only']} "
          f"assertion={st['assertion']} code={st['code_no_assertion']} "
          f"contrastive={st['comment_contrastive']} affirmative={st['comment_affirmative']}")
    return 0


def test_kiss_cites_discrimination():
    """Collected by pytest; CI also runs the file in script mode.

    Without this, `pytest tools/` collects ZERO tests from a file named
    `test_*.py` and reports success having executed none of the controls above —
    the vacuity mechanism, in the file whose job is to prove the citation audit
    is not vacuous. CI gates on this explicitly (#158's shape), and it caught the
    omission here.
    """
    assert main() == 0, "the citation audit failed its discrimination controls"


if __name__ == "__main__":
    sys.exit(main())
