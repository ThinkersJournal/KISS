#!/usr/bin/env python3
"""
kiss_bundle.py — assemble the KISS spec suite into paste-into-an-LLM audit bundles.

The KISS suite is ten markdown documents (~1 MB, ~860 normative clauses). To have
an external LLM audit, critique, and suggest improvements to the standard, you
need the text in ONE artifact — but a naive `cat spec/*.md` fails an auditor two
ways: it loses the file boundaries (every document is itself full of `#`/`##`
headings, some with mid-document H1s, so a heading is NOT a reliable file
delimiter), and it drops the context an auditor needs to critique a suite of
interdependent standards (the dependency DAG, the clause-ID/traceability
convention, and which normative MUSTs actually have a conformance test).

This tool fixes both. It wraps each document in an XML-style
`<document path="...">` tag (a boundary a model parses unambiguously and never
confuses with markdown), orders the documents topologically (foundational
vocabularies first), and prepends an <audit-brief> that tells the model what KISS
is, how to read the DAG, the normative conventions, the LIVE traceability status
(reused from tools/kiss_trace.py), and what kinds of critique are most useful.

Three tiers, because ~860 clauses of spec is ~250k tokens — only a 1M-context
model swallows the whole suite, and even then audit quality is better focused:

  full      — every document, DAG-ordered, tagged. Whole-suite consistency pass
              on a 1M-context model. (~250k tokens.)
  per-doc   — one sub-standard in FULL, prefixed with the OUTLINE of its
              transitive DAG dependencies (headings + clause IDs, no clause
              bodies) so imports can be checked without the full neighbor text.
              The sweet spot for a deep single-document audit on any chat UI
              (~30-50k tokens).
  skeleton  — the informative umbrella in full plus an outline of every
              sub-standard. A cheap "does the DAG hold together" cross-document
              view that fits anywhere.

Usage:
  python tools/kiss_bundle.py                       # write all tiers to dist/
  python tools/kiss_bundle.py --tier full           # only the whole-suite bundle
  python tools/kiss_bundle.py --stdout --tier full  # to stdout (pipe to clip)
  python tools/kiss_bundle.py --stdout --doc ops    # one sub-standard + dep outlines
  python tools/kiss_bundle.py --no-coverage         # skip the live kiss_trace embed
  python tools/kiss_bundle.py --out-dir build/audit # choose the output directory

Python 3.8+, standard library only. Reuses tools/kiss_trace.py for coverage.
"""
from __future__ import annotations

import argparse
import os
import re
import sys

# kiss_trace.py is the canonical clause/coverage engine; reuse it rather than
# re-implement the parser. When run as `python tools/kiss_bundle.py`, this
# script's directory is already sys.path[0]; add it explicitly so an `import
# kiss_bundle` from elsewhere (a test) can still find its sibling.
_HERE = os.path.dirname(os.path.abspath(__file__))
if _HERE not in sys.path:
    sys.path.insert(0, _HERE)
try:
    import kiss_trace as kt
except Exception:  # pragma: no cover - kiss_trace is a repo sibling and stdlib-only
    kt = None

# ---------------------------------------------------------------------------
# The suite shape. A valid topological sort (a document appears after every
# document it depends on) and, per edge, the label the umbrella's §2.2 edge
# table gives it. This mirrors spec/umbrella.md §2.2; kiss_bundle draws only the
# dependency structure from it, and the umbrella document itself remains the
# authority (it is included verbatim in every whole-suite and skeleton bundle).
# ---------------------------------------------------------------------------
FRONT_DOOR = "umbrella"
SUITE_ORDER = ["classify", "ops", "grammar", "contract",
               "announce", "synth", "consume", "emit", "conform"]
FULL_ORDER = [FRONT_DOOR] + SUITE_ORDER

ROLE = {
    "umbrella": "front-door (informative; defines no normative clauses)",
    "classify": "foundational — data vocabulary",
    "ops":      "foundational — computation vocabulary",
    "grammar":  "structural",
    "contract": "structural",
    "announce": "protocol",
    "synth":    "protocol (a.k.a. Synth/Provision)",
    "consume":  "structural",
    "emit":     "structural",
    "conform":  "cross-cutting (tests every other sub-standard)",
}

# Direct dependencies, each with its umbrella §2.2 edge label.
DEPS = {
    "umbrella": [],
    "classify": [],
    "ops":      [],
    "grammar":  [("classify", "STRUCTURAL"), ("ops", "STRUCTURAL")],
    "contract": [("classify", "STRUCTURAL"), ("ops", "STRUCTURAL"),
                 ("grammar", "STRUCTURAL")],
    "announce": [("classify", "OPAQUE"), ("contract", "OPAQUE")],
    "synth":    [("announce", "STRUCTURAL"), ("contract", "STRUCTURAL"),
                 ("ops", "STRUCTURAL")],
    "consume":  [("classify", "STRUCTURAL"), ("ops", "STRUCTURAL"),
                 ("contract", "STRUCTURAL")],
    "emit":     [("classify", "STRUCTURAL"), ("ops", "STRUCTURAL"),
                 ("contract", "STRUCTURAL")],
    "conform":  [(s, "TEST") for s in
                 ["classify", "ops", "grammar", "contract",
                  "announce", "synth", "consume", "emit"]],
}


def transitive_deps(stem):
    """Every stem `stem` depends on, directly or transitively, in topological
    order (a dependency before anything that depends on it)."""
    seen = set()

    def visit(s):
        for dep, _label in DEPS.get(s, []):
            if dep not in seen:
                seen.add(dep)
                visit(dep)
    visit(stem)
    return [s for s in FULL_ORDER if s in seen]


# ---------------------------------------------------------------------------
# Regexes for the outline (skeleton) view. A clause DEFINITION line and any
# markdown heading; both reuse kiss_trace's clause-id grammar so the two tools
# cannot drift on what a clause looks like.
# ---------------------------------------------------------------------------
_CID = kt.CLAUSE_ID if kt else r"KISS-[A-Z]+-\d+(?:\.\d+)?-\d{4}[a-z]?"
RE_DEF_LINE = re.compile(r"^\s*[-*]\s+\*\*(" + _CID + r")\*\*\s*[—–-]\s*(.*)$")
RE_HEAD_LINE = re.compile(r"^(#{1,6})\s+(.*\S)\s*$")
RE_FENCE = re.compile(r"^\s*(```|~~~)")


def est_tokens(text):
    """A deliberately-bracketed token estimate. English prose runs ~4 chars/token;
    dense spec text with identifiers, tables, and symbols runs closer to ~3.3, so
    the truth for this corpus sits between chars/4 and chars/3.3. Returned as
    (low, high) so callers can answer "will it fit" honestly rather than precisely."""
    n = len(text)
    return n / 4.0, n / 3.3


def _kfmt(lo, hi):
    return f"~{lo/1000:.0f}k-{hi/1000:.0f}k tokens"


# ---------------------------------------------------------------------------
# Coverage — reuse kiss_trace's engine to compute, for the audit brief, how many
# normative clauses actually have an executable conformance test. This is the
# single most useful orienting fact for an auditor: it points critique straight
# at the untested MUSTs. Defensive throughout: a coverage hiccup must never stop
# the bundle from being produced, since concatenation is the tool's real job.
# ---------------------------------------------------------------------------
def compute_coverage(spec_dir, conf_dir, tools_dir):
    if kt is None:
        return {"error": "kiss_trace.py could not be imported"}
    try:
        results = []
        for stem in kt.SPECS:
            path = os.path.join(spec_dir, stem + ".md")
            res = kt.DocResult(stem)
            if os.path.exists(path):
                kt.parse(path, res)
            results.append(res)

        clause_test = {}
        for res in results:
            for cid, t, _ in res.matrix:
                clause_test[cid] = t

        harness = kt.discover_tests(conf_dir)
        ledger = kt.read_ledger(os.path.join(conf_dir, "UNBACKED.tsv"))
        lint_cov = kt.discover_lint_coverage(tools_dir)

        # reverse traceability: a clause is backed if a test cites it, even if the
        # spec's *Test:* name and the harness fn name have drifted.
        cited = {}
        for tname, info in harness.items():
            for cid in info.get("clauses", ()):
                cited.setdefault(cid, set()).add(tname)

        backed, unbacked = {}, {}
        for c, t in clause_test.items():
            if t in harness or c in cited:
                backed[c] = t
            else:
                unbacked[c] = t

        def eff_category(c):
            if c in lint_cov:
                return "lint"
            p = ledger.get(c)
            if p and p["category"] != "untested":
                return p["category"]
            return "untested"

        lint_only = sorted(c for c in unbacked if c in lint_cov)
        genuinely_untested = sorted(
            c for c in unbacked if eff_category(c) == "untested")

        per = {}
        for c in clause_test:
            sub = kt.sub_of(c)
            slot = per.setdefault(sub, [0, 0])
            slot[1] += 1
            if c in backed:
                slot[0] += 1

        return {
            "total": len(clause_test),
            "backed": len(backed),
            "unbacked": len(unbacked),
            "lint_only": len(lint_only),
            "genuinely_untested": genuinely_untested,
            "per_sub": per,
        }
    except Exception as exc:  # pragma: no cover - defensive
        return {"error": f"{type(exc).__name__}: {exc}"}


def format_coverage(cov, sample=50):
    if not cov or "error" in cov:
        why = cov.get("error", "unknown") if cov else "not computed"
        return ("## Live traceability status\n"
                f"(unavailable: {why}. Treat every clause as possibly untested; "
                "the suite's own gate is `python tools/kiss_trace.py --report`.)\n")
    lines = []
    lines.append("## Live traceability status "
                 "(generated from tools/kiss_trace.py against conformance/)")
    pct = 100.0 * cov["backed"] / cov["total"] if cov["total"] else 0.0
    lines.append(
        f"{cov['backed']} of {cov['total']} normative clauses ({pct:.1f}%) are backed "
        f"by an executable conformance test. {cov['unbacked']} are not; of those, "
        f"{cov['lint_only']} are enforced only by a document lint and "
        f"{len(cov['genuinely_untested'])} are GENUINELY UNTESTED — a normative MUST "
        f"with no test, no lint, and no recorded reason. Nothing in the suite is "
        f"Frozen; this is a pre-freeze draft.")
    lines.append("")
    lines.append("Executable coverage by sub-standard (backed / total):")
    for sub in sorted(cov["per_sub"], key=lambda s: -cov["per_sub"][s][1]):
        b, n = cov["per_sub"][sub]
        p = 100.0 * b / n if n else 0.0
        lines.append(f"  {sub:<9} {b:>4}/{n:<4} {p:>5.1f}%")
    ut = cov["genuinely_untested"]
    if ut:
        lines.append("")
        lines.append(
            f"Genuinely-untested clauses (high-value audit targets — is each even "
            f"testable as written?). Showing {min(sample, len(ut))} of {len(ut)}; "
            f"full list via `python tools/kiss_trace.py --strict`:")
        shown = ", ".join(ut[:sample])
        if len(ut) > sample:
            shown += f", ... and {len(ut) - sample} more"
        lines.append("  " + shown)
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# Document reading + the outline (skeleton) reduction.
# ---------------------------------------------------------------------------
def read_doc(spec_dir, stem):
    path = os.path.join(spec_dir, stem + ".md")
    with open(path, encoding="utf-8") as fh:
        return fh.read()


def outline(text, summary_len=140):
    """Reduce a document to its navigable skeleton: every heading (at its depth)
    interleaved, in document order, with each clause's id + a one-line summary.
    Clause BODIES, prose, tables, and code are dropped. A `#` inside a fenced
    code block is a shell comment, not a heading, so fenced regions are skipped."""
    out, in_fence = [], False
    for line in text.splitlines():
        if RE_FENCE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        mdef = RE_DEF_LINE.match(line)
        if mdef:
            cid, rest = mdef.group(1), mdef.group(2)
            summary = re.sub(r"\*\*|`", "", rest).strip()
            if len(summary) > summary_len:
                summary = summary[:summary_len].rstrip() + "…"
            out.append(f"- {cid} — {summary}" if summary else f"- {cid}")
            continue
        mh = RE_HEAD_LINE.match(line)
        if mh:
            out.append(f"{mh.group(1)} {mh.group(2)}")
    return "\n".join(out)


def doc_block(spec_dir, stem, mode):
    """A single document wrapped in its boundary tag. `mode` is 'full' (verbatim
    text) or 'outline' (skeleton only)."""
    deps = ", ".join(d for d, _ in DEPS.get(stem, [])) or "none"
    attrs = (f'path="spec/{stem}.md" sub-standard="{stem}" '
             f'role="{ROLE.get(stem, "?")}" depends-on="{deps}" mode="{mode}"')
    if mode == "outline":
        body = ("(OUTLINE ONLY — headings and clause identifiers, no clause "
                "bodies. Included as dependency context; audit the full text via "
                f"the per-document bundle for {stem}.)\n\n" + outline(read_doc(spec_dir, stem)))
    else:
        body = read_doc(spec_dir, stem)
    return f"<document {attrs}>\n{body}\n</document>"


# ---------------------------------------------------------------------------
# The audit brief (preamble). The single most important part of the bundle: it
# is what turns a wall of text into a targeted audit.
# ---------------------------------------------------------------------------
def edge_table_text():
    rows = []
    for stem in SUITE_ORDER:
        for dep, label in DEPS[stem]:
            rows.append(f"  KISS-{dep.capitalize()} -> KISS-{stem.capitalize()}  [{label}]")
    rows.append("  each of the other eight -> KISS-Conform  [TEST]")
    return "\n".join(rows)


def build_preamble(what_you_see, coverage_text):
    return f"""<audit-brief>
# KISS Specification Suite — audit bundle

You are asked to AUDIT, CRITIQUE, and SUGGEST IMPROVEMENTS to the KISS
specification reproduced below. KISS (Kernel Interface Standards Suite) is a
suite of interrelated, independently-conformable standards that define the
*interface between* machine-learning libraries, compute libraries, and kernel
providers: how a provider announces the kernels it has, how the two parties
negotiate capabilities and operand shapes/types, how a consumer learns what a
kernel computes and how to call it (the contract), and how a missing kernel is
provisioned on request. It standardizes the seam between software — wire
formats, ABIs, protocols, and a shared vocabulary of data, computation, and
contracts — NOT kernel implementations, source languages, in-ecosystem
load/dispatch, or any implementation's IR internals.

## What you are looking at
{what_you_see}

Each source file is wrapped in an XML-style tag:
`<document path="spec/NAME.md" sub-standard="NAME" role="..." depends-on="..." mode="full|outline">`.
A file boundary is a `</document>` tag — NEVER a markdown heading. The documents
contain their own `#`/`##` headings, including mid-document H1s (e.g. a bare
`# NORMATIVE CONFORMANCE SPECIFICATION (§6+)` line), so do not treat any heading
as a file boundary. A `mode="outline"` document is headings + clause identifiers
ONLY (dependency context), not the full clause text.

## The suite: nine sub-standards + one informative umbrella
Read a dependency before its dependents; documents below are in topological
order. Edges point dependency -> dependent; each carries the umbrella §2.2 label
(STRUCTURAL = depends on the other's parsed structure; OPAQUE = carries it as
length-delimited bytes it never parses; TEST = Conform tests it):
{edge_table_text()}

## Normative conventions you need to critique accurately
- Clause identifiers look like `KISS-OPS-6.0-0001` = `KISS-<SUB>-<section>-<NNNN>`
  with an optional trailing atomicity letter. They are append-only (a retired id
  is burned, never reused) and each maps 1:1 to a named conformance test.
- Each sub-standard is a dual-document template: sections §0-§5 are
  front-matter / informative (purpose, rationale, terms, references,
  conventions); the normative conformance specification is §6 onward. A `*Test:*`
  tag under a clause names its conformance test; a §9 matrix repeats the mapping.
- Keywords MUST / SHOULD / MAY follow RFC 2119 / 8174.
- "Frozen" is a lifecycle state (a frozen clause's wire bytes cannot change);
  nothing in this bundle is Frozen — it is a pre-freeze draft proposal.

{coverage_text}
## What critique is most useful (be specific — cite clause IDs)
1. Cross-document contradictions and vocabulary drift — a term one sub-standard
   OWNS but another redefines, re-forks, or uses inconsistently (the suite's
   central risk; e.g. the determinism/fidelity enum owned by KISS-Ops §6.0).
2. DAG / ownership violations — a clause depending "upward", duplicating a
   foundational definition, or parsing bytes an OPAQUE edge says it must not.
3. Ambiguous or untestable normative language — a MUST that no conformance test
   could actually check, or whose pass/fail is undefined. The untested clauses
   listed above are prime suspects.
4. Scope creep — a clause standardizing something the umbrella §1.2 excludes
   (kernel implementations, source languages, intra-ecosystem load/dispatch, IR
   internals).
5. Gaps — a behaviour described in prose with no governing clause, or a clause
   whose correctness has no oracle / no golden vector.
6. Wire-format / ABI under-specification — byte layout, endianness, alignment,
   field order, or versioning that two independent implementers could realize
   differently while both believing they conform.

Prefer concrete failure scenarios (inputs, bytes, two-implementation divergence)
over general impressions. When you propose a change, name the clause and give the
replacement text.
</audit-brief>"""


# ---------------------------------------------------------------------------
# Tier builders.
# ---------------------------------------------------------------------------
def build_full(spec_dir, coverage_text):
    what = ("The COMPLETE suite: the informative umbrella followed by all nine "
            "sub-standards in full, in topological order. This is large (~250k "
            "tokens) — paste it only into a long-context model. For a focused "
            "single-document audit on a smaller context, use a per-document bundle.")
    parts = [build_preamble(what, coverage_text)]
    for stem in FULL_ORDER:
        parts.append(doc_block(spec_dir, stem, "full"))
    return "\n\n".join(parts) + "\n"


def build_per_doc(spec_dir, stem, coverage_text):
    deps = transitive_deps(stem)
    dep_note = (", ".join(deps) if deps else "none")
    what = (f"ONE sub-standard in full — KISS-{stem.capitalize()} — preceded by "
            f"the OUTLINE (headings + clause ids only) of its dependency closure "
            f"({dep_note}) so you can check its imports without their full text. "
            f"Focus your audit on the full document; treat the outlines as the "
            f"vocabulary it is entitled to rely on.")
    parts = [build_preamble(what, coverage_text)]
    for dep in deps:
        parts.append(doc_block(spec_dir, dep, "outline"))
    parts.append(doc_block(spec_dir, stem, "full"))
    return "\n\n".join(parts) + "\n"


def build_skeleton(spec_dir, coverage_text):
    what = ("The SKELETON: the informative umbrella in full, then every "
            "sub-standard reduced to headings + clause identifiers (no clause "
            "bodies). Use it for a cheap cross-document consistency pass — does "
            "the dependency DAG hold together, are clause ranges contiguous, does "
            "an owned concept appear where it should? — not for wording-level review.")
    parts = [build_preamble(what, coverage_text)]
    parts.append(doc_block(spec_dir, FRONT_DOOR, "full"))
    for stem in SUITE_ORDER:
        parts.append(doc_block(spec_dir, stem, "outline"))
    return "\n\n".join(parts) + "\n"


# ---------------------------------------------------------------------------
# CLI.
# ---------------------------------------------------------------------------
def _report(path, text):
    lo, hi = est_tokens(text)
    kb = len(text.encode("utf-8")) / 1024
    print(f"  wrote {path}  ({kb:.0f} KB, {_kfmt(lo, hi)})", file=sys.stderr)


def main(argv=None):
    ap = argparse.ArgumentParser(
        description="Assemble the KISS spec suite into paste-into-an-LLM audit bundles.")
    ap.add_argument("--spec-dir", default=None, help="path to spec/ (default: repo spec/)")
    ap.add_argument("--conformance-dir", default=None,
                    help="path to conformance/ (default: repo conformance/)")
    ap.add_argument("--out-dir", default=None,
                    help="directory for the generated bundles (default: repo dist/)")
    ap.add_argument("--tier", choices=["all", "full", "per-doc", "skeleton"],
                    default="all", help="which tier(s) to write (default: all)")
    ap.add_argument("--doc", choices=SUITE_ORDER, default=None,
                    help="with --stdout, which sub-standard's per-doc bundle to emit")
    ap.add_argument("--stdout", action="store_true",
                    help="emit ONE bundle to stdout instead of writing files "
                         "(chosen by --doc, else --tier skeleton, else full)")
    ap.add_argument("--no-coverage", action="store_true",
                    help="skip the live kiss_trace coverage embed in the audit brief")
    # kiss_trace's coverage discovery runs every sibling `kiss_*.py --emit-coverage`.
    # kiss_bundle is a generator, not a lint: it declares no clause coverage, cleanly.
    ap.add_argument("--emit-coverage", action="store_true", help=argparse.SUPPRESS)
    args = ap.parse_args(argv)

    if args.emit_coverage:
        return 0  # not a lint; contributes nothing to kiss_trace's coverage map

    root = os.path.dirname(_HERE)
    spec_dir = args.spec_dir or os.path.join(root, "spec")
    conf_dir = args.conformance_dir or os.path.join(root, "conformance")
    out_dir = args.out_dir or os.path.join(root, "dist")

    missing = [s for s in FULL_ORDER if not os.path.exists(os.path.join(spec_dir, s + ".md"))]
    if missing:
        print(f"error: spec files missing from {spec_dir}: {', '.join(missing)}",
              file=sys.stderr)
        return 2

    if args.no_coverage:
        coverage_text = ("## Live traceability status\n(omitted with --no-coverage; "
                         "run `python tools/kiss_trace.py --report`.)\n")
    else:
        coverage_text = format_coverage(compute_coverage(spec_dir, conf_dir, _HERE))

    if args.stdout:
        if args.doc:
            text = build_per_doc(spec_dir, args.doc, coverage_text)
        elif args.tier == "skeleton":
            text = build_skeleton(spec_dir, coverage_text)
        else:
            text = build_full(spec_dir, coverage_text)
        # Write UTF-8 bytes directly: the spec text carries —, ±, ∞, → and the
        # like, and sys.stdout on Windows defaults to a legacy code page that
        # cannot encode them (a silent 0-byte emit when piped). The file paths
        # already pin encoding="utf-8"; the stdout path must too.
        sys.stdout.buffer.write(text.encode("utf-8"))
        return 0

    os.makedirs(out_dir, exist_ok=True)
    print(f"KISS audit bundles -> {out_dir}", file=sys.stderr)
    if args.tier in ("all", "full"):
        p = os.path.join(out_dir, "kiss-suite-full.md")
        with open(p, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(build_full(spec_dir, coverage_text))
        _report(p, open(p, encoding="utf-8").read())
    if args.tier in ("all", "per-doc"):
        for stem in SUITE_ORDER:
            p = os.path.join(out_dir, f"kiss-{stem}.md")
            with open(p, "w", encoding="utf-8", newline="\n") as fh:
                fh.write(build_per_doc(spec_dir, stem, coverage_text))
            _report(p, open(p, encoding="utf-8").read())
    if args.tier in ("all", "skeleton"):
        p = os.path.join(out_dir, "kiss-skeleton.md")
        with open(p, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(build_skeleton(spec_dir, coverage_text))
        _report(p, open(p, encoding="utf-8").read())
    return 0


if __name__ == "__main__":
    sys.exit(main())
