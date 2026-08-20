"""Discrimination controls for the coverage ratchet (`kiss_trace.py --ratchet`).

The ratchet exists because `--strict` has been red since it was written, and a
check that has always been red teaches everyone to ignore it. A ratchet that
could not go red would be the same defect wearing the opposite colour, so what
has to be proven is that it fails in BOTH directions and stays green in between.

Every case builds a synthetic spec + harness, runs the REAL tool over it, and
asserts the outcome. Coverage is moved by ADDING OR REMOVING A TEST — a genuine
coverage change, not an edit to the floor — because a ratchet that only responds
to its own floor file has not been shown to respond to the thing it measures.

  1. AT THE FLOOR        floor == live                      -> green   (CONTROL)
  2. REGRESSION          a backing test disappears          -> red, "REGRESSION"
  3. STALE FLOOR         a new backing test appears         -> red, "STALE"
  4. UNTESTED RISES      a clause loses its only backing     -> red, "REGRESSION"
  5. LINT DIMENSION      lint shortfall masked by a harness
                         surplus — identical ENFORCED sums       -> red   (#187)
  6. INCOMPLETE FLOOR    a key missing from the floor file       -> red

Case 1 is the control: without it, a ratchet hardcoded to fail would pass 2-6.
Case 5 is the one a single ENFORCED number cannot see — floor 2+1 vs live 3+0
both total 3, and only comparing the parts separately catches it.

The subprocess fixtures cannot stage a live lint<->harness move (a fixture clause
can never be lint-covered — lint coverage is discovered from the real `tools/`),
so the substitution / laundering / masked-downgrade cases call `classify_ratchet`
DIRECTLY with a synthetic previous lint set, and a git-fixture drives
`base_ledger_lint` to prove it reads the base by REF not disk. Together they lift
the old limit that a live substitution could not be exercised here.

The `classify_ratchet`-direct block also covers the two RECORDED-MOVE families that
share a regression's count signature and are told from it only by a marking (#261/#267):

  #261 DE-CREDITING  harness -> decredited (an honest downward correction of a false
                     credit). GREEN only when floored AND marked `decredited` on disk;
                     an UNMARKED harness drop stays a regression; self-disposes once
                     recorded at base.
  #267 ARRIVAL       a new clause born WITH its lint detector (-> lint). GREEN only when
                     recorded `lint:<tool>`, declared by --emit-coverage, AND not
                     harness-backed at base. That last is two fail-closed gates: 3a the
                     base FLOOR harness count (catches a harness->lint downgrade absorbed
                     by a floor bump, which `harness_lost` goes blind to — `harness_delta`
                     reads 0) and 3b the base LEDGER+SPEC sets (catches the count-flat
                     residual — a downgrade offset by a promotion — since the ledger domain
                     IS the unbacked set, so in-spec-but-not-in-ledger == was harness). A
                     git-fixture drives `base_floor_harness` to prove it reads base by REF.

Also proven (the #213 review finding, one level in): in a git checkout `--ratchet`
REQUIRES `--base-ref` — a COUNT must not gate the SET comparison, because a
constant-count lint<->harness swap leaves every count at the floor. Only a genuinely
git-less run may skip it, and it must ANNOUNCE the lint dimension went unchecked
rather than print at-the-floor.

Run: python tools/test_kiss_ratchet.py
"""
import os
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace  # noqa: E402 — on sys.path above; classify_ratchet + base_ledger_lint

STEMS = ["umbrella", "announce", "classify", "ops", "grammar", "contract",
         "synth", "consume", "emit", "conform"]
TOOL = os.path.join(os.path.dirname(os.path.abspath(__file__)), "kiss_trace.py")


def fid(n):
    return f"KISS-OPS-6.0-{n}"


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


def harness_with(tests):
    return "\n".join(f"#[test]\nfn {t}() {{ assert_eq!(1 + 1, 2); }}\n" for t in tests)


def run_ratchet(tmp, rows, tests, floor, ledger_rows=()):
    """Build a fixture suite + floor, run the REAL ratchet, return (ok, output)."""
    spec = os.path.join(tmp, "spec")
    conf = os.path.join(tmp, "conformance")
    os.makedirs(spec, exist_ok=True)
    os.makedirs(conf, exist_ok=True)
    for s in STEMS:
        with open(os.path.join(spec, s + ".md"), "w", encoding="utf-8") as f:
            f.write(spec_with(rows) if s == "ops" else "")
    with open(os.path.join(conf, "fixture_tests.rs"), "w", encoding="utf-8") as f:
        f.write(harness_with(tests))
    with open(os.path.join(conf, "UNBACKED.tsv"), "w", encoding="utf-8") as f:
        f.write("# fixture ledger\n")
        for cid, test in ledger_rows:
            f.write(f"{cid}\t{test}\tuntested\t\n")
    with open(os.path.join(conf, "COVERAGE_FLOOR.tsv"), "w", encoding="utf-8") as f:
        f.write("# fixture floor\n")
        for k, v in floor.items():
            f.write(f"{k}\t{v}\n")
    try:
        out = subprocess.run(
            [sys.executable, TOOL, "--ratchet", "--spec-dir", spec,
             "--conformance-dir", conf],
            capture_output=True, text=True, timeout=300)
    except subprocess.TimeoutExpired:
        return False, "TIMEOUT: the ratchet did not finish within 300s"
    return out.returncode == 0, out.stdout + out.stderr


ROWS3 = [("0042", "test_ops_fixture_a"), ("0043", "test_ops_fixture_b"), ("0044", "test_ops_fixture_c")]
ALL3 = ["test_ops_fixture_a", "test_ops_fixture_b", "test_ops_fixture_c"]
FLOOR3 = {"harness": 3, "lint": 0, "untested": 0}

failures = []
ran = []  # every control that executed — asserted against a pinned count so an early
          # return that skips controls cannot report success (a green run and a green run
          # that ran half the controls are otherwise the same exit code). This file is the
          # instrument that proves the instrument, so a silent skip here is the worst place.

EXPECTED_CONTROLS = 44


def check(name, cond, detail=""):
    ran.append(name)
    if not cond:
        failures.append(f"  {name}: {detail}")


def main():
    with tempfile.TemporaryDirectory() as d:
        # 1. CONTROL — floor matches live. Must be GREEN. Without this, a ratchet
        #    hardcoded to fail passes every case below.
        # The fixtures live in a git-less tempdir, so an at-floor run is GREEN but must ANNOUNCE
        # that the lint dimension went unchecked — a plain `at_floor` here is the constant-count
        # swap hole (#213), where a run that looks at-the-floor hides a harness->lint downgrade.
        ok, out = run_ratchet(os.path.join(d, "c1"), ROWS3, ALL3, FLOOR3)
        check("at-the-floor control (git-less)", ok and "at the floor" in out,
              f"expected green 'at the floor', got ok={ok}\n{out[-400:]}")
        check("git-less at-floor announces the unchecked lint dimension",
              "NOT CHARACTERIZED" in out,
              f"a git-less at-floor printed a clean at_floor instead of stating the lint "
              f"dimension went unchecked: {out[-400:]}")

        # 2. REGRESSION — a backing test disappears, so harness falls below floor.
        ok, out = run_ratchet(os.path.join(d, "c2"), ROWS3, ["test_ops_fixture_a", "test_ops_fixture_b"], FLOOR3,
                              ledger_rows=[(fid("0044"), "test_ops_fixture_c")])
        check("regression detected", (not ok) and "RATCHET REGRESSION" in out,
              f"a lost backing test did not fail the ratchet: ok={ok}\n{out[-400:]}")

        # 3. STALE FLOOR — a new backing test appears, so harness rises past floor.
        #    Must ALSO be red: green means AT the floor, never under it.
        ok, out = run_ratchet(os.path.join(d, "c3"), ROWS3, ALL3,
                              {"harness": 2, "lint": 0, "untested": 0},
                              ledger_rows=[])
        check("stale floor detected", (not ok) and "STALE" in out,
              f"coverage past the floor did not fail: ok={ok}\n{out[-400:]}")
        check("stale floor names the new value", "harness 3" in out,
              f"the stale message must say what to set: {out[-400:]}")

        # 4. UNTESTED RISES — same lost test, watched from the other side. The
        #    clause is in the ledger as untested, so `untested` goes 0 -> 1.
        ok, out = run_ratchet(os.path.join(d, "c4"), ROWS3, ["test_ops_fixture_a", "test_ops_fixture_b"],
                              {"harness": 2, "lint": 0, "untested": 0},
                              ledger_rows=[(fid("0044"), "test_ops_fixture_c")])
        check("untested rise detected", (not ok) and "untested" in out,
              f"a rise in untested did not fail: ok={ok}\n{out[-400:]}")

        # 5. LINT DIMENSION WATCHED SEPARATELY — the floor declares a lint clause
        #    that is not live. ENFORCED (harness + lint) would be 3 + 0 = 3 against
        #    a floor of 2 + 1 = 3: IDENTICAL SUMS. Only comparing the parts sees it.
        #    This is #187's substitution shape, measured from the floor side.
        ok, out = run_ratchet(os.path.join(d, "c5"), ROWS3, ALL3,
                              {"harness": 2, "lint": 1, "untested": 0})
        check("lint dimension enforced separately", (not ok) and "lint" in out,
              f"a lint shortfall masked by an equal harness surplus did not fail — "
              f"the ratchet is comparing the SUM: ok={ok} {out[-500:]}")

        # 6. INCOMPLETE FLOOR — a missing key must fail loudly rather than being
        #    treated as "no constraint". A floor with a key silently absent is a
        #    ratchet with that dimension switched off.
        ok, out = run_ratchet(os.path.join(d, "c6"), ROWS3, ALL3,
                              {"harness": 3, "untested": 0})
        check("incomplete floor detected", (not ok) and "incomplete" in out,
              f"a floor missing `lint` did not fail: ok={ok}\n{out[-400:]}")

    # ---- SUBSTITUTION vs LAUNDERING (classify_ratchet, the identity check #213) ----
    # These call the classifier directly so a real lint↔harness movement can be staged —
    # fixture clauses can never be lint-covered (lint coverage is discovered from the real
    # tools/), which is the STATED LIMIT the count-only test above could not get past.
    def cr(floor, live, live_lint, live_harness, prev_lint, disk_lint=None,
           live_decredited=None, prev_decredited=None, disk_decredited=None,
           live_coverage=None, prev_floor_harness=None,
           base_ledger_all=None, base_spec_ids=None):
        def s(x):
            return None if x is None else set(x)
        return kiss_trace.classify_ratchet(
            floor, live, set(live_lint), set(live_harness),
            None if prev_lint is None else set(prev_lint),
            None if disk_lint is None else set(disk_lint),
            live_decredited=s(live_decredited), prev_decredited=s(prev_decredited),
            disk_decredited=s(disk_decredited), live_coverage=s(live_coverage),
            prev_floor_harness=prev_floor_harness,
            base_ledger_all=s(base_ledger_all), base_spec_ids=s(base_spec_ids))

    # clean lint→harness substitution: X leaves lint, arrives in harness. h+1 / l−1 / u flat.
    v, _ = cr({"harness": 2, "lint": 1, "untested": 0}, {"harness": 3, "lint": 0, "untested": 0},
              [], ["X", "A", "B"], ["X"])
    check("substitution recognized", v == "substitution", f"a lint→harness upgrade was not "
          f"recognized as a substitution: got {v}")

    # THE LAUNDERING FLIP (as important as any control here): X: lint→untested and, unrelated,
    # Y: untested→harness. Counts are BYTE-IDENTICAL to the substitution above (h+1 / l−1 / u
    # flat) but it is a regression plus a win. Identity separates them; counts cannot.
    v, lines = cr({"harness": 2, "lint": 1, "untested": 1}, {"harness": 3, "lint": 0, "untested": 1},
                  [], ["Y", "A", "B"], ["X"])
    check("laundering is NOT a substitution", v == "regression",
          f"a regression was laundered as a substitution — the detector swallowed it: got {v}")

    # MASKED DOWNGRADE under a flat count: X: lint→harness AND Z: harness→lint. Every count is
    # flat; Z silently lost its behavioral backing. harness_lost = |{X}| − Δharness(0) = 1.
    v, lines = cr({"harness": 3, "lint": 1, "untested": 0}, {"harness": 3, "lint": 1, "untested": 0},
                  ["Z"], ["X", "A", "B"], ["X"])
    check("masked harness→lint downgrade caught under a flat count", v == "regression",
          f"a downgrade hidden by an equal-count swap passed as clean: got {v}")

    # set-based at-floor: lint set unchanged, counts at floor.
    v, _ = cr({"harness": 2, "lint": 1, "untested": 0}, {"harness": 2, "lint": 1, "untested": 0},
              ["X"], ["A", "B"], ["X"])
    check("set-based at-floor", v == "at_floor", f"got {v}")

    # git-free path (prev_lint=None, lint count unchanged) still discriminates on counts.
    v, _ = cr({"harness": 3, "lint": 0, "untested": 0}, {"harness": 2, "lint": 0, "untested": 0},
              [], ["A", "B"], None)
    check("git-free regression still fires", v == "regression", f"got {v}")

    # COMPLETED SUBSTITUTION (#223): the floor is already bumped to POST (h+N / l−N), so counts
    # are AT the floor while the base ledger still shows the moved clause as lint. The old
    # harness_lost MISFIRES here — Δharness = 0 against the bumped floor, left_to_harness = 1 —
    # and printed "1 disappeared (harness 3 -> 3)", a message that contradicts its parentheses.
    # META: every fixture above models the floor at PRE (the in-progress state). No fixture was
    # ever in the POST state a real substitution PR is in, which is why a broken harness_lost
    # stayed green through all of them. A fixture set that models only the pre-fix state never
    # tests the post — the same population gap as a flip that cannot reach the arm it protects.
    v, _ = cr({"harness": 3, "lint": 0, "untested": 0}, {"harness": 3, "lint": 0, "untested": 0},
              [], ["X", "A", "B"], ["X"], [])
    check("completed substitution (floor bumped, ledger updated) is GREEN",
          v == "substitution_recorded",
          f"a recorded lint→harness substitution was misread once the floor was bumped: got {v}")

    # STALE LEDGER gate: identical, but the on-disk ledger STILL lists X as lint. Green there
    # would fire on every later unrelated PR (the base ledger lists X forever), reporting a
    # substitution commits after it happened. The ledger update must gate the green.
    v, _ = cr({"harness": 3, "lint": 0, "untested": 0}, {"harness": 3, "lint": 0, "untested": 0},
              [], ["X", "A", "B"], ["X"], ["X"])
    check("completed substitution with a STALE ledger is a regression",
          v == "regression",
          f"a green here repeats on every later PR — the ledger update must gate it: got {v}")

    # LEDGER UNREADABLE (#225 review finding): disk_lint=None must NOT degrade to green through
    # `None or set()` — "I could not read the ledger" is not "the ledger is clean", and it would
    # resolve GREEN, the currency hazard arriving through the environment. The input the claim
    # names (#226) is disk_lint=None, and its verdict must be red, not substitution_recorded.
    v, _ = cr({"harness": 3, "lint": 0, "untested": 0}, {"harness": 3, "lint": 0, "untested": 0},
              [], ["X", "A", "B"], ["X"], None)
    check("completed substitution with an UNREADABLE ledger is NOT green",
          v == "ledger_unverifiable",
          f"an unreadable ledger degraded to a green substitution_recorded via `None or set()`: "
          f"got {v}")

    # ---- #261 HONEST DE-CREDITING (harness -> decredited) ----
    # A false credit (a MENTION counted as a backing, #187/#191) corrected downward is
    # harness -N / untested +N — BYTE-IDENTICAL to a regression. The base->disk `decredited`
    # SET is the discriminator: a de-crediting is MARKED, a regression is silent. `decredited`
    # rolls into the untested total but is tracked as its own set. `crd` stages a pure
    # de-crediting (no lint movement, backings A+B intact).
    def crd(floor, live, **kw):
        return cr(floor, live, [], ["A", "B"], [], [], **kw)

    # COMPLETED (floor bumped to POST h-1/u+1, D listed `decredited` on disk) -> GREEN.
    v, _ = crd({"harness": 2, "lint": 0, "untested": 1}, {"harness": 2, "lint": 0, "untested": 1},
               live_decredited=["D"], prev_decredited=[], disk_decredited=["D"])
    check("completed de-crediting (floored + ledger-marked) is GREEN",
          v == "decrediting_recorded", f"an honest de-crediting was not recognized: got {v}")

    # IN-PROGRESS (floor still at PRE) -> `decrediting`, tells you to bump the floor.
    v, _ = crd({"harness": 3, "lint": 0, "untested": 0}, {"harness": 2, "lint": 0, "untested": 1},
               live_decredited=["D"], prev_decredited=[], disk_decredited=["D"])
    check("in-progress de-crediting says bump the floor", v == "decrediting", f"got {v}")

    # THE DISCRIMINATOR FLIP: same counts (h-1/u+1) but NOTHING is marked `decredited` — an
    # accidental loss, not a documented correction. Must stay a REGRESSION. This is the whole
    # point of #261: the honest correction is distinguished from the regression it resembles
    # ONLY by the marking, never by the counts.
    v, _ = crd({"harness": 3, "lint": 0, "untested": 0}, {"harness": 2, "lint": 0, "untested": 1},
               live_decredited=[], prev_decredited=[], disk_decredited=[])
    check("an UNMARKED harness drop is still a regression (de-credit is marked, regression silent)",
          v == "regression", f"a silent harness loss was excused as a de-crediting: got {v}")

    # UNREADABLE ledger at the completed shape -> not green (the currency hazard through the
    # environment, #225), symmetric to the substitution case above.
    v, _ = crd({"harness": 2, "lint": 0, "untested": 1}, {"harness": 2, "lint": 0, "untested": 1},
               live_decredited=["D"], prev_decredited=[], disk_decredited=None)
    check("de-crediting with an UNREADABLE ledger is NOT green", v == "ledger_unverifiable",
          f"an unreadable ledger degraded to a green de-crediting: got {v}")

    # UNRECORDED: the base->disk diff shows a de-credit but the disk ledger does not list it
    # `decredited` -> regression (else it re-fires on every later PR, the #223 shape).
    v, _ = crd({"harness": 2, "lint": 0, "untested": 1}, {"harness": 2, "lint": 0, "untested": 1},
               live_decredited=["D"], prev_decredited=[], disk_decredited=[])
    check("a de-crediting absent from the disk ledger is a regression", v == "regression",
          f"an unrecorded de-credit passed: got {v}")

    # ALREADY de-credited at BASE (prev lists D): not new this PR, counts at floor -> at_floor.
    # Proves the verdict SELF-DISPOSES — green once, at the PR that lands it, then silent.
    v, _ = crd({"harness": 2, "lint": 0, "untested": 1}, {"harness": 2, "lint": 0, "untested": 1},
               live_decredited=["D"], prev_decredited=["D"], disk_decredited=["D"])
    check("a de-crediting already recorded at base self-disposes to at_floor", v == "at_floor",
          f"a settled de-credit re-fired instead of disposing: got {v}")

    # ---- #267 BORN-WITH-DETECTOR ARRIVAL (-> lint) ----
    # A new normative clause and its lint land together: the clause arrives in the lint set with
    # the floor's lint bumped. Count shape is IDENTICAL to lint_drift and to a silent harness->doc
    # downgrade, so green only when genuine: (1) recorded `lint:<tool>` on disk, (2) declared by
    # --emit-coverage, (3) not harness-backed at base — checked by TWO fail-closed gates: 3a the
    # base FLOOR harness count (cheap first filter) and 3b the base LEDGER+SPEC sets (the deciding
    # check: present in the base spec but absent from the base ledger == was harness-backed).
    # `cra` stages an arrival (`live_lint` both arrives — prev_lint empty — and is on disk) and
    # DEFAULTS the base sets to the brand-new case (absent from the base spec -> no downgrade),
    # overridable per test.
    def cra(floor, live, live_lint, **kw):
        kw.setdefault("base_ledger_all", [])
        kw.setdefault("base_spec_ids", [])
        return cr(floor, live, live_lint, ["A", "B"], [], live_lint, **kw)

    # LEGIT arrival, BRAND-NEW clause N (absent from base spec), floor lint bumped, harness flat.
    v, _ = cra({"harness": 2, "lint": 1, "untested": 0}, {"harness": 2, "lint": 1, "untested": 0},
               ["N"], live_coverage=["N"], prev_floor_harness=2)
    check("born-with-detector arrival (brand-new clause) is GREEN",
          v == "arrival_recorded", f"a clause born with its detector was not recognized: got {v}")

    # LEGIT arrival, UNTESTED->LINT UPGRADE: X pre-existed (in base spec) AND was in the base
    # ledger (unbacked at base), so it was NOT harness-backed -> a fine new documentary
    # enforcement, GREEN. This is the case condition 3b must ADMIT, not just the downgrade it must
    # reject — a base-spec membership check alone would wrongly red this.
    v, _ = cra({"harness": 2, "lint": 1, "untested": 0}, {"harness": 2, "lint": 1, "untested": 0},
               ["X"], live_coverage=["X"], prev_floor_harness=2,
               base_spec_ids=["X"], base_ledger_all=["X"])
    check("an untested->lint upgrade (in base spec AND base ledger) is GREEN",
          v == "arrival_recorded", f"a legitimate untested->lint upgrade was rejected: got {v}")

    # CONDITION 3a FLIP (base FLOOR harness): a harness->lint downgrade (Z) ABSORBED by a floor
    # bump. base floor harness 3, live harness 2. `harness_lost` is BLIND (harness_delta reads 0);
    # 3a catches the drop. base sets pass 3b (both "new") so 3a is isolated as the blocker.
    v, _ = cra({"harness": 2, "lint": 2, "untested": 0}, {"harness": 2, "lint": 2, "untested": 0},
               ["Z", "N"], live_coverage=["Z", "N"], prev_floor_harness=3)
    check("a harness->lint downgrade absorbed by a floor bump is caught by the base-floor gate",
          v == "lint_drift",
          f"a laundered downgrade passed 3a — the silent condition-3 hole: got {v}")

    # CONDITION 3b FLIP (THE RESIDUAL, now CLOSED): a harness->lint downgrade (Z) OFFSET by an
    # untested->harness promotion (M) that holds the harness COUNT flat, so 3a passes (base floor
    # harness 3 == live harness 3). Only the base LEDGER+SPEC sets see it: Z is in the base spec
    # but absent from the base ledger (== was harness-backed) -> a downgrade. Must be RED. This is
    # the case that fooled both lanes; per the #267 review it goes red rather than documented.
    v, _ = cr({"harness": 3, "lint": 2, "untested": 0}, {"harness": 3, "lint": 2, "untested": 0},
              ["Z", "N"], ["A", "B", "M"], [], ["Z", "N"], live_coverage=["Z", "N"],
              prev_floor_harness=3, base_spec_ids=["Z"], base_ledger_all=["M"])
    check("a count-flat downgrade offset by a promotion is caught by the base ledger+spec sets",
          v == "lint_drift",
          f"the count-flat residual passed as an arrival — 3b did not close it: got {v}")

    # CONDITION 1 FLIP: the arrival is not recorded `lint:<tool>` on disk -> lint_drift (raw cr,
    # disk_lint empty; base sets pass so condition 1 is isolated).
    v, _ = cr({"harness": 2, "lint": 1, "untested": 0}, {"harness": 2, "lint": 1, "untested": 0},
              ["N"], ["A", "B"], [], [], live_coverage=["N"], prev_floor_harness=2,
              base_ledger_all=[], base_spec_ids=[])
    check("an arrival absent from the disk ledger is not green", v == "lint_drift", f"got {v}")

    # CONDITION 2 FLIP (the fixture-breakable seam that proves condition 2 is a LIVE gate, not a
    # tautology of the caller's construction): recorded on disk but NOT declared by any tool's
    # --emit-coverage -> lint_drift.
    v, _ = cra({"harness": 2, "lint": 1, "untested": 0}, {"harness": 2, "lint": 1, "untested": 0},
               ["N"], live_coverage=[], prev_floor_harness=2)
    check("an arrival not declared by --emit-coverage is not green", v == "lint_drift", f"got {v}")

    # CONDITION 3a UNCERTIFIABLE: the base FLOOR could not be read -> decline the green rather than
    # pass an unchecked gate (the environment-degradation refusal, #225).
    v, _ = cra({"harness": 2, "lint": 1, "untested": 0}, {"harness": 2, "lint": 1, "untested": 0},
               ["N"], live_coverage=["N"], prev_floor_harness=None)
    check("an arrival with an unreadable base floor declines the green", v == "lint_drift",
          f"an uncertifiable condition 3a passed as green: got {v}")

    # CONDITION 3b UNCERTIFIABLE: the base LEDGER/SPEC sets could not be read (floor fine) ->
    # decline the green. Same fail-closed discipline; a base read that degrades to None must never
    # become a silent pass.
    v, _ = cra({"harness": 2, "lint": 1, "untested": 0}, {"harness": 2, "lint": 1, "untested": 0},
               ["N"], live_coverage=["N"], prev_floor_harness=2,
               base_ledger_all=None, base_spec_ids=None)
    check("an arrival with unreadable base ledger/spec sets declines the green", v == "lint_drift",
          f"an uncertifiable condition 3b passed as green: got {v}")

    # UNREADABLE ledger at the arrival shape -> ledger_unverifiable (symmetric to the others).
    v, _ = cr({"harness": 2, "lint": 1, "untested": 0}, {"harness": 2, "lint": 1, "untested": 0},
              ["N"], ["A", "B"], [], None, live_coverage=["N"], prev_floor_harness=2,
              base_ledger_all=[], base_spec_ids=[])
    check("an arrival with an UNREADABLE ledger is not green", v == "ledger_unverifiable",
          f"got {v}")

    # ---- CURRENCY HAZARD (base_ledger_lint reads the REF, not the disk, #213) ----
    with tempfile.TemporaryDirectory() as g:
        conf = os.path.join(g, "conformance")
        os.makedirs(conf)
        ledger = os.path.join(conf, "UNBACKED.tsv")
        floor = os.path.join(conf, "COVERAGE_FLOOR.tsv")

        def git(*a):
            subprocess.run(["git", "-C", g, *a], check=True, capture_output=True, text=True)
        git("init", "-q")
        git("config", "user.email", "t@t")
        git("config", "user.name", "t")
        # BASE (committed) state: X is lint; floor harness 3.
        with open(ledger, "w", encoding="utf-8") as f:
            f.write("# ledger\nKISS-OPS-6.0-0042\ttest_x\tlint:kiss_ops\tnote\n")
        with open(floor, "w", encoding="utf-8") as f:
            f.write("# floor\nharness\t3\nlint\t1\nuntested\t0\n")
        git("add", "-A")
        git("commit", "-q", "-m", "base")
        # NEW state ON DISK: X removed (moved to harness), floor harness bumped DOWN to 2. NOT
        # committed — precisely the regenerate-then-run order that silences a disk-reading check.
        with open(ledger, "w", encoding="utf-8") as f:
            f.write("# ledger\n")
        with open(floor, "w", encoding="utf-8") as f:
            f.write("# floor\nharness\t2\nlint\t1\nuntested\t0\n")
        base = kiss_trace.base_ledger_lint(ledger, "HEAD")
        check("base ledger is read from the REF, not the regenerated disk file",
              base == {"KISS-OPS-6.0-0042"},
              f"read the on-disk (already-new) ledger instead of the base ref: got {base}")
        # An indeterminable base MUST be None so the caller fails loud — never an empty diff,
        # which is the same silent pass reached through the environment.
        bad = kiss_trace.base_ledger_lint(ledger, "no-such-ref-xyz")
        check("indeterminable base returns None (fail-loud, not an empty diff)", bad is None,
              f"a missing ref degraded to a set instead of None: got {bad}")
        # The #267 condition-3 gate reads the base FLOOR by REF, same hazard: the on-disk floor
        # already shows harness 2, but HEAD holds 3. A disk read would report 2 and let a harness
        # drop pass as an arrival; the ref read must return 3.
        fh = kiss_trace.base_floor_harness(floor, "HEAD")
        check("base floor harness is read from the REF, not the regenerated disk file", fh == 3,
              f"read the on-disk (already-bumped) floor instead of the base ref: got {fh}")
        badf = kiss_trace.base_floor_harness(floor, "no-such-ref-xyz")
        check("indeterminable base floor returns None (declines the arrival green, not passes it)",
              badf is None, f"a missing ref degraded to an int instead of None: got {badf}")

    # ---- THE REVIEW FINDING (#213, one level in): a COUNT must not gate the SET check ----
    # A constant-count lint↔harness swap leaves every count at the floor, so a count gate on
    # the base read lets it pass as at_floor. Fix: in a git checkout --ratchet REQUIRES
    # --base-ref (unconditionally, not gated on the lint count); only a genuinely git-less run
    # may skip it, and it must SAY the lint dimension went unchecked, never print at_floor.
    with tempfile.TemporaryDirectory() as gd:
        gspec = os.path.join(gd, "spec")
        gconf = os.path.join(gd, "conformance")
        os.makedirs(gspec)
        os.makedirs(gconf)
        for s in STEMS:
            with open(os.path.join(gspec, s + ".md"), "w", encoding="utf-8") as f:
                f.write(spec_with(ROWS3) if s == "ops" else "")
        with open(os.path.join(gconf, "fixture_tests.rs"), "w", encoding="utf-8") as f:
            f.write(harness_with(ALL3))
        with open(os.path.join(gconf, "UNBACKED.tsv"), "w", encoding="utf-8") as f:
            f.write("# ledger\n")
        with open(os.path.join(gconf, "COVERAGE_FLOOR.tsv"), "w", encoding="utf-8") as f:
            f.write("# floor\nharness\t3\nlint\t0\nuntested\t0\n")

        def gg(*a):
            subprocess.run(["git", "-C", gd, *a], check=True, capture_output=True, text=True)
        gg("init", "-q")
        gg("config", "user.email", "t@t")
        gg("config", "user.name", "t")
        gg("add", "-A")
        gg("commit", "-q", "-m", "base")
        r = subprocess.run([sys.executable, TOOL, "--ratchet", "--spec-dir", gspec,
                            "--conformance-dir", gconf], capture_output=True, text=True, timeout=300)
        gout = r.stdout + r.stderr
        check("git checkout --ratchet REQUIRES --base-ref (a count must not gate the set check)",
              r.returncode != 0 and "base-ref" in gout,
              f"a git-checkout --ratchet without --base-ref passed at-the-floor — the constant-"
              f"count swap hole: rc={r.returncode}\n{gout[-400:]}")

        # A REFUSAL IS NOT A VIOLATION. The check above only asks that the refusal is
        # non-zero, which a single failure state satisfies -- so it cannot tell "I declined
        # to measure" from "the floor moved". Reporting the decline as VIOLATIONS FOUND
        # spends the word that must keep meaning a floor breach, and this file's own
        # rationale is that an always-red check teaches everyone to ignore it. CI always
        # passes --base-ref, so the false alarm lands only on a human spot-checking by hand
        # -- the reader least able to tell it is spurious.
        check("--ratchet without --base-ref reports INCONCLUSIVE, not a floor violation",
              r.returncode == 2 and "INCONCLUSIVE" in gout and "VIOLATIONS FOUND" not in gout,
              f"a usage refusal must be its own exit state (2), distinct from a floor breach "
              f"(1) and from clean (0): rc={r.returncode}\n{gout[-400:]}")

        # THE CONTROL THAT MAKES THE ONE ABOVE SAFE. Without it, "always report INCONCLUSIVE
        # when --base-ref is missing" passes -- and that would turn every real regression run
        # without a base into a soft non-answer, strictly worse than the false alarm it
        # replaced. A genuine violation MUST outrank a refusal.
        with open(os.path.join(gconf, "COVERAGE_FLOOR.tsv"), "w", encoding="utf-8") as f:
            f.write("# floor\nharness\t2\nlint\t0\nuntested\t0\n")
        rv = subprocess.run([sys.executable, TOOL, "--ratchet", "--spec-dir", gspec,
                             "--conformance-dir", gconf], capture_output=True, text=True,
                            timeout=300)
        vout = rv.stdout + rv.stderr
        check("a real floor breach OUTRANKS the missing-base refusal",
              rv.returncode == 1 and "VIOLATIONS FOUND" in vout,
              f"floor 2 vs live 3 is a genuine breach and must report VIOLATIONS FOUND even "
              f"with --base-ref absent, or INCONCLUSIVE becomes a way to mask one: "
              f"rc={rv.returncode}\n{vout[-400:]}")

    # ---- STALE BASE, END TO END (#276 review) ------------------------------------------
    # THE DETECTOR WAS CONTROLLED; THE WIRING WAS NOT. The four controls below call
    # `base_is_current` directly, so dropping `inconclusive = True` at the call site left
    # every one of them GREEN while the tool returned exit 0 / RESULT: CLEAN on a stale
    # tree -- the whole feature silently disabled, which is the exact class #276 exists to
    # eliminate. A unit control on a predicate says nothing about whether anyone consults it.
    with tempfile.TemporaryDirectory() as ed:
        espec = os.path.join(ed, "spec")
        econf = os.path.join(ed, "conformance")
        os.makedirs(espec)
        os.makedirs(econf)
        for st in STEMS:
            with open(os.path.join(espec, st + ".md"), "w", encoding="utf-8") as f:
                f.write(spec_with(ROWS3) if st == "ops" else "")
        with open(os.path.join(econf, "fixture_tests.rs"), "w", encoding="utf-8") as f:
            f.write(harness_with(ALL3))
        with open(os.path.join(econf, "UNBACKED.tsv"), "w", encoding="utf-8") as f:
            f.write("# ledger\n")

        def eg(*a):
            subprocess.run(["git", "-C", ed, *a], check=True, capture_output=True, text=True)

        def at_floor(h):
            with open(os.path.join(econf, "COVERAGE_FLOOR.tsv"), "w", encoding="utf-8") as f:
                f.write(f"# floor\nharness\t{h}\nlint\t0\nuntested\t0\n")

        at_floor(3)
        eg("init", "-q")
        eg("config", "user.email", "t@t")
        eg("config", "user.name", "t")
        eg("add", "-A")
        eg("commit", "-q", "-m", "base")
        eg("branch", "work")
        eg("commit", "-q", "--allow-empty", "-m", "one")
        eg("commit", "-q", "--allow-empty", "-m", "two")
        eg("branch", "-f", "moved")
        eg("checkout", "-q", "work")

        def run_against_moved():
            r = subprocess.run([sys.executable, TOOL, "--ratchet", "--spec-dir", espec,
                                "--conformance-dir", econf, "--base-ref", "moved"],
                               capture_output=True, text=True, timeout=300)
            return r.returncode, r.stdout + r.stderr

        rc, out = run_against_moved()
        check("END TO END: a moved base yields exit 2, not CLEAN",
              rc == 2 and "NOT an ancestor" in out and "RESULT: CLEAN" not in out,
              f"the tool must consult the staleness predicate, not merely have one: "
              f"rc={rc} (want 2)\n{out[-500:]}")

        # THE PROPERTY RANKED FIRST AND PREVIOUSLY UNCONTROLLED ON THIS PATH. A refusal must
        # never mask a real breach -- and the stale warning must still print alongside it,
        # or the reader fixes the breach against a base that is still wrong.
        at_floor(99)
        rc, out = run_against_moved()
        check("END TO END: a real breach on a stale tree still exits 1, warning retained",
              rc == 1 and "NOT an ancestor" in out,
              f"a breach must OUTRANK staleness and the staleness must still be reported: "
              f"rc={rc} (want 1)\n{out[-500:]}")

    # ---- STALE BASE (#276) -------------------------------------------------------------
    # The ratchet compares the branch's FLOOR against the branch's LIVE figures. Those can
    # agree with each other while BOTH disagree with the base, so a branch that has sat
    # while main moved reports CLEAN -- correctly, about a tree nobody merges into. It went
    # green that way four times in one session. `base_is_current` asks the one predicate the
    # comparison cannot: is the base still an ancestor of what I am measuring?
    with tempfile.TemporaryDirectory() as sd:
        conf = os.path.join(sd, "conformance")
        os.makedirs(conf)
        ledger = os.path.join(conf, "UNBACKED.tsv")
        with open(ledger, "w", encoding="utf-8") as f:
            f.write("# ledger\n")

        def sg(*a):
            subprocess.run(["git", "-C", sd, *a], check=True, capture_output=True, text=True)
        sg("init", "-q")
        sg("config", "user.email", "t@t")
        sg("config", "user.name", "t")
        sg("add", "-A")
        sg("commit", "-q", "-m", "base")
        sg("branch", "basepoint")
        # advance the BASE two commits past the working head
        sg("commit", "-q", "--allow-empty", "-m", "one")
        sg("commit", "-q", "--allow-empty", "-m", "two")
        sg("branch", "-f", "movedbase")
        sg("checkout", "-q", "basepoint")

        # Called ONCE and cached: two calls inside one control repeat the git subprocesses
        # and, if they ever disagreed, would make the failure unreadable -- a small version
        # of the same problem this feature exists to catch.
        moved = kiss_trace.base_is_current(ledger, "movedbase")
        check("a base that has moved ahead is reported STALE, with the distance",
              moved == 2,
              f"expected distance 2, got {moved!r}")

        # THE PAIRED CONTROL. Without it, "always report stale" passes the case above --
        # which would turn every correct run into a refusal and be strictly worse than the
        # silence it replaced.
        check("a base that IS an ancestor is not reported stale",
              kiss_trace.base_is_current(ledger, "basepoint") is True,
              f"an up-to-date base must be True, got {kiss_trace.base_is_current(ledger, 'basepoint')!r}")

        check("an unknown ref is UNKNOWABLE (None), never a staleness claim",
              kiss_trace.base_is_current(ledger, "no-such-ref-xyz") is None,
              "an unresolvable ref must not be reported as a stale base -- the base-ledger "
              "read fails on the same condition and already reports its own refusal")

    with tempfile.TemporaryDirectory() as nd:
        conf = os.path.join(nd, "conformance")
        os.makedirs(conf)
        ledger = os.path.join(conf, "UNBACKED.tsv")
        with open(ledger, "w", encoding="utf-8") as f:
            f.write("# ledger\n")
        check("a git-less tree is UNKNOWABLE, not stale",
              kiss_trace.base_is_current(ledger, "origin/main") is None,
              "outside a repo there is no ancestry to read; claiming staleness would be "
              "an assertion from an absent measurement")

    if failures:
        print("FAIL - the coverage ratchet does not discriminate:")
        print("\n".join(failures))
        return 1
    print("ok - ratchet is green at the floor and red in BOTH directions, tells a")
    print("     substitution from a laundered regression by IDENTITY, catches a masked")
    print("     downgrade under a flat count, and reads the base ledger by ref not disk")
    return 0


def test_kiss_ratchet_discrimination():
    """Collected by pytest; CI also runs the file in script mode.

    Without this, `pytest tools/` collects ZERO tests from a `test_*.py` file and
    reports success having run none of the controls — the #158 shape, which is
    gated in CI and has already caught one omission in this session.
    """
    ran.clear()
    assert main() == 0, "the coverage ratchet failed its discrimination controls"
    # POPULATION: a green run and a green run that skipped half the controls are the
    # same exit code, so pin the count. An early return anywhere in main() leaves
    # len(ran) short and reddens here — checked AFTER main() so a mid-function return
    # cannot skip the check itself.
    assert len(ran) == EXPECTED_CONTROLS, (
        f"main() returned 0 but only {len(ran)}/{EXPECTED_CONTROLS} controls ran — an early "
        f"return skipped some, and a suite that skips controls must not report success")


if __name__ == "__main__":
    rc = main()
    if len(ran) != EXPECTED_CONTROLS:
        print(f"FAIL - only {len(ran)}/{EXPECTED_CONTROLS} controls ran (an early return "
              f"skipped some); a suite that skips controls must not exit 0")
        rc = 1
    sys.exit(rc)
