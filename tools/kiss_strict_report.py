#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""
kiss_strict_report.py -- the `strict` job's report, with a pin on what it tolerates.

`kiss_trace.py --strict` (KISS-Conform §6.2) and `kiss_trace.py --freeze-ready` (umbrella
§5.3 condition 3) are red on this tree for a REAL reason: normative MUSTs have no executable
test, and no sub-standard is freeze-ready. That is a finding, not a broken check. Showing it
as a permanently failed job taught everyone to ack it by name; this script reports the numbers
instead and fails only when the red is no longer exactly the known one.

It runs the three instruments, WRITES THE NUMBERS (to $GITHUB_STEP_SUMMARY when set, and to
stdout always), and exits 0 only when ALL of these hold:

  1. `--assert-known-red` exits 0 (the failure is exactly the documented untested-MUST red);
  2. the `--strict` output parses to an untested-MUST count and an ENFORCED a/b figure;
  3. the `--freeze-ready` output parses to the per-sub-standard X/Y rows and the
     "N of M sub-standard(s) satisfy" line.

Anything else exits 1 and says why: a crashed instrument, an unrecognised output shape, an
argparse refusal, or a red that has changed. A tolerated red stays tolerated only while
something pins what it is tolerating (#343); a number that cannot be read is not a number.

`--_broken_invocation` hands `kiss_trace.py` a flag it does not have, so the report MUST fail:
the workflow runs it as a negative control on every run.

`--emit-coverage` returns immediately with nothing (KISS-Trace's lint discovery runs it on every
sibling tool; this one enforces no clause) and the tool refuses to re-enter itself.

Usage:
  python tools/kiss_strict_report.py
  python tools/kiss_strict_report.py --_broken_invocation   # must exit 1
  python tools/kiss_strict_report.py --emit-coverage        # prints nothing, exits 0 at once
"""
import os
import re
import subprocess
import sys

if not __debug__:
    raise SystemExit(
        "refusing to run under -O/PYTHONOPTIMIZE: `assert` is stripped, so these "
        "controls would report success having verified nothing")

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
TRACE = os.path.join(HERE, "kiss_trace.py")

ENFORCED_RE = re.compile(
    r"ENFORCED \(harness (\d+) \+ lint (\d+)\) = (\d+)/(\d+) \(([\d.]+)%\)\. "
    r"Genuinely untested \(incl\. decredited\): (\d+)\.")
SATISFY_RE = re.compile(r"(\d+) of (\d+) sub-standard\(s\) satisfy")
ROW_RE = re.compile(r"\[\s*(FAIL|ok)\s*\]\s+([A-Z]+)\s+(\d+)/(\d+)\s+traced")


def _decode(p):
    return p.returncode, (p.stdout + p.stderr).decode("utf-8", errors="replace")


def _env():
    # The marker is the recursion guard: main() refuses to run the instruments when it is
    # already set, so a child that somehow re-enters this tool cannot spawn kiss_trace again.
    return dict(os.environ, PYTHONIOENCODING="utf-8", KISS_STRICT_REPORT_ACTIVE="1")


# One literal argv per instrument (no argv assembled from variables): the commands are the
# whole attack surface of this module and they are all visible here.
def run_strict():
    return _decode(subprocess.run([sys.executable, TRACE, "--strict"], capture_output=True,
                                  env=_env(), cwd=ROOT))


def run_freeze_ready():
    return _decode(subprocess.run([sys.executable, TRACE, "--freeze-ready"],
                                  capture_output=True, env=_env(), cwd=ROOT))


def run_assert_known_red():
    return _decode(subprocess.run([sys.executable, TRACE, "--assert-known-red"],
                                  capture_output=True, env=_env(), cwd=ROOT))


def run_refused():
    """A kiss_trace.py invocation it refuses (argparse exit 2): the negative control."""
    return _decode(subprocess.run([sys.executable, TRACE, "--strict", "--no-such-flag"],
                                  capture_output=True, env=_env(), cwd=ROOT))


def parse_strict(out):
    """(harness, lint, enforced, total, pct, untested) or None if the shape is unrecognised."""
    m = ENFORCED_RE.search(out)
    if not m:
        return None
    h, l, e, t, pct, u = m.groups()
    return int(h), int(l), int(e), int(t), pct, int(u)


def parse_freeze(out):
    """({SUB: (status, traced, total)}, satisfied, subs) or None if unrecognised."""
    rows = {m.group(2): (m.group(1), int(m.group(3)), int(m.group(4)))
            for m in ROW_RE.finditer(out)}
    s = SATISFY_RE.search(out)
    if not rows or not s:
        return None
    return rows, int(s.group(1)), int(s.group(2))


def render(strict, freeze):
    h, l, e, t, pct, u = strict
    rows, sat, subs = freeze
    lines = [
        "## `strict`: the standard's own completeness, reported (not a pass/fail)",
        "",
        f"- **Untested normative MUSTs:** {u} (genuinely untested, incl. decredited)",
        f"- **Enforced:** {e}/{t} clauses ({pct}%) = harness {h} + lint {l}",
        f"- **Freeze-ready sub-standards (umbrella §5.3 condition 3):** {sat} of {subs}",
        "",
        "| sub-standard | traced | status |",
        "|---|---|---|",
    ]
    for sub in sorted(rows):
        st, a, b = rows[sub]
        lines.append(f"| {sub} | {a}/{b} | {'ready' if st == 'ok' else 'not ready'} |")
    lines += ["", "The red is the documented one (`--assert-known-red`, blocking in the "
              "`kiss-trace` job). This job turns red only if that stops being true or an "
              "instrument fails. A hard-fail on a freeze or a conformance claim is NOT "
              "implemented yet: see ThinkersJournal/KISS#524."]
    return "\n".join(lines)


def decide(rc_strict, out_strict, rc_freeze, out_freeze, rc_known, out_known):
    """(exit_code, markdown_or_reason). Pure, so the controls can feed it every shape."""
    strict = parse_strict(out_strict)
    freeze = parse_freeze(out_freeze)
    if rc_known != 0:
        return 1, ("the red is no longer exactly the KNOWN one: `--assert-known-red` exited "
                   f"{rc_known}. Output tail:\n" + out_known[-1500:])
    if strict is None:
        return 1, ("could not read the untested-MUST figure from `--strict` (exit "
                   f"{rc_strict}); output tail:\n" + out_strict[-1500:])
    if freeze is None:
        return 1, ("could not read the freeze-readiness rows from `--freeze-ready` (exit "
                   f"{rc_freeze}); output tail:\n" + out_freeze[-1500:])
    return 0, render(strict, freeze)


def main(argv):
    # FIRST, before any subprocess: kiss_trace.py runs every sibling `kiss_*.py --emit-coverage`
    # (discover_lint_coverage), and a tool that ignores the flag runs its whole main() there. This
    # one would spawn kiss_trace.py, which would spawn it again (#266). It enforces no clause, so
    # the answer is the empty coverage record: print nothing, exit 0, immediately.
    if "--emit-coverage" in argv:
        return 0
    if os.environ.get("KISS_STRICT_REPORT_ACTIVE"):
        print("refusing to run: kiss_strict_report.py was invoked from inside its own "
              "instruments (recursion)", file=sys.stderr)
        return 1
    broken = "--_broken_invocation" in argv
    rc_s, out_s = run_refused() if broken else run_strict()
    rc_f, out_f = run_refused() if broken else run_freeze_ready()
    rc_k, out_k = run_refused() if broken else run_assert_known_red()
    code, text = decide(rc_s, out_s, rc_f, out_f, rc_k, out_k)
    print(text)
    if broken:
        # The negative control: do not write a FAILED banner into the job summary or emit an
        # error annotation, or every green run would carry a red mark it did not earn.
        return code
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as fh:
            banner = "" if code == 0 else "## `strict` report FAILED" + chr(10) * 2
            fh.write(banner + text + chr(10))
    if code != 0:
        print("::error::strict report failed: see above", file=sys.stderr)
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
