#!/usr/bin/env python3
"""
kiss_tables.py — the KISS shared-enumeration consistency lint.

A *document* lint, complementary to tools/kiss_trace.py: where kiss_trace checks
that each clause maps to an executable test, this checks that the several places
the spec re-lists a shared, owned enumeration do not DRIFT out of agreement.

KISS deliberately re-states some sets in more than one place — an owner's
normative table, an informative overview table, a shared-vocabulary anchor
sentence — "so the Ops binary encoding is self-contained" (ops §6.16 preamble).
The cost is that an additive amendment can update one copy and miss another. That
is not hypothetical: PRs #30/#31 added `s16`/`u16`/`u64` to the Classify §6.1
dtype table and the Ops §6.16 layout table, but left the Ops §2.8 shared-anchor
list at seventeen tokens. A markdown check finds that class of drift the moment
it lands, at the PR, instead of years later by inspection.

THE DTYPE SET is the first (and, today, only) enumeration checked. Its owner and
single source of truth is KISS-CLASSIFY-6.1-0001, whose own words are "The scalar
dtype set MUST be **exactly** the twenty tokens in the table above (...)". This
lint extracts that authoritative set and asserts that:

  1. every FULL dtype markdown table in the suite carries exactly it (the
     informative §2.6 table, the normative §6.1 table, the Ops §6.16 layout
     table — detected structurally, by a first column that is ≥ 15 dtype tokens,
     so a subset table is never mistaken for a full one);
  2. every FULL dtype inline list carries exactly it — the §6.1-0001 parenthetical
     itself, and the Ops §2.8 "**dtype** tokens (...)" shared anchor — detected by
     their specific introducing phrases, so a subset list such as "dtypes
     (`f16`, `f32`, `f64`)" is never checked as if it were the whole set;
  3. the count word in §6.1-0001 ("twenty") equals the set's actual size, so a
     token added to the table without updating the prose count is caught too.

Exit status is 0 when every site agrees and 1 on any drift, so the checker is a
CI gate. Stdlib only.

Usage:  python tools/kiss_tables.py [--spec-dir spec]
"""
from __future__ import annotations

import argparse
import os
import re
import sys

NUMBER_WORDS = {
    "seventeen": 17, "eighteen": 18, "nineteen": 19, "twenty": 20,
    "twenty-one": 21, "twenty-two": 22, "twenty-three": 23,
}

# The umbrella + the nine sub-standards (umbrella defines no clauses, but the
# shared enums it narrates may appear in any of the ten files).
SPECS = ["umbrella", "announce", "classify", "ops", "grammar", "contract",
         "synth", "consume", "emit", "conform"]


def _is_dtype_token(w):
    """A dtype token is a lowercase alnum word that contains a digit (f16, bf16,
    s16, e4m3, c64, b1, ...) or is exactly `bool`. This admits the interleaved
    FP8/FP4 names (e4m3, e5m2, e2m1) and excludes digit-free field names (`rank`,
    `structure_key`) and prose, so a mixed backtick list yields only the dtypes.
    """
    return bool(re.fullmatch(r"[a-z][a-z0-9]*", w)) and (
        any(c.isdigit() for c in w) or w == "bool")


def backtick_tokens(text):
    """Every dtype token inside a backtick span in `text`, in order.

    Handles both spellings the spec uses: comma-separated individual spans
    (`f16`, `bf16`, ...) and one space-separated span (`f16 bf16 ...`).
    """
    out = []
    for span in re.findall(r"`([^`]+)`", text):
        out.extend(w for w in re.split(r"[\s,]+", span.strip()) if _is_dtype_token(w))
    return out


def authoritative_set(classify_text):
    """The dtype set owned by KISS-CLASSIFY-6.1-0001, plus its declared count word.

    Returns (tokens_in_order, count_word_or_None). The clause text is:
      "The scalar dtype set MUST be **exactly** the twenty tokens in the table
       above (`f16`, `bf16`, ... `c64`); ..."
    """
    m = re.search(
        r"scalar dtype set MUST be \*\*exactly\*\* the\s+([a-z-]+)\s+tokens"
        r"[^(]*\((.*?)\)",
        classify_text, re.S)
    if not m:
        return None, None
    return backtick_tokens(m.group(2)), m.group(1)


def markdown_tables(text):
    """Yield each markdown table as a list of rows, each row a list of cell strings."""
    rows, in_table = [], False
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("|") and s.endswith("|"):
            in_table = True
            rows.append([c.strip() for c in s.strip("|").split("|")])
        elif in_table:
            yield rows
            rows, in_table = [], False
    if in_table:
        yield rows


def full_dtype_tables(text, authoritative):
    """Every markdown table whose first column is a FULL dtype set (≥15 tokens).

    A subset table (e.g. a 3-row op-domain table) never clears the threshold, so
    it is not checked against the whole set.
    """
    auth = set(authoritative)
    for rows in markdown_tables(text):
        col0 = []
        for r in rows:
            if not r:
                continue
            cell = r[0].strip().strip("`").strip()
            toks = backtick_tokens(r[0]) or ([cell] if _is_dtype_token(cell) else [])
            if toks:
                col0.append(toks[0])
        hits = [t for t in col0 if t in auth]
        if len(hits) >= 15:
            yield col0


def inline_anchor_lists(text):
    """FULL dtype inline lists, by their specific introducing phrases.

    Two sites qualify — and only these, so subset lists are never caught:
      * the Ops §2.8 shared anchor:  "the **dtype** tokens (`...`)"
      * (the §6.1-0001 parenthetical is the authoritative source, checked apart)
    """
    for m in re.finditer(r"\*\*dtype\*\* tokens?\s*\((.*?)\)", text, re.S):
        yield "dtype-anchor", backtick_tokens(m.group(1))


def check(spec_dir):
    def read(stem):
        p = os.path.join(spec_dir, stem + ".md")
        return open(p, encoding="utf-8").read() if os.path.exists(p) else ""

    classify, ops = read("classify"), read("ops")
    violations = []

    auth, count_word = authoritative_set(classify)
    if not auth:
        return ["could not find the KISS-CLASSIFY-6.1-0001 authoritative dtype list"]
    auth_set = set(auth)

    # (3) the declared count word must equal the actual size
    if count_word in NUMBER_WORDS and NUMBER_WORDS[count_word] != len(auth):
        violations.append(
            f"§6.1-0001 says '{count_word}' ({NUMBER_WORDS[count_word]}) but lists "
            f"{len(auth)} tokens")
    if len(auth) != len(set(auth)):
        dup = [t for t in auth if auth.count(t) > 1]
        violations.append(f"§6.1-0001 list has duplicate token(s): {sorted(set(dup))}")

    def diff(where, got):
        g = set(got)
        if g != auth_set:
            miss = sorted(auth_set - g)
            extra = sorted(g - auth_set)
            parts = []
            if miss:
                parts.append(f"missing {miss}")
            if extra:
                parts.append(f"unexpected {extra}")
            violations.append(f"{where}: {'; '.join(parts)} (vs §6.1-0001 owner)")

    # (1) full dtype tables in classify.md and ops.md
    for stem, text in (("classify", classify), ("ops", ops)):
        for i, col0 in enumerate(full_dtype_tables(text, auth), 1):
            diff(f"{stem}.md full dtype table #{i}", col0)

    # (2) full dtype inline anchor lists (Ops §2.8 today)
    for kind, toks in inline_anchor_lists(ops):
        diff(f"ops.md {kind} inline list", toks)
    for kind, toks in inline_anchor_lists(classify):
        diff(f"classify.md {kind} inline list", toks)

    # (4) the determinism/fidelity enum — owned by KISS-OPS-6.0-0001, spelled
    # verbatim, imported never re-forked (umbrella §2.1, §3.3). Every brace-form
    # of the enum across the suite must carry EXACTLY the canonical member set.
    violations += check_determinism_enum(spec_dir)

    # (5) the compute-fidelity (MathPrecision) enum — owned by KISS-OPS-6.17-0001,
    # same import-never-re-fork rule as the determinism enum.
    violations += check_mathprecision_enum(spec_dir)

    # (6) the KISS-Consume refusal taxonomy — owned by KISS-CONSUME-6.4-0001.
    violations += check_refusal_taxonomy(spec_dir)

    # (7) the KISS-Contract universal seven-section core — owned by KISS-CONTRACT-6.2-0001.
    violations += check_seven_sections(spec_dir)

    # (8) the reduce_axes four-category set — Ops §6.11-0011 <-> Classify §6.7-0005.
    violations += check_reduce_axes(spec_dir)

    # (9) the two FEAT capability bits — Announce §7.2 definition <-> §7.2-0003 clause.
    violations += check_feat_bits(spec_dir)

    return violations, auth


# The canonical determinism/fidelity enum owned by KISS-OPS-6.0-0001.
DETERMINISM_MEMBERS = {"exact-byte", "ULP/tolerance", "order-invariant/nondeterministic"}


def check_determinism_enum(spec_dir):
    """Every `{...}` enum form containing `exact-byte`, anywhere in the suite, MUST
    carry exactly the canonical member set — this catches a downstream sub-standard
    re-forking the enum (adding/renaming a class) instead of importing it, which
    would silently split the comparator vocabulary the whole DAG shares."""
    out = []
    owner_seen = False
    for stem in SPECS:
        p = os.path.join(spec_dir, stem + ".md")
        if not os.path.exists(p):
            continue
        text = open(p, encoding="utf-8").read()
        for m in re.finditer(r"\{[^{}]*exact-byte[^{}]*\}", text):
            members = {t.strip() for t in re.split(r"[,\s]*,[,\s]*",
                       " ".join(m.group(0)[1:-1].split())) if t.strip()}
            if stem == "ops":
                owner_seen = True
            if members != DETERMINISM_MEMBERS:
                miss = sorted(DETERMINISM_MEMBERS - members)
                extra = sorted(members - DETERMINISM_MEMBERS)
                parts = []
                if miss:
                    parts.append(f"missing {miss}")
                if extra:
                    parts.append(f"unexpected {extra}")
                out.append(f"{stem}.md determinism enum re-forked: {'; '.join(parts)} "
                           f"(vs KISS-OPS-6.0-0001 owner)")
    if not owner_seen:
        out.append("KISS-OPS-6.0-0001 owner: no canonical determinism enum found in ops.md")
    return out


# The canonical compute-fidelity (MathPrecision) enum owned by KISS-OPS-6.17-0001.
MATHPRECISION_MEMBERS = {"bit-stable", "reduced-mantissa-permitted"}


def check_mathprecision_enum(spec_dir):
    """Every `{...}` form containing `bit-stable`, anywhere in the suite, MUST carry
    exactly the two canonical members — Contract/Consume/Emit/Synth import this enum
    "verbatim, never re-forked", so a downstream copy that renames or adds a value is
    a re-fork this catches. The owner also declares the count word `two-member`."""
    out = []
    owner_seen = False
    for stem in SPECS:
        p = os.path.join(spec_dir, stem + ".md")
        if not os.path.exists(p):
            continue
        text = open(p, encoding="utf-8").read()
        # brace forms may straddle a line break, so scan whole-file text
        for m in re.finditer(r"\{[^{}]*bit-stable[^{}]*\}", text):
            members = {t.strip() for t in re.split(r"[,\s]*,[,\s]*",
                       " ".join(m.group(0)[1:-1].split())) if t.strip()}
            if stem == "ops":
                owner_seen = True
            if members != MATHPRECISION_MEMBERS:
                miss = sorted(MATHPRECISION_MEMBERS - members)
                extra = sorted(members - MATHPRECISION_MEMBERS)
                parts = []
                if miss:
                    parts.append(f"missing {miss}")
                if extra:
                    parts.append(f"unexpected {extra}")
                out.append(f"{stem}.md MathPrecision enum re-forked: {'; '.join(parts)} "
                           f"(vs KISS-OPS-6.17-0001 owner)")
    if not owner_seen:
        out.append("KISS-OPS-6.17-0001 owner: no canonical MathPrecision enum found in ops.md")
    # the owner's count word `two-member` must equal the member-set size
    ops = os.path.join(spec_dir, "ops.md")
    if os.path.exists(ops):
        otext = open(ops, encoding="utf-8").read()
        m = re.search(r"the\s+([a-z]+)-member enum `\{[^}]*bit-stable", otext)
        words = {"two": 2, "three": 3}
        if m and words.get(m.group(1)) not in (None, len(MATHPRECISION_MEMBERS)):
            out.append(f"§6.17-0001 says '{m.group(1)}-member' but the enum has "
                       f"{len(MATHPRECISION_MEMBERS)} members")
    return out


# The KISS-Consume refusal taxonomy owned by KISS-CONSUME-6.4-0001. The two whole-input
# tokens `not-a-kernel`/`wrong-op-class` never appear in the legitimate 2-token residue
# subset {unrecognized-but-expressible, inexpressible-residue}, so their presence in a
# list marks it as a FULL-set restatement that must then name all four.
REFUSAL_MEMBERS = {"not-a-kernel", "wrong-op-class",
                   "unrecognized-but-expressible", "inexpressible-residue"}
REFUSAL_DISCRIMINATORS = {"not-a-kernel", "wrong-op-class"}


def _undecorate(s):
    """Drop markdown emphasis/backtick decoration so `**\\`not-a-kernel\\`**` == the token."""
    return s.replace("*", "").replace("`", "")


def check_refusal_taxonomy(spec_dir):
    """The owner clause names all four categories and the count word `four`; and every
    parenthetical list anywhere in the suite that names a whole-input category (the
    discriminator) MUST name all four — catching a downstream copy that drops or
    renames one. The deliberate 2-token residue subset carries no discriminator, so it
    is never mistaken for a full-set restatement."""
    out = []
    # (a) the owner clause: all four present + count word 'four'
    consume = os.path.join(spec_dir, "consume.md")
    if os.path.exists(consume):
        ctext = open(consume, encoding="utf-8").read()
        m = re.search(r"\*\*KISS-CONSUME-6\.4-0001\*\*(.*?)(?=\n\s*-\s+\*\*KISS|\n#)", ctext, re.S)
        if m:
            body = _undecorate(m.group(1))
            miss = sorted(t for t in REFUSAL_MEMBERS if t not in body)
            if miss:
                out.append(f"§6.4-0001 owner is missing categor(y/ies): {miss}")
            cw = re.search(r"MUST be exactly the\s+(\w+)\s+categories", body)
            words = {"four": 4}
            if cw and words.get(cw.group(1)) not in (None, len(REFUSAL_MEMBERS)):
                out.append(f"§6.4-0001 says '{cw.group(1)}' but names {len(REFUSAL_MEMBERS)}")
        else:
            out.append("KISS-CONSUME-6.4-0001 owner clause not found")
    # (b) every DOWNSTREAM parenthetical full-set restatement must carry all four.
    # The owner doc (consume.md) is excluded here — it discusses the categories in
    # deliberate subsets (the two whole-input failures, the two residue categories),
    # so token presence cannot tell an intentional subset from a typo there; the owner
    # is instead covered by check (a). Downstream docs only ever restate the full
    # taxonomy, so a discriminator token marks a full-set list that must name all four.
    for stem in SPECS:
        if stem == "consume":
            continue
        p = os.path.join(spec_dir, stem + ".md")
        if not os.path.exists(p):
            continue
        text = open(p, encoding="utf-8").read()
        for m in re.finditer(r"\(([^()]*)\)", text):
            grp = _undecorate(m.group(1))
            if REFUSAL_DISCRIMINATORS & {t for t in REFUSAL_MEMBERS if t in grp}:
                miss = sorted(t for t in REFUSAL_MEMBERS if t not in grp)
                if miss:
                    out.append(f"{stem}.md refusal-taxonomy list is missing {miss} "
                               f"(vs KISS-CONSUME-6.4-0001 owner)")
    return out


# The KISS-Contract universal required core owned by KISS-CONTRACT-6.2-0001, in order.
SEVEN_SECTIONS = ["Identity", "Semantics", "Interface", "Dispatch",
                  "Capabilities", "Guarantees", "Provenance"]


def _contract_clause(ctext, cid):
    m = re.search(r"\*\*KISS-CONTRACT-" + re.escape(cid) +
                  r"\*\*(.*?)(?=\n\s*-\s+\*\*KISS|\n#)", ctext, re.S)
    return m.group(1) if m else None


def _clause_body(text, sub, short):
    """Body of a `- **KISS-<SUB>-<short>** — ...` clause, up to the next clause or heading."""
    m = re.search(r"\*\*KISS-" + sub + r"-" + re.escape(short) +
                  r"\*\*(.*?)(?=\n\s*-\s+\*\*KISS|\n#)", text, re.S)
    return m.group(1) if m else None


# The four reduce-field categories (KISS-Ops §6.11-0011 / KISS-Classify §6.7-0005): three
# literal tokens plus the `x<hh>` keepdim-mask token (spelled `x<hh>` in Ops, `x` in Classify).
REDUCE_LITERALS = ["-", "rall", "rlast"]


def _reduce_literals(body):
    """The three literal reduce categories a clause DEFINES, keyed on the definition form
    `\\`tok\\` (meaning)` so an incidental later mention of the token is not counted."""
    return {m.group(1) for m in re.finditer(r"`([^`]+)`\s*\(", body)
            if m.group(1) in ("-", "rall", "rlast")}


def check_reduce_axes(spec_dir):
    """The reduce_axes field distinguishes exactly four categories, restated in both the
    Ops-semantics clause (§6.11-0011, the backed clause) and the Classify token-codec clause
    (§6.7-0005). Each MUST name the three literal categories `-`/`rall`/`rlast` (keyed on the
    `\\`tok\\` (meaning)` definition form) plus the `x`/`x<hh>` keepdim-mask, and carry the
    count word `four`. The Ops enumeration lists each token in that form exactly once, so a
    rename/drop there is caught (the backing for §6.11-0011); the Classify side restates some
    tokens more than once, so its check is a full-drop guard rather than a rename detector."""
    out = []
    for label, sub, short, stem in (("Ops", "OPS", "6.11-0011", "ops"),
                                    ("Classify", "CLASSIFY", "6.7-0005", "classify")):
        p = os.path.join(spec_dir, stem + ".md")
        body = _clause_body(open(p, encoding="utf-8").read(), sub, short) if os.path.exists(p) else None
        if not body:
            out.append(f"{label} §{short}: reduce_axes clause not found")
            continue
        lits = _reduce_literals(body)
        if lits != {"-", "rall", "rlast"}:
            miss = sorted({"-", "rall", "rlast"} - lits)
            out.append(f"{label} §{short} reduce_axes literal categories drift: missing {miss}")
        if not re.search(r"`x(<hh>)?`", body):
            out.append(f"{label} §{short}: the `x`/`x<hh>` keepdim-mask category is gone")
        if "four" not in body:
            out.append(f"{label} §{short}: the count word 'four' is gone from the reduce_axes description")
    return out


# The two provider-level FEAT capability bits (KISS-Announce §7.2), restated in the §7.2
# FEAT definition and the §7.2-0003 interpretation clause.
FEAT_BITS = {32: "PROVISION_ON_REQUEST", 33: "CONTRACT_QUERY"}


def check_feat_bits(spec_dir):
    """Within announce.md the bit->name mapping of the two FEAT capability bits is stated
    twice — the §7.2 `- **FEAT**` definition and the §7.2-0003 clause — and both MUST agree
    with {32: PROVISION_ON_REQUEST, 33: CONTRACT_QUERY}. Extraction uses strict adjacency
    (`bit NN [as] \\`NAME\\``), so a bare 'bit 32' mention with no adjacent name is ignored."""
    out = []
    p = os.path.join(spec_dir, "announce.md")
    if not os.path.exists(p):
        return ["announce.md not found for the FEAT-bit check"]
    text = open(p, encoding="utf-8").read()
    feat_def = re.search(r"- \*\*FEAT\*\*(.*?)(?=\n\s*-\s+\*\*|\n#)", text, re.S)
    sources = [("§7.2 FEAT definition", feat_def.group(1) if feat_def else None),
               ("§7.2-0003 clause", _clause_body(text, "ANNOUNCE", "7.2-0003"))]
    for label, region in sources:
        if region is None:
            out.append(f"announce {label}: not found")
            continue
        pairs = {int(b): n for b, n in re.findall(r"bit (3[23])\s+(?:as\s+)?`(\w+)`", region)}
        if pairs != FEAT_BITS:
            out.append(f"announce {label} FEAT-bit map drift: {pairs} (vs {FEAT_BITS})")
    return out


def check_seven_sections(spec_dir):
    """The seven contract-core section names, consistent everywhere the full ordered
    list is restated. Each list is order-checked, not just membership-checked, because
    §6.11-0004 carries the names TWICE (a numbered `Name(n)` order list and a lowercase
    `\\`name\\`` list) and a membership check would pass a rename of only one copy.

      (a) §6.2-0001 owner:  the `Name (§6.x)` core list == the seven in order + count 'seven'
      (b) §6.11-0004:       the `Name(n)` order list AND the `\\`name\\`` list each == the seven
      (c) restatements:     every clean delimited Identity..Provenance list == the seven in order
    """
    out = []
    canon = set(SEVEN_SECTIONS)
    canon_lower = [s.lower() for s in SEVEN_SECTIONS]
    contract = os.path.join(spec_dir, "contract.md")
    if os.path.exists(contract):
        ctext = open(contract, encoding="utf-8").read()
        # (a) owner §6.2-0001 — names each carry a `(§6.x)` section reference
        body = _contract_clause(ctext, "6.2-0001")
        if body is None:
            out.append("KISS-CONTRACT-6.2-0001 clause not found")
        else:
            names = re.findall(r"([A-Z][a-z]+)\s*\(§", body)
            if names != SEVEN_SECTIONS:
                out.append(f"§6.2-0001 core list drift: {names} (vs the seven in order)")
        if "seven section" not in ctext.lower():
            out.append("§6.2-0001: the count word 'seven' is gone from the core description")
        # (b) §6.11-0004 — both the numbered order list and the lowercase name list
        body = _contract_clause(ctext, "6.11-0004")
        if body is None:
            out.append("KISS-CONTRACT-6.11-0004 clause not found")
        else:
            numbered = [n for n, _d in re.findall(r"([A-Z][a-z]+)\((\d)\)", body)]
            if numbered != SEVEN_SECTIONS:
                out.append(f"§6.11-0004 numbered-order list drift: {numbered}")
            low = [w for w in re.findall(r"`([a-z]+)`", body) if w in set(canon_lower)]
            if low != canon_lower:
                out.append(f"§6.11-0004 lowercase name list drift: {low}")
    # (c) clean delimited Identity..Provenance lists across the suite (no inner parens)
    for stem in SPECS:
        p = os.path.join(spec_dir, stem + ".md")
        if not os.path.exists(p):
            continue
        text = open(p, encoding="utf-8").read()
        for m in re.finditer(r"[{(]\s*(Identity[^{}()]*?Provenance)\s*[})]", text):
            names = [w for w in re.split(r"[,\s]+", _undecorate(m.group(1))) if w in canon]
            if names != SEVEN_SECTIONS:
                out.append(f"{stem}.md seven-section list drift: {names} "
                           f"(vs KISS-CONTRACT-6.2-0001 order)")
    return out


# The normative clauses this lint ENFORCES: a violation of each fails the lint,
# so the traceability gate may count them lint-backed (they bind the spec document
# — the closed dtype set restated in two owners — which is a linter's job, not a
# harness test). Each is (clause_id, what a violation looks like).
COVERS = [
    ("KISS-CLASSIFY-6.1-0001",
     "the closed 20-token dtype set drifts (count word or a full dtype table)"),
    ("KISS-OPS-6.16-0001",
     "the Ops §6.16 dtype layout table drops/adds a token vs the Classify owner"),
    ("KISS-OPS-6.0-0001",
     "a sub-standard re-forks the determinism/fidelity enum instead of importing it"),
    ("KISS-OPS-6.17-0001",
     "a sub-standard re-forks the compute-fidelity (MathPrecision) enum instead of importing it"),
    ("KISS-CONSUME-6.4-0001",
     "a refusal-taxonomy restatement drops/renames one of the four categories"),
    ("KISS-CONTRACT-6.2-0001",
     "the universal seven-section core drifts in an owner clause or a restated list"),
    ("KISS-CONTRACT-6.11-0004",
     "the seven-section document order drifts from the §6.2-0001 core"),
    ("KISS-OPS-6.11-0011",
     "the reduce_axes four-category set drifts between Ops §6.11-0011 and Classify §6.7-0005"),
    ("KISS-ANNOUNCE-7.2-0003",
     "a FEAT capability bit's name drifts between the §7.2 definition and the §7.2-0003 clause"),
]


def main():
    ap = argparse.ArgumentParser(description="KISS shared-enumeration consistency lint")
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--emit-coverage", action="store_true",
                    help="print the clause IDs this lint enforces (clause<TAB>note), "
                         "for tools/kiss_trace.py to count as lint-backed")
    args = ap.parse_args()
    here = os.path.dirname(os.path.abspath(__file__))
    spec_dir = args.spec_dir or os.path.join(os.path.dirname(here), "spec")

    if args.emit_coverage:
        for cid, note in COVERS:
            print(f"{cid}\t{note}")
        return 0

    result = check(spec_dir)
    if isinstance(result, list):  # fatal
        print("KISS table lint — FATAL")
        for v in result:
            print(f"  - {v}")
        return 1
    violations, auth = result

    print("KISS shared-enumeration lint  —  " + spec_dir)
    print("=" * 68)
    print(f"  dtype set (owner KISS-CLASSIFY-6.1-0001): {len(auth)} tokens")
    print(f"    {' '.join(auth)}")
    print("  + determinism, MathPrecision, refusal-taxonomy and seven-section-core enums")
    print("-" * 68)
    if violations:
        print(f"  DRIFT — {len(violations)} enumeration(s) disagree with the owner:")
        for v in violations:
            print(f"      - {v}")
        print("-" * 68)
        print("  RESULT: DRIFT FOUND")
        return 1
    print("  every full dtype table and anchor list agrees with the owner.")
    print("  RESULT: CLEAN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
