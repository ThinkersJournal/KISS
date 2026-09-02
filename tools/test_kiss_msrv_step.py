"""Behavioural controls for the MSRV leg (#369).

`rust-version = "1.77"` was DECLARED and never MEASURED. Nothing had ever built at it, so
the number was unverified in BOTH directions -- it could be too low (the crate needs more)
or too high (a false promise excluding consumers who could have compiled).

    THE LEG THAT PROVES NECESSITY IS THE ONE MOST EASILY SATISFIED BY NOTHING HAPPENING.

A minimality check says "one minor below must FAIL". Four different things produce a
non-zero exit, and only one of them is evidence:

    error[E0658]: ... is unstable              <- the ONLY acceptable failure
    error: failed to load manifest             <- edition floor
    error: toolchain '1.76' is not installed   <- rustup never got it
    error: rustc 1.76.0 is not supported by    <- the CIRCULARITY, if --ignore-rust-version
                                                  is ever dropped: the declared number
                                                  causes the failure that justifies it

A leg accepting any non-zero exit reports all four as "the floor is necessary" -- and three
of those are the check failing to run, dressed as a finding.

These controls EXTRACT the step's `run:` block from the workflow and EXECUTE it under
`bash -e` with `cargo` and `rustup` replaced by shell functions whose exit codes and output
each case chooses. What is asserted is the STEP'S EXIT CODE AND ANNOTATION -- not the
presence of words in a YAML file. `sed` is left real, because reading the floor out of
Cargo.toml is part of what is being tested.

EXIT CODES ARE THE VERDICT VOCABULARY, and they must stay distinct:

    0  the declaration is CORRECT
    1  a FINDING about the floor (too low, or too high)
    3  a LEG ERROR -- the check could not run; NOT a statement about the floor

Collapsing 1 and 3 is the defect this file exists to prevent: it converts "I could not
measure" into "I measured, and the floor is necessary".

Run: python tools/test_kiss_msrv_step.py
"""
import os
import pathlib
import re
import subprocess
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
WF = HERE.parent / ".github" / "workflows" / "conformance.yml"
STEP = "The declared rust-version is SUFFICIENT and NECESSARY (#369)"
RE_EXPR = re.compile(r"\$\{\{\s*([^}]+?)\s*\}\}")

# `rustup` always succeeds; `cargo` dispatches on what it was asked to do. Functions rather
# than PATH stubs so the behaviour is identical on a runner and on Git Bash.
PRELUDE = """
rustup() { return 0; }
cargo() {
  case " $* " in
    *" --version "*)            echo "cargo 0.0.0"; return ${STUB_ID_RC:-0} ;;
    *" --ignore-rust-version "*) printf '%s\\n' "$STUB_MIN_OUT"; return ${STUB_MIN_RC:-1} ;;
    *)                          return ${STUB_SUF_RC:-0} ;;
  esac
}
"""

E0658 = "error[E0658]: `round_ties_even` is unstable"


def run_block(text=None, step=STEP):
    """The step's `run:` script, dedented. Raises if the step or its block is missing."""
    lines = (text if text is not None else WF.read_text(encoding="utf-8")).splitlines()
    i = next((n for n, l in enumerate(lines) if l.strip() == "- name: " + step), None)
    if i is None:
        raise AssertionError("step %r not found in %s" % (step, WF))
    j = next((n for n in range(i + 1, len(lines)) if lines[n].strip().startswith("run:")), None)
    if j is None or not lines[j].strip().endswith("|"):
        raise AssertionError("step %r has no block `run: |`" % step)
    indent = len(lines[j]) - len(lines[j].lstrip())
    body = []
    for l in lines[j + 1:]:
        if l.strip() and (len(l) - len(l.lstrip())) <= indent:
            break
        body.append(l)
    first = next((l for l in body if l.strip()), None)
    if first is None:
        raise AssertionError("step %r has an EMPTY `run:` block" % step)
    off = len(first) - len(first.lstrip())
    return "\n".join(l[off:] if len(l) > off else "" for l in body)


def execute(floor='rust-version = "1.77"', id_rc=0, suf_rc=0, min_rc=1, min_out=E0658):
    """Run the real block against a fixture Cargo.toml; returns (exit code, output)."""
    script = RE_EXPR.sub(lambda m: "", run_block())
    with tempfile.TemporaryDirectory() as td:
        d = pathlib.Path(td)
        (d / "Cargo.toml").write_text(
            '[package]\nname = "fixture"\n%s\n' % floor, encoding="utf-8")
        sh = d / "step.sh"
        sh.write_text(PRELUDE + script + "\n", encoding="utf-8")
        env = dict(os.environ)
        env.update({"STUB_ID_RC": str(id_rc), "STUB_SUF_RC": str(suf_rc),
                    "STUB_MIN_RC": str(min_rc), "STUB_MIN_OUT": min_out})
        r = subprocess.run(["bash", "--noprofile", "--norc", "-e", "-o", "pipefail",
                            sh.as_posix()],
                           capture_output=True, text=True, timeout=120,
                           cwd=d.as_posix(), env=env)
    return r.returncode, r.stdout + r.stderr


class MsrvStepTest(unittest.TestCase):
    def test_the_block_carries_no_actions_expression(self):
        """Guards the harness. A `${{ ... }}` would be substituted to empty here and tested
        as something CI never runs."""
        self.assertEqual(sorted(set(RE_EXPR.findall(run_block()))), [])

    def test_the_declaration_being_correct_exits_zero(self):
        """Control. Without it a block that exited non-zero unconditionally would satisfy
        every failing case below."""
        rc, out = execute()
        self.assertEqual(rc, 0, out)
        self.assertIn("the declaration is CORRECT", out)

    def test_the_floor_is_READ_from_the_manifest_not_hardcoded(self):
        """A literal would be a SECOND COPY of the declaration, free to drift from the thing
        it is meant to test — and the drift is silent, because a stale copy still builds.

        Driven by giving the fixture a DIFFERENT floor and watching the probe follow it.
        Asserting the absence of `1.77` in the text would not prove the value is used.
        """
        rc, out = execute(floor='rust-version = "1.82"')
        self.assertEqual(rc, 0, out)
        self.assertIn("declared floor 1.82 ; probing 1.81", out,
                      "the probe did not follow the manifest — the floor is hardcoded")

    def test_a_missing_rust_version_is_a_LEG_ERROR(self):
        rc, out = execute(floor="# no rust-version here")
        self.assertEqual(rc, 3, out)
        self.assertIn("no rust-version", out)

    def test_a_toolchain_that_cannot_identify_itself_is_a_LEG_ERROR_not_a_finding(self):
        """The failure mode that makes a necessity check satisfiable by nothing happening.

        If `rustup` never got the toolchain, "the floor is real" and "the check never ran"
        produce the same red. Exit 3, and the annotation says NOT a finding.
        """
        rc, out = execute(id_rc=1)
        self.assertEqual(rc, 3, "a missing toolchain must not be reported as a floor result:\n" + out)
        self.assertIn("did not identify itself", out)
        self.assertIn("NOT a finding", out)

    def test_a_floor_that_does_not_build_is_a_FINDING_too_low(self):
        """Row 2 of #369's table. The predictable misreading is "my new check is broken",
        followed by bumping the floor to whatever passes — which RATIFIES an unmeasured
        number instead of measuring it. The annotation says so at the point of failure."""
        rc, out = execute(suf_rc=1)
        self.assertEqual(rc, 1, out)
        self.assertIn("floor TOO LOW", out)
        self.assertIn("do not bump to whatever passes", out)

    def test_a_floor_one_minor_ABOVE_what_is_needed_is_a_FINDING_too_high(self):
        """Row 3. The only direction the sufficiency leg cannot see: losing the last
        construct that required the floor and nobody noticing the number is now a false
        promise."""
        rc, out = execute(min_rc=0)
        self.assertEqual(rc, 1, out)
        self.assertIn("floor TOO HIGH", out)

    def test_a_NON_E0658_failure_is_a_LEG_ERROR_not_evidence(self):
        """THE CENTRAL CONTROL. Three of the four ways to exit non-zero are the check
        failing to run, and a leg accepting any non-zero exit reports all of them as
        "the floor is necessary"."""
        for out_text, why in [
                ("error: failed to load manifest for workspace", "edition floor"),
                ("error: toolchain '1.76' is not installed", "rustup"),
                ("error: rustc 1.76.0 is not supported by the following package",
                 "the circularity"),
        ]:
            with self.subTest(why=why):
                rc, out = execute(min_rc=1, min_out=out_text)
                self.assertEqual(rc, 3, "%s was accepted as evidence the floor is "
                                        "necessary:\n%s" % (why, out))
                self.assertIn("NOT with error[E0658]", out)

    def test_the_minimality_probe_passes_ignore_rust_version(self):
        """Without the flag the check is CIRCULAR: cargo refuses the build BECAUSE of the
        declared number, so the leg goes red for every crate on every repo whether or not
        the floor is real — a check that cannot return the interesting answer.

        Asserted on the extracted script rather than by behaviour, because the stub cannot
        reproduce cargo's own refusal; the companion is the circularity case above, which
        proves that refusal is rejected if it does arrive.
        """
        # ASSERT THE INVOCATION LINE, NOT THE BLOCK. The first version was
        # `assertIn("--ignore-rust-version", run_block())`, which matches the COMMENT
        # explaining why the flag is mandatory — so it passed with the flag deleted from
        # the actual cargo call. Mutation-proven vacuous, then fixed: the seeded removal
        # reddened three OTHER controls and never this one.
        calls = [l for l in run_block().splitlines()
                 if l.lstrip().startswith(("OUT=$(cargo", "cargo ", "if ! cargo"))
                 and "--no-run" in l]
        self.assertEqual(len(calls), 2, "expected two cargo build invocations: %r" % calls)
        probe = [l for l in calls if "$BELOW" in l]
        self.assertEqual(len(probe), 1, "no minimality probe found: %r" % calls)
        self.assertIn("--ignore-rust-version", probe[0],
                      "the minimality probe lost the flag — the check is CIRCULAR: cargo "
                      "refuses the build because of the declared number itself, so the leg "
                      "reds for every crate on every repo regardless of the floor")

    def test_both_legs_compile_the_SAME_surface_and_it_includes_the_tests(self):
        """The target-selection ruling, pinned because its reasoning is not local.

        `cargo build` never compiles `conformance/tests` — 20,378 of 31,197 lines. The
        SUFFICIENCY leg is the one that breaks: it goes green having never compiled 65% of
        the crate, and green is the direction nobody questions.

        The ruling is NOT the line ratio, though. It is whether `rust-version` promises what
        a CONSUMER needs to compile the crate as a dependency, or what someone needs to RUN
        the harness. For KISS: `publish = false`, zero dependents machine-wide, and the
        README documents the crate only as `cargo test` — so the tests are inside the
        promise. vulkane faces the identical question and `build` is correct there, which is
        why a later reader must not "simplify" this to match another repo.

        And BOTH legs must use the same surface, or the pair measures two different crates
        and calls it a pair.
        """
        block = run_block()
        builds = re.findall(r"cargo \"\+\$[A-Z]+\" (\S+)(?: --no-run)?", block)
        self.assertNotIn("build", builds,
                         "a leg uses `cargo build`, which never compiles conformance/tests")
        self.assertEqual(block.count("test --no-run"), 2,
                         "the two legs do not compile the same surface: %r" % builds)

    def test_the_harness_can_fail(self):
        """The extractor must raise on a missing step rather than silently testing nothing —
        a renamed step would otherwise turn every control above into a vacuous pass."""
        with self.assertRaises(AssertionError):
            run_block(step="No Such Step")
        with self.assertRaises(AssertionError):
            run_block(text="jobs:\n  x:\n    steps:\n      - name: %s\n        run: echo hi\n" % STEP)


if __name__ == "__main__":
    unittest.main()
