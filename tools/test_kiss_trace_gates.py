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

Run: python tools/test_kiss_trace_gates.py
"""
import os
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
        let m = crate::runtime_gate_some!("msvc", find_msvc());
        assert!(m.is_ok());
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

    #[cfg(feature = "cuda")]
    #[test]
    fn cfg_gated_still_works() {
        assert!(true);
    }
}
'''

failures = []


def check(cond, msg):
    if cond:
        print(f"  ok   {msg}")
    else:
        print(f"  FAIL {msg}")
        failures.append(msg)


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

    print()
    if failures:
        print(f"FAILED: {len(failures)} check(s)")
        return 1
    print(f"PASS: {6} checks")
    return 0


if __name__ == "__main__":
    sys.exit(main())
