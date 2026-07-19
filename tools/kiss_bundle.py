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

The suite SHAPE is not hardcoded: the document set, the dependency DAG, its edge
labels, and the topological order are all DERIVED at run time from the umbrella's
§2.2 edge table (which the umbrella declares authoritative for prerequisite
closure) and reconciled against kiss_trace's SPECS list and the spec/ directory.
Any disagreement between those three — a new sub-standard, a changed edge, a doc
on disk nobody wired in — is a hard error, not a silently stale bundle. A tool
whose whole job is faithful audit input must not drift from the spec it mirrors.

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
import heapq
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

# The informative front-door document, and the single cross-cutting sub-standard
# whose dependency is "every other one" (summarized, not enumerated, in the
# umbrella's edge table). Both are stable, named members of the suite.
FRONT_DOOR = "umbrella"
CROSS_CUTTING = "conform"

# Editorial one-line role descriptions, used only for the <document role="...">
# attribute. Purely cosmetic: a document without an entry falls back to a role
# derived from its position in the DAG (see Suite.role), so a newly-added
# sub-standard still gets a sensible label without a code change.
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

_NUMWORDS = {1: "one", 2: "two", 3: "three", 4: "four", 5: "five", 6: "six",
             7: "seven", 8: "eight", 9: "nine", 10: "ten", 11: "eleven",
             12: "twelve"}


def _numword(n):
    return _NUMWORDS.get(n, str(n))


class SuiteError(Exception):
    """The derived suite shape disagrees with an authoritative source (the
    umbrella §2.2 DAG, kiss_trace's SPECS, or the spec/ directory). Raised rather
    than silently emitting a stale or partial bundle."""


# ---------------------------------------------------------------------------
# Deriving the suite shape from the spec itself.
# ---------------------------------------------------------------------------
def _name_to_stem(name):
    """`KISS-Synth/Provision` -> `synth`, `KISS-Conform` -> `conform`; anything
    not spelled `KISS-<Name>` (a table header cell, prose) -> None."""
    name = name.strip()
    if not name.startswith("KISS-"):
        return None
    return name[len("KISS-"):].split("/")[0].strip().lower()


def parse_edge_table(umbrella_text):
    """Derive the dependency DAG from the umbrella's §2.2 edge table.

    Returns (deps, nodes) where deps maps each sub-standard stem to a list of
    (dependency_stem, label) pairs and nodes is the set of every sub-standard.
    The table's explicit `| KISS-X → KISS-Y | LABEL |` rows give every non-test
    edge; the single summary row `each of the other eight → KISS-Conform` is
    expanded to a TEST edge from every other node. Raises SuiteError if the
    section, the rows, or the labels are not what this tool knows how to read —
    a loud failure is correct, because a mis-parsed DAG would mis-order and
    mis-scope every bundle.
    """
    lines = umbrella_text.splitlines()
    start = next((i for i, ln in enumerate(lines)
                  if re.match(r"^###\s+2\.2(\s|\.|$)", ln)), None)
    if start is None:
        raise SuiteError("umbrella §2.2 (Dependency DAG) heading not found")
    end = next((j for j in range(start + 1, len(lines))
                if re.match(r"^#{2,3}\s+\d", lines[j])), len(lines))

    edges = []          # (dep_stem, dependent_stem, label)
    nodes = set()
    aggregate = set()   # dependents whose dependency is "each of the others"
    in_fence = False
    for ln in lines[start:end]:
        if RE_FENCE.match(ln):
            in_fence = not in_fence
            continue
        if in_fence or "→" not in ln or not ln.lstrip().startswith("|"):
            continue
        cells = [c.strip() for c in ln.strip().strip("|").split("|")]
        if len(cells) < 2 or "→" not in cells[0]:
            continue
        dep_s, dependent_s = (x.strip() for x in cells[0].split("→", 1))
        dependent = _name_to_stem(dependent_s)
        if dependent is None:          # the `Dependency → Dependent` header row
            continue
        nodes.add(dependent)
        dep = _name_to_stem(dep_s)
        if dep is None:                # `each of the other eight → KISS-Conform`
            aggregate.add(dependent)
        else:
            nodes.add(dep)
            edges.append((dep, dependent, cells[1].split()[0].upper() if cells[1] else ""))

    if not edges:
        raise SuiteError("umbrella §2.2 edge table parsed no dependency rows")
    bad = sorted({lab for _, _, lab in edges if lab not in ("STRUCTURAL", "OPAQUE")})
    if bad:
        raise SuiteError("umbrella §2.2 has unrecognized edge label(s): " + ", ".join(bad))

    deps = {n: [] for n in nodes}
    for dep, dependent, lab in edges:
        deps[dependent].append((dep, lab))
    for agg in aggregate:              # conform depends on all others, as tests
        deps[agg] = [(n, "TEST") for n in sorted(nodes) if n != agg]
    return deps, nodes


def reconcile(nodes, front_door, trace_specs, disk_stems):
    """Every way the derived DAG can disagree with an authoritative source,
    as a list of human-readable error strings (empty == consistent). This is the
    ratchet: a new sub-standard, a renamed doc, or a file nobody wired in shows
    up here and stops the build until the three sources agree again."""
    errs = []
    if trace_specs is not None:
        expected = set(trace_specs) - {front_door}
        extra = nodes - expected
        gone = expected - nodes
        if extra:
            errs.append("umbrella §2.2 names sub-standard(s) kiss_trace SPECS lacks: "
                        + ", ".join(sorted(extra)))
        if gone:
            errs.append("kiss_trace SPECS has sub-standard(s) absent from umbrella §2.2: "
                        + ", ".join(sorted(gone)))
    ghost = nodes - disk_stems
    if ghost:
        errs.append("umbrella §2.2 names doc(s) with no spec/ file: " + ", ".join(sorted(ghost)))
    unwired = disk_stems - nodes - {front_door}
    if unwired:
        errs.append("spec/ has doc(s) not in the umbrella §2.2 DAG "
                    "(new sub-standard? wire it into the umbrella): " + ", ".join(sorted(unwired)))
    return errs


def _topo_key(stem):
    """Order nodes by their position in kiss_trace's SPECS sequence when known,
    so the derived order matches the suite's own declared sequence; fall back to
    alphabetical for a node kiss_trace does not list (kt absent, or brand-new)."""
    if kt is not None and stem in kt.SPECS:
        return (0, kt.SPECS.index(stem))
    return (1, stem)


def topo_order(deps, key=_topo_key):
    """Kahn's algorithm over the DAG (edges point dependency -> dependent), with a
    priority queue on `key` so the order is deterministic. Raises SuiteError on a
    cycle — which would mean the suite's prerequisite closure is ill-defined."""
    indeg = {n: 0 for n in deps}
    dependents = {n: [] for n in deps}
    for stem, dl in deps.items():
        for dep, _ in dl:
            indeg[stem] += 1
            dependents[dep].append(stem)
    heap = [(key(n), n) for n in deps if indeg[n] == 0]
    heapq.heapify(heap)
    order = []
    while heap:
        _, n = heapq.heappop(heap)
        order.append(n)
        for m in dependents[n]:
            indeg[m] -= 1
            if indeg[m] == 0:
                heapq.heappush(heap, (key(m), m))
    if len(order) != len(deps):
        stuck = sorted(n for n in deps if n not in order)
        raise SuiteError("dependency DAG has a cycle among: " + ", ".join(stuck))
    return order


class Suite:
    """The derived, validated suite shape passed to every builder."""

    def __init__(self, front_door, suite_order, deps, roles):
        self.front_door = front_door
        self.suite_order = suite_order            # topological, sub-standards only
        self.full_order = [front_door] + suite_order
        self.deps = deps                          # stem -> [(dep, label), ...]
        self._roles = roles

    def role(self, stem):
        if stem in self._roles:
            return self._roles[stem]
        dl = self.deps.get(stem, [])              # graph-derived fallback
        if not dl:
            return "foundational"
        labels = {lab for _, lab in dl}
        if labels == {"OPAQUE"}:
            return "protocol"
        if labels == {"TEST"}:
            return "cross-cutting"
        return "structural"

    def transitive_deps(self, stem):
        """Every stem `stem` depends on, directly or transitively, in topological
        order (a dependency before anything that depends on it)."""
        seen = set()

        def visit(s):
            for dep, _ in self.deps.get(s, []):
                if dep not in seen:
                    seen.add(dep)
                    visit(dep)
        visit(stem)
        return [s for s in self.full_order if s in seen]

    def edge_table_text(self):
        """The umbrella §2.2 edge set, rendered for the audit brief: one line per
        explicit edge, then the cross-cutting sub-standard summarized (mirroring
        the umbrella, which does not enumerate its every-other-doc test edges)."""
        rows = []
        for stem in self.suite_order:
            if stem == CROSS_CUTTING:
                continue
            for dep, label in self.deps[stem]:
                rows.append(f"  KISS-{dep.capitalize()} -> KISS-{stem.capitalize()}  [{label}]")
        n = len(self.deps.get(CROSS_CUTTING, []))
        rows.append(f"  each of the other {_numword(n)} -> KISS-Conform  [TEST]")
        return "\n".join(rows)


def load_suite(spec_dir):
    """Read the umbrella, derive the DAG, reconcile it against kiss_trace and the
    filesystem, and return a validated Suite. Raises SuiteError on any drift."""
    upath = os.path.join(spec_dir, FRONT_DOOR + ".md")
    if not os.path.exists(upath):
        raise SuiteError(f"missing front-door spec: {upath}")
    deps, nodes = parse_edge_table(read_doc(spec_dir, FRONT_DOOR))

    disk_stems = {f[:-3] for f in os.listdir(spec_dir) if f.endswith(".md")}
    errs = reconcile(nodes, FRONT_DOOR, kt.SPECS if kt else None, disk_stems)
    if errs:
        raise SuiteError("suite shape has drifted from its sources:\n  - " + "\n  - ".join(errs))

    order = topo_order(deps)
    if CROSS_CUTTING in deps:      # display the cross-cutting deps in suite order
        deps[CROSS_CUTTING].sort(key=lambda dl: order.index(dl[0]))
    return Suite(FRONT_DOOR, order, deps, dict(ROLE))


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


def doc_block(spec_dir, suite, stem, mode):
    """A single document wrapped in its boundary tag. `mode` is 'full' (verbatim
    text) or 'outline' (skeleton only)."""
    deps = ", ".join(d for d, _ in suite.deps.get(stem, [])) or "none"
    attrs = (f'path="spec/{stem}.md" sub-standard="{stem}" '
             f'role="{suite.role(stem)}" depends-on="{deps}" mode="{mode}"')
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
def build_preamble(suite, what_you_see, coverage_text):
    n = _numword(len(suite.suite_order))
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

## The suite: {n} sub-standards + one informative umbrella
Read a dependency before its dependents; documents below are in topological
order. Edges point dependency -> dependent; each carries the umbrella §2.2 label
(STRUCTURAL = depends on the other's parsed structure; OPAQUE = carries it as
length-delimited bytes it never parses; TEST = Conform tests it):
{suite.edge_table_text()}

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
def build_full(spec_dir, suite, coverage_text):
    n = _numword(len(suite.suite_order))
    what = (f"The COMPLETE suite: the informative umbrella followed by all {n} "
            "sub-standards in full, in topological order. This is large (~250k "
            "tokens) — paste it only into a long-context model. For a focused "
            "single-document audit on a smaller context, use a per-document bundle.")
    parts = [build_preamble(suite, what, coverage_text)]
    for stem in suite.full_order:
        parts.append(doc_block(spec_dir, suite, stem, "full"))
    return "\n\n".join(parts) + "\n"


def build_per_doc(spec_dir, suite, stem, coverage_text):
    deps = suite.transitive_deps(stem)
    dep_note = (", ".join(deps) if deps else "none")
    what = (f"ONE sub-standard in full — KISS-{stem.capitalize()} — preceded by "
            f"the OUTLINE (headings + clause ids only) of its dependency closure "
            f"({dep_note}) so you can check its imports without their full text. "
            f"Focus your audit on the full document; treat the outlines as the "
            f"vocabulary it is entitled to rely on.")
    parts = [build_preamble(suite, what, coverage_text)]
    for dep in deps:
        parts.append(doc_block(spec_dir, suite, dep, "outline"))
    parts.append(doc_block(spec_dir, suite, stem, "full"))
    return "\n\n".join(parts) + "\n"


def build_skeleton(spec_dir, suite, coverage_text):
    what = ("The SKELETON: the informative umbrella in full, then every "
            "sub-standard reduced to headings + clause identifiers (no clause "
            "bodies). Use it for a cheap cross-document consistency pass — does "
            "the dependency DAG hold together, are clause ranges contiguous, does "
            "an owned concept appear where it should? — not for wording-level review.")
    parts = [build_preamble(suite, what, coverage_text)]
    parts.append(doc_block(spec_dir, suite, suite.front_door, "full"))
    for stem in suite.suite_order:
        parts.append(doc_block(spec_dir, suite, stem, "outline"))
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
    ap.add_argument("--doc", default=None,
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

    try:
        suite = load_suite(spec_dir)
    except SuiteError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    if args.doc is not None and args.doc not in suite.suite_order:
        print(f"error: --doc {args.doc!r} is not a sub-standard; choose from: "
              f"{', '.join(suite.suite_order)}", file=sys.stderr)
        return 2

    if args.no_coverage:
        coverage_text = ("## Live traceability status\n(omitted with --no-coverage; "
                         "run `python tools/kiss_trace.py --report`.)\n")
    else:
        coverage_text = format_coverage(compute_coverage(spec_dir, conf_dir, _HERE))

    if args.stdout:
        if args.doc:
            text = build_per_doc(spec_dir, suite, args.doc, coverage_text)
        elif args.tier == "skeleton":
            text = build_skeleton(spec_dir, suite, coverage_text)
        else:
            text = build_full(spec_dir, suite, coverage_text)
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
            fh.write(build_full(spec_dir, suite, coverage_text))
        _report(p, open(p, encoding="utf-8").read())
    if args.tier in ("all", "per-doc"):
        for stem in suite.suite_order:
            p = os.path.join(out_dir, f"kiss-{stem}.md")
            with open(p, "w", encoding="utf-8", newline="\n") as fh:
                fh.write(build_per_doc(spec_dir, suite, stem, coverage_text))
            _report(p, open(p, encoding="utf-8").read())
    if args.tier in ("all", "skeleton"):
        p = os.path.join(out_dir, "kiss-skeleton.md")
        with open(p, "w", encoding="utf-8", newline="\n") as fh:
            fh.write(build_skeleton(spec_dir, suite, coverage_text))
        _report(p, open(p, encoding="utf-8").read())
    return 0


if __name__ == "__main__":
    sys.exit(main())
