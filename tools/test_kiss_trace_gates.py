"""Negative controls for runtime-gate discovery in kiss_trace (KISS #140).

The defect: a test that declines at run time (`eprintln!("SKIP"); return;`) reports
`ok` while asserting nothing, and the clause it backs is credited anyway. The fix
makes the skip *declarable* via `runtime_gate!` / `runtime_gate_some!`, so the matrix
can see it.

These tests demonstrate the discovery DISCRIMINATES — it is not enough that a
declared gate is found; an undeclared skip must NOT be, and an ordinary test must
not be mistaken for gated. Per the same principle the harness clauses draft
(§6.5-0011): an instrument that has never been shown to reject the wrong input
supplies no evidence.

TWO SECTIONS, and they check different layers:

  1. DISCOVERY — `discover_tests` labels a gate correctly (the #140 fix).
  2. REPORT — the coverage report *acts* on that label: it names the GATE-ONLY
     clauses and prints the qualified figure beside the raw one. This is the
     backing for the draft clause §6.1-0009a (RFC derive-and-discriminate).

Section 2 exists because discovery and reporting are separable, and only the
second is what any reader of the coverage number actually sees. `discover_tests`
could label every gate perfectly while the report ignored the labels entirely,
and section 1 would still pass — which is precisely the gap that let §6.1-0009
be recorded as backed by this file when it was not.

Run: python tools/test_kiss_trace_gates.py
"""
import os
import re
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt

FIXTURE = '''
#[cfg(test)]
mod tests {
    #[test]
    fn plain_test_is_not_gated() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn declares_a_runtime_gate_some() {
        // The macro UNWRAPS the Option and yields the inner value, so `m` is the
        // toolchain itself, not a Result. This fixture is scanned and never
        // compiled, but a type-incorrect example is a trap for the next reader.
        let m = crate::runtime_gate_some!("msvc", find_msvc());
        assert!(!m.path.as_os_str().is_empty());
    }

    #[test]
    fn declares_a_runtime_gate_predicate() {
        kiss_conformance::runtime_gate!("cuda", nvcc_present());
        assert!(true);
    }

    #[test]
    fn open_coded_skip_is_the_defect_and_must_not_be_seen_as_declared() {
        let Some(_m) = find_msvc() else { eprintln!("SKIP: no MSVC"); return; };
        assert!(true);
    }

    /// Prose about runtime_gate!("msvc", ..) — this test declares no gate; the
    /// macro name appears only in this doc comment.
    #[test]
    fn comment_mentioning_the_macro_is_not_a_declaration() {
        assert!(true);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn cfg_gated_still_works() {
        assert!(true);
    }
}
'''

failures = []
passed = 0


def check(cond, msg):
    global passed
    if cond:
        passed += 1
        print(f"  ok   {msg}")
    else:
        print(f"  FAIL {msg}")
        failures.append(msg)


# ---- section 2 fixtures: a whole suite, run through the real gate -------------

STEMS = ["umbrella", "announce", "classify", "ops", "grammar", "contract",
         "synth", "consume", "emit", "conform"]

RE_RAW = re.compile(r"ENFORCED \(harness \d+ \+ lint \d+\) = (\d+)/(\d+)")
RE_QUAL = re.compile(r"ENFORCED, EXCLUDING GATE-ONLY = (\d+)/(\d+)")


def fid(ordinal):
    """A fixture clause ID, assembled at run time.

    Never a source literal. `discover_lint_coverage` executes every `kiss_*.py`
    in this directory and harvests clause IDs from its output, and the citation
    scanner reads source for the same grammar — a literal fixture ID risks being
    picked up as a real declaration or reported as a dangling citation against
    the live suite. The `6.99` band is unused by any real document, so even a
    leak cannot collide with a live clause.
    """
    return "-".join(["KISS", "OPS", "6.99", ordinal])


def spec_with(rows):
    """A minimal well-formed `ops.md`: body definitions + the §9 matrix."""
    out = ["## 6.0 Fixture\n\n"]
    for ordinal, test in rows:
        out.append(f"- **{fid(ordinal)}** — A fixture clause. An implementation "
                   f"MUST do the fixture thing. *Test:* `{test}`.\n")
    out.append("\n## 9. Traceability\n\n| Clause | Test |\n|---|---|\n")
    for ordinal, test in rows:
        out.append(f"| {fid(ordinal)} | `{test}` |\n")
    return "".join(out)


def run_report(tmp, rows, harness_src):
    """Build a fixture suite, run the REAL gate over it, return (ok, output)."""
    spec = os.path.join(tmp, "spec")
    conf = os.path.join(tmp, "conformance")
    os.makedirs(spec, exist_ok=True)
    os.makedirs(conf, exist_ok=True)
    for s in STEMS:
        with open(os.path.join(spec, s + ".md"), "w", encoding="utf-8") as f:
            f.write(spec_with(rows) if s == "ops" else "")
    with open(os.path.join(conf, "fixture_tests.rs"), "w", encoding="utf-8") as f:
        f.write(harness_src)
    tool = os.path.join(os.path.dirname(os.path.abspath(__file__)), "kiss_trace.py")
    out = subprocess.run(
        [sys.executable, tool, "--spec-dir", spec, "--conformance-dir", conf],
        capture_output=True, text=True)
    return out.returncode == 0, out.stdout + out.stderr


MIXED_ROWS = [("0042", "test_ops_fixture_cfg_gated"),
              ("0043", "test_ops_fixture_runtime_gated"),
              ("0044", "test_ops_fixture_plain")]

MIXED_HARNESS = '''
#[cfg(feature = "cuda")]
#[test]
fn test_ops_fixture_cfg_gated() { assert!(true); }

#[test]
fn test_ops_fixture_runtime_gated() {
    kiss_conformance::runtime_gate!("cuda", nvcc_present());
    assert!(true);
}

#[test]
fn test_ops_fixture_plain() { assert!(true); }
'''

CLEAN_ROWS = [("0044", "test_ops_fixture_plain")]

CLEAN_HARNESS = '''
#[test]
fn test_ops_fixture_plain() { assert!(true); }
'''


def main():
    with tempfile.TemporaryDirectory() as d:
        conf = os.path.join(d, "conformance")
        os.makedirs(os.path.join(conf, "src"))
        with open(os.path.join(conf, "src", "fixture.rs"), "w", encoding="utf-8") as f:
            f.write(FIXTURE)
        found = kt.discover_tests(conf)

    print("runtime-gate discovery:")
    check("plain_test_is_not_gated" in found, "plain test is discovered at all")
    check(found.get("plain_test_is_not_gated", {}).get("gate") is None,
          "an ungated test is NOT reported as gated (no false positive)")
    check(found.get("declares_a_runtime_gate_some", {}).get("gate") == "runtime:msvc",
          "runtime_gate_some!(\"msvc\", ..) is discovered as runtime:msvc")
    check(found.get("declares_a_runtime_gate_predicate", {}).get("gate") == "runtime:cuda",
          "runtime_gate!(\"cuda\", ..) is discovered as runtime:cuda")
    check(found.get("cfg_gated_still_works", {}).get("gate") == "cuda",
          "the pre-existing cfg-feature gate still works (no regression)")

    # THE negative control: the defect shape itself. An open-coded skip is exactly
    # what #140 is about — it must NOT be silently treated as a declared gate, or
    # the fix would paper over the defect instead of surfacing it.
    check(found.get("open_coded_skip_is_the_defect_and_must_not_be_seen_as_declared",
                    {}).get("gate") is None,
          "an OPEN-CODED skip is not mistaken for a declared gate")

    # Gate discovery reads the test BODY, never the doc comment. Prose naming the
    # macro — this project's own docs do exactly that — must not mark a test gated,
    # or the instrument that detects instrument dishonesty is itself dishonest.
    check(found.get("comment_mentioning_the_macro_is_not_a_declaration",
                    {}).get("gate") is None,
          "a COMMENT naming runtime_gate! does not falsely mark a test as gated")

    # ---- section 2: the REPORT acts on the label (draft §6.1-0009a) ----------
    # Three clauses, each backed by exactly one test: one cfg-gated, one
    # runtime-gated, one unconditional. So GATE-ONLY = 2 and the honest figure
    # is 1/3, not the 3/3 the raw count reports.
    print()
    print("gate-only reporting (draft KISS-CONFORM-6.1-0009a):")
    with tempfile.TemporaryDirectory() as d:
        ok, out = run_report(d, MIXED_ROWS, MIXED_HARNESS)
    check(ok, "the gate accepts the fixture suite (control: it is well-formed)")
    if not ok:
        print(out)
    check("2 of the backed are GATE-ONLY" in out,
          "a clause whose every backing test is gated is reported as GATE-ONLY")
    check("1 cfg-feature gated" in out,
          "the cfg-feature gated clause is broken out by gate kind")
    check("1 runtime gated (cuda=1)" in out,
          "the runtime-gated clause is broken out, naming the declared gate")

    raw, qual = RE_RAW.search(out), RE_QUAL.search(out)
    check(raw is not None and qual is not None,
          "both figures are printed — the raw one never stands alone")
    # The whole point of the split figure is that the two DIFFER by exactly the
    # gated backing. Equal numbers would mean the qualifier is decorative.
    check(bool(raw and qual) and int(qual.group(1)) == int(raw.group(1)) - 2,
          "the qualified figure is lower than the raw one by the gate-only count")

    # THE DISCRIMINATION CONTROL. Without it, a report that unconditionally
    # printed a GATE-ONLY section would satisfy every check above. A suite whose
    # backing all executes has nothing to qualify, and must say nothing.
    with tempfile.TemporaryDirectory() as d:
        ok2, out2 = run_report(d, CLEAN_ROWS, CLEAN_HARNESS)
    check(ok2, "the gate accepts an ungated fixture suite (control)")
    if not ok2:
        print(out2)
    check("GATE-ONLY" not in out2,
          "a suite with NO gated backing reports no GATE-ONLY set (no false positive)")
    check(RE_QUAL.search(out2) is None,
          "and prints no qualified figure — there is nothing to qualify")

    print()
    if failures:
        print(f"FAILED: {len(failures)} check(s), {passed} passed")
        return 1
    # Counted, not asserted: a hardcoded total silently understates the moment a
    # check is added, which is this file's own subject matter.
    print(f"PASS: {passed} checks")
    return 0


if __name__ == "__main__":
    sys.exit(main())
