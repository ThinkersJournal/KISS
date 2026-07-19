#!/usr/bin/env python3
"""
kiss_wire.py — the KISS wire-format spec<->reference-binding lint.

A document lint, sibling of tools/kiss_vocab.py. Where kiss_vocab binds the Classify
and Ops VOCABULARIES (op-family codes, OpAttrs enums, structure_key sub-vocabularies)
to the reference codec, this binds the WIRE-FORMAT constants of the framed
sub-standards — Announce, Contract, Grammar — to the way the reference harness
implements them: the frame magics, the pinned structural limits, the header tokens,
and the decline-code enum.

The teeth are the same shape as kiss_vocab's: each value is pinned in exactly one
normative clause and bound once in conformance/src/*.rs, so a cross-document lint has
nothing to disagree with, but the spec and its reference reader CAN drift — change the
spec's `MAX_PROFILES` cap or the `KISC` magic without the reader (or vice versa) and
the reference reader silently accepts or rejects the wrong bytes. This lint is the
tooth for exactly that, per wire-format value.

Four check groups:

  1. pinned integers   MAX_RANK / ENVELOPE_LEN / MAX_PROFILES / MAX_STRUCTURE_KEY_LEN /
                       MAX_AVAILABILITY_RECORDS  (spec clause value == harness const)
  2. contract header   the `KISC` magic, the `kiss-contract` kind, the version `1`
  3. grammar region    the `KGRM` magic, the consumers enum, the node-kind enum
  4. decline codes     the six §6.4-0009 decline_code values, across Announce, Synth,
                       and the reference decline_code module

Each value is extracted from BOTH sides and compared; a drift in either direction
fails. Exit status 0 on agreement, 1 on drift. Stdlib only.

Usage:
  python tools/kiss_wire.py                  # check
  python tools/kiss_wire.py --emit-coverage  # print the clauses it enforces
"""
from __future__ import annotations

import argparse
import os
import re
import sys


def _read(path):
    return open(path, encoding="utf-8").read() if os.path.exists(path) else None


def clause_body(text, sub, short):
    """Body of a `- **KISS-<SUB>-<short>** — ...` clause, up to the next clause or heading."""
    m = re.search(r"\*\*KISS-" + sub + r"-" + re.escape(short) +
                  r"\*\*(.*?)(?=\n\s*-\s+\*\*KISS|\n#)", text, re.S)
    return m.group(1) if m else None


def _evalnum(s):
    """Evaluate a pinned integer written as a decimal (`4096`), a power (`2^20`), or a
    Rust shift (`1 << 20`). Returns int or None."""
    s = s.strip()
    m = re.fullmatch(r"(\d+)", s)
    if m:
        return int(m.group(1))
    m = re.fullmatch(r"(\d+)\s*\^\s*(\d+)", s)
    if m:
        return int(m.group(1)) ** int(m.group(2))
    m = re.fullmatch(r"(\d+)\s*<<\s*(\d+)", s)
    if m:
        return int(m.group(1)) << int(m.group(2))
    return None


# ---- (1) pinned integers -----------------------------------------------------
# (backs_clause, spec_doc, clause_short, spec_value_regex, harness_file, harness_regex).
# The spec regex runs against the CLAUSE BODY only, so the many bare numbers in the
# announce §6.1 offset/size table never interfere; the harness regex keys on the unique
# const name. Values are evaluated (2^n / 1<<n) before comparison.
PINNED_INTS = [
    ("KISS-CLASSIFY-6.4-0001", "classify", "CLASSIFY", "6.4-0001",
     r"`MAX_RANK` MUST equal `(\d+)`", "structure_key.rs",
     r"MAX_RANK\s*:\s*u32\s*=\s*(\d+)"),
    ("KISS-ANNOUNCE-6.1-0001", "announce", "ANNOUNCE", "6.1-0001",
     r"exactly (\d+) bytes", "announce.rs",
     r"ENVELOPE_LEN\s*:\s*usize\s*=\s*(\d+)"),
    ("KISS-ANNOUNCE-6.1-0007", "announce", "ANNOUNCE", "6.1-0007",
     r"`<=\s*(\d+)`", "announce.rs",
     r"MAX_PROFILES\s*:\s*usize\s*=\s*(\d+)"),
    ("KISS-ANNOUNCE-6.3-0009", "announce", "ANNOUNCE", "6.3-0009",
     r"MAX_STRUCTURE_KEY_LEN\s*=\s*(\d+)", "announce.rs",
     r"MAX_STRUCTURE_KEY_LEN\s*:\s*u32\s*=\s*(\d+)"),
    ("KISS-ANNOUNCE-6.3-0010", "announce", "ANNOUNCE", "6.3-0010",
     r"MAX_AVAILABILITY_RECORDS\s*=\s*(2\^\d+)", "announce.rs",
     r"MAX_AVAILABILITY_RECORDS\s*:\s*u32\s*=\s*([0-9]+\s*<<\s*[0-9]+)"),
]


def check_pinned_ints(spec_dir, src_dir):
    out, n = [], 0
    for backs, doc, sub, short, spec_re, hfile, h_re in PINNED_INTS:
        stext = _read(os.path.join(spec_dir, doc + ".md"))
        htext = _read(os.path.join(src_dir, hfile))
        if stext is None or htext is None:
            out.append(f"§{short}: could not read {doc}.md or {hfile}")
            continue
        body = clause_body(stext, sub, short)
        sm = re.search(spec_re, body) if body else None
        hm = re.search(h_re, htext)
        if not sm:
            out.append(f"§{short}: pinned value not found in the clause")
            continue
        if not hm:
            out.append(f"§{short}: harness const not found in {hfile}")
            continue
        sval, hval = _evalnum(sm.group(1)), _evalnum(hm.group(1))
        n += 1
        if sval != hval:
            out.append(f"§{short} pinned-int drift: spec {sval} vs harness {hval}")
    return out, n


# ---- (2) contract header tokens ----------------------------------------------
def check_contract_header(spec_dir, src_dir):
    out, n = [], 0
    stext = _read(os.path.join(spec_dir, "contract.md"))
    htext = _read(os.path.join(src_dir, "contract.rs"))
    if stext is None or htext is None:
        return ["contract: could not read contract.md or contract.rs"], 0
    # magic: spec §6.11-0002 "(ASCII `KISC`)" vs harness b"KISC"
    sm = re.search(r"ASCII `([A-Z]{4})`", clause_body(stext, "CONTRACT", "6.11-0002") or "")
    hm = re.search(r'b"([A-Z]{4})"', htext)
    if sm and hm:
        n += 1
        if sm.group(1) != hm.group(1):
            out.append(f"contract magic drift: spec `{sm.group(1)}` vs harness b\"{hm.group(1)}\"")
    else:
        out.append("contract magic: not found on both sides")
    # kind: spec §6.1-0007 `kiss-contract` vs harness read_document guard
    sm = re.search(r"exact token `([a-z-]+)`", clause_body(stext, "CONTRACT", "6.1-0007") or "")
    hm = re.search(r'kind\s*!=\s*"([a-z-]+)"', htext)
    if sm and hm:
        n += 1
        if sm.group(1) != hm.group(1):
            out.append(f"contract kind drift: spec `{sm.group(1)}` vs harness \"{hm.group(1)}\"")
    else:
        out.append("contract kind: not found on both sides")
    # version: spec §6.1-0008 decimal token `1` vs harness guard
    sm = re.search(r"exact decimal\s+token `(\d+)`", clause_body(stext, "CONTRACT", "6.1-0008") or "")
    hm = re.search(r'version\s*!=\s*"(\d+)"', htext)
    if sm and hm:
        n += 1
        if sm.group(1) != hm.group(1):
            out.append(f"contract version drift: spec `{sm.group(1)}` vs harness \"{hm.group(1)}\"")
    else:
        out.append("contract version: not found on both sides")
    return out, n


# ---- (3) grammar region magic + enums ----------------------------------------
def _spec_byte_enum(body):
    """{byte -> UPPER name} from clause forms like `0x00 = INTERIOR`, `0x01 = Op`."""
    out = {}
    for m in re.finditer(r"`?0x0?([0-9a-fA-F]) = (\w+)`?", body or ""):
        out[int(m.group(1), 16)] = m.group(2).upper()
    return out


def _harness_byte_enum(htext, prefix):
    """{byte -> UPPER name} from `pub const <PREFIX>_<NAME>: u8 = 0x0N;`, prefix stripped."""
    out = {}
    for m in re.finditer(r"pub const " + prefix + r"_(\w+)\s*:\s*u8\s*=\s*0x0?([0-9a-fA-F]);", htext):
        out[int(m.group(2), 16)] = m.group(1).upper()
    return out


def check_grammar_wire(spec_dir, src_dir):
    out, n = [], 0
    stext = _read(os.path.join(spec_dir, "grammar.md"))
    htext = _read(os.path.join(src_dir, "grammar.rs"))
    if stext is None or htext is None:
        return ["grammar: could not read grammar.md or grammar.rs"], 0
    # magic: §6.8-0013 "(ASCII `KGRM`)" vs harness *b"KGRM"
    sm = re.search(r"ASCII `([A-Z]{4})`", clause_body(stext, "GRAMMAR", "6.8-0013") or "")
    hm = re.search(r'MAGIC\s*:\s*\[u8;\s*4\]\s*=\s*\*b"([A-Z]{4})"', htext)
    if sm and hm:
        n += 1
        if sm.group(1) != hm.group(1):
            out.append(f"grammar magic drift: spec `{sm.group(1)}` vs harness *b\"{hm.group(1)}\"")
    else:
        out.append("grammar magic: not found on both sides")
    # consumers enum (§6.8-0005) vs CONSUMERS_* consts
    sc = _spec_byte_enum(clause_body(stext, "GRAMMAR", "6.8-0005"))
    hc = _harness_byte_enum(htext, "CONSUMERS")
    if sc and hc:
        n += 1
        if sc != hc:
            out.append(f"grammar consumers enum drift: spec {sc} vs harness {hc}")
    else:
        out.append("grammar consumers enum: not found on both sides")
    # node-kind enum (§6.8-0010) vs KIND_* consts
    sk = _spec_byte_enum(clause_body(stext, "GRAMMAR", "6.8-0010"))
    hk = _harness_byte_enum(htext, "KIND")
    if sk and hk:
        n += 1
        if sk != hk:
            out.append(f"grammar node-kind enum drift: spec {sk} vs harness {hk}")
    else:
        out.append("grammar node-kind enum: not found on both sides")
    return out, n


# ---- (4) decline-code enum ---------------------------------------------------
def _decline_tables(text):
    """Every {value -> NAME} map from a markdown decline table (rows
    `| \\`0x0000000N\\` | \\`NAME\\` | ... |`), dropping the reserved 0x00 row."""
    maps = []
    cur, in_tbl = {}, False
    for line in (text or "").splitlines():
        m = re.match(r"\s*\|\s*`0x0000000([0-9a-f])`\s*\|\s*`?([A-Za-z_]+)`?\s*\|", line)
        if m:
            in_tbl = True
            val, name = int(m.group(1), 16), m.group(2)
            if name.isupper():           # skip the lowercase `reserved` row
                cur[val] = name
        elif in_tbl:
            if cur:
                maps.append(cur)
            cur, in_tbl = {}, False
    if cur:
        maps.append(cur)
    return maps


def _harness_decline(announce_rs):
    """{value -> NAME} from the `pub mod decline_code { ... }` block only, so the
    surrounding SAVL/CYRQ/CRSP/CDEC tag consts never pollute the map."""
    m = re.search(r"pub mod decline_code\s*\{(.*?)\n\}", announce_rs, re.S)
    if not m:
        return None
    out = {}
    for cm in re.finditer(r"pub const (\w+)\s*:\s*u32\s*=\s*0x0000_000([0-9a-fA-F]);", m.group(1)):
        out[int(cm.group(2), 16)] = cm.group(1)
    return out


def check_decline_codes(spec_dir, src_dir):
    out = []
    ann = _read(os.path.join(spec_dir, "announce.md"))
    syn = _read(os.path.join(spec_dir, "synth.md"))
    rs = _read(os.path.join(src_dir, "announce.rs"))
    if ann is None or rs is None:
        return ["decline: could not read announce.md or announce.rs"], 0
    # owner = the most complete announce.md decline table (the §6.4-0009 one)
    ann_maps = _decline_tables(ann)
    owner = max(ann_maps, key=len) if ann_maps else {}
    if len(owner) != 6:
        out.append(f"decline owner: expected 6 codes in the §6.4-0009 table, found {len(owner)}")
        return out, 0
    # every announce + synth decline table must agree with the owner on shared codes
    for label, text in (("announce.md", ann), ("synth.md", syn or "")):
        for tbl in _decline_tables(text):
            for val, name in tbl.items():
                if owner.get(val) != name:
                    out.append(f"{label} decline table: 0x{val:08x}=`{name}` "
                               f"disagrees with owner `{owner.get(val)}`")
    # harness decline_code module must equal the owner
    h = _harness_decline(rs)
    if h is None:
        out.append("decline: no `pub mod decline_code` found in announce.rs")
    elif h != owner:
        out.append(f"decline codec drift: harness {h} vs owner {owner}")
    return out, 1


COVERS = [
    ("KISS-CLASSIFY-6.4-0001", "MAX_RANK drifts between §6.4-0001 and structure_key.rs"),
    ("KISS-ANNOUNCE-6.1-0001", "the 56-byte envelope size drifts between §6.1-0001 and ENVELOPE_LEN"),
    ("KISS-ANNOUNCE-6.1-0007", "the profiles cap 16 drifts between §6.1-0007 and MAX_PROFILES"),
    ("KISS-ANNOUNCE-6.3-0009", "MAX_STRUCTURE_KEY_LEN drifts between §6.3-0009 and announce.rs"),
    ("KISS-ANNOUNCE-6.3-0010", "MAX_AVAILABILITY_RECORDS drifts between §6.3-0010 and announce.rs"),
    ("KISS-CONTRACT-6.1-0007", "the recognized kind `kiss-contract` drifts between §6.1-0007 and the reader"),
    ("KISS-CONTRACT-6.1-0008", "the schema version token `1` drifts between §6.1-0008 and the reader"),
    ("KISS-GRAMMAR-6.8-0013", "the region magic `KGRM` drifts between §6.8-0013 and grammar.rs"),
    ("KISS-ANNOUNCE-6.4-0009", "a decline_code value drifts between the §6.4-0009 owner and the reference codec"),
    ("KISS-SYNTH-6.6-0002", "a Synth-reused decline_code drifts from the §6.4-0009 owner"),
]


def check(spec_dir, conf_dir):
    src_dir = os.path.join(conf_dir, "src")
    violations, counts = [], {}
    for name, fn in (("pinned-ints", check_pinned_ints),
                     ("contract-header", check_contract_header),
                     ("grammar-wire", check_grammar_wire),
                     ("decline-codes", check_decline_codes)):
        v, n = fn(spec_dir, src_dir)
        violations += v
        counts[name] = n
    return violations, counts


def main():
    ap = argparse.ArgumentParser(description="KISS wire-format spec<->reference-binding lint")
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--conformance-dir", default=None)
    ap.add_argument("--emit-coverage", action="store_true",
                    help="print the clause IDs this lint enforces (clause<TAB>note)")
    args = ap.parse_args()
    here = os.path.dirname(os.path.abspath(__file__))
    root = os.path.dirname(here)
    spec_dir = args.spec_dir or os.path.join(root, "spec")
    conf_dir = args.conformance_dir or os.path.join(root, "conformance")

    if args.emit_coverage:
        for cid, note in COVERS:
            print(f"{cid}\t{note}")
        return 0

    violations, counts = check(spec_dir, conf_dir)

    print("KISS wire-format spec<->binding lint")
    print("=" * 68)
    for name, n in counts.items():
        print(f"  {name:<16} {n} value(s) checked")
    print("-" * 68)
    if violations:
        print(f"  DRIFT — {len(violations)} disagreement(s):")
        for v in violations:
            print(f"      - {v}")
        print("  RESULT: DRIFT FOUND")
        return 1
    print("  every wire-format magic, pinned limit, header token and decline code")
    print("  agrees between the spec and the reference harness.")
    print("  RESULT: CLEAN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
