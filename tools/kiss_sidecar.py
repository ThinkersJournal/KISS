# SPDX-License-Identifier: MIT OR Apache-2.0
"""KISS-Conform clause sidecar (Appendix E; KISS-CONFORM-6.1-0007, 6.2-0004) — #144, option A.

The machine-readable sidecar `spec/sidecar/<stem>.json` is the sole authoritative clause source for
every sub-standard that HAS one. This tool does two things:

  --write STEM...   SEED a sidecar from the prose, once. The result is `generator: hand-maintained`
                    (Appendix E's non-POD tier): after seeding, the sidecar is edited, the prose is
                    not its source. Re-running --write over an existing sidecar is refused unless
                    --force, because regenerating from prose would invert 6.1-0007 (#144 option B).
  --check [STEM...] The 6.2-0004 reconciliation: the prose's clause-ID tokens (and each clause's
                    keyword) must equal the sidecar's. Any difference is DRIFT and exits 1. With no
                    STEM, every sidecar present is checked.

Two Appendix E fields are not derivable from prose; both were ruled 2026-10-15 (board 193 #462):
`determinism_class` is OPTIONAL per clause (absent = n/a, so the tool never writes it), and
`wire_abi_schema_version` is each sub-standard's own schema string where its §8 states one. Where
none is stated the field records `NOT_STATED` and is counted UNSET; nothing is guessed.
"""
import argparse
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
SPEC = os.path.join(ROOT, "spec")
SIDECAR = os.path.join(SPEC, "sidecar")
sys.path.insert(0, HERE)
import kiss_trace as kt  # noqa: E402

KEYWORD = re.compile(r"\b(MUST NOT|MUST|SHALL NOT|SHALL|SHOULD NOT|SHOULD|MAY NOT|MAY)\b")
# Appendix E's enumerant set; a NOT form maps onto the negated modal Appendix E lists.
KEYWORD_ENUM = {"MUST": "MUST", "MUST NOT": "MUST NOT", "SHALL": "SHALL", "SHALL NOT": "SHALL",
                "SHOULD": "SHOULD", "SHOULD NOT": "SHOULD", "MAY": "MAY", "MAY NOT": "MAY"}
NORMATIVE = {"MUST", "MUST NOT", "SHALL"}
DETERMINISM_ENUM = {"exact-byte", "ULP/tolerance", "order-invariant/nondeterministic"}
# Each sub-standard's own schema string, taken only where its §8 states a value (board 193 #462).
# Absent stems stay UNSET: announce `envelope_version` 1 (6.0-0001), classify `STRUCTURE_KEY_VERSION`
# 6 (6.4-0003), contract `contract_version` 4 (6.1-0008).
SCHEMA_STRING = {"announce": "1", "classify": "6", "contract": "4"}
# Recorded in the field itself (Appendix E rejects unknown fields): the KISS lane is the editor of
# record for every sub-standard until editors are ratified, so it resolves the ask by reading §8.
NOT_STATED = "none stated (editor: KISS lane, pending ratification)"


def prose_clauses(stem, spec_dir=None):
    """[(clause_id, keyword)] in ascending clause-ID order, read from `spec/<stem>.md`."""
    text = open(os.path.join(spec_dir or SPEC, stem + ".md"), encoding="utf-8").read()
    defs = [(m.group(1), m.start()) for m in kt.RE_DEF.finditer(text)]
    heads = [m.start() for m in kt.RE_HEAD.finditer(text)]
    bounds = sorted([p for _, p in defs] + heads + [len(text)])
    out = {}
    for cid, pos in defs:
        end = next(b for b in bounds if b > pos)
        m = KEYWORD.search(text[pos:end])
        out[cid] = KEYWORD_ENUM[m.group(1)] if m else None
    return sorted(out.items())


def build(stem):
    clauses = []
    for cid, kw in prose_clauses(stem):
        section = cid.split("-")[2]
        clauses.append({
            "clause_id": cid,
            "keyword": kw,
            "normative": (kw in NORMATIVE) if kw else None,
            "section": section,
            "status": "live",                 # no prose convention marks a retired clause yet
        })
    return {
        "clauses": clauses,
        "generator": "hand-maintained",
        "sub_standard_token": "KISS-" + stem.upper(),
        "wire_abi_schema_version": SCHEMA_STRING.get(stem, NOT_STATED),
    }


def dumps(doc):
    """Appendix E serialization: sorted keys, no insignificant whitespace, LF."""
    return json.dumps(doc, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n"


def sidecar_path(stem, spec_dir=None):
    return os.path.join(os.path.join(spec_dir, "sidecar") if spec_dir else SIDECAR, stem + ".json")


def load(stem, spec_dir=None):
    with open(sidecar_path(stem, spec_dir), encoding="utf-8", newline="") as f:
        raw = f.read()
    return raw, json.loads(raw)


def unset_count(doc):
    return 1 if doc.get("wire_abi_schema_version") in (None, NOT_STATED) else 0


def drift(stem, spec_dir=None):
    """The 6.2-0004 reconciliation. Returns a list of human-readable drift lines (empty = none)."""
    raw, doc = load(stem, spec_dir)
    problems = []
    if raw != dumps(doc):
        problems.append("sidecar is not in Appendix E byte-deterministic form (sorted keys, no whitespace, LF)")
    if doc.get("sub_standard_token") != "KISS-" + stem.upper():
        problems.append(f"sub_standard_token {doc.get('sub_standard_token')!r} != KISS-{stem.upper()}")
    side = {c["clause_id"]: c.get("keyword") for c in doc["clauses"]}
    if [c["clause_id"] for c in doc["clauses"]] != sorted(side):
        problems.append("clauses are not in ascending clause-ID order, or an ID repeats")
    for c in doc["clauses"]:
        # Appendix E: OPTIONAL, so absent is valid; a present value must be an enumerant.
        if "determinism_class" in c and c["determinism_class"] not in DETERMINISM_ENUM:
            problems.append(f"{c['clause_id']}: determinism_class {c['determinism_class']!r} is not an "
                            f"enumerant (omit the field for n/a)")
    prose = dict(prose_clauses(stem, spec_dir))
    for cid in sorted(set(prose) - set(side)):
        problems.append(f"{cid}: in the prose, not in the sidecar (the sidecar is authoritative: add it there)")
    for cid in sorted(set(side) - set(prose)):
        problems.append(f"{cid}: in the sidecar, not in the prose (retire it in the sidecar, or restore the prose)")
    for cid in sorted(set(side) & set(prose)):
        if side[cid] != prose[cid]:
            problems.append(f"{cid}: keyword sidecar={side[cid]!r} prose={prose[cid]!r}")
    return problems


def present():
    if not os.path.isdir(SIDECAR):
        return []
    return sorted(f[:-5] for f in os.listdir(SIDECAR) if f.endswith(".json"))


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--write", nargs="+", metavar="STEM", help="seed these sidecars from prose (once)")
    ap.add_argument("--force", action="store_true", help="allow --write over an existing sidecar")
    ap.add_argument("--check", nargs="*", metavar="STEM", help="6.2-0004 drift check (default: all present)")
    a = ap.parse_args(argv)
    if a.write:
        os.makedirs(SIDECAR, exist_ok=True)
        for stem in a.write:
            if os.path.exists(sidecar_path(stem)) and not a.force:
                print(f"REFUSED {stem}: sidecar exists; --write regenerates from prose, which inverts "
                      f"6.1-0007 (#144 option B). Edit the sidecar, or pass --force deliberately.")
                return 2
            with open(sidecar_path(stem), "w", encoding="utf-8", newline="") as f:
                f.write(dumps(build(stem)))
            print(f"seeded spec/sidecar/{stem}.json")
        return 0
    if a.check is not None:
        stems = a.check or present()
        if not stems:
            print("no sidecar present: nothing to check (the declared coverage set is empty)")
            return 0
        bad = 0
        for stem in stems:
            if not os.path.isfile(sidecar_path(stem)):
                print(f"  [FAIL] {stem}: no spec/sidecar/{stem}.json")
                bad += 1
                continue
            _, doc = load(stem)
            p = drift(stem)
            tag = "FAIL" if p else " ok "
            note = ("wire_abi_schema_version: no schema string stated in the sub-standard"
                    if unset_count(doc) else f"wire_abi_schema_version {doc['wire_abi_schema_version']!r}")
            print(f"  [{tag}] {stem:<9} {len(doc['clauses']):>4} clauses; {unset_count(doc)} field(s) UNSET ({note})")
            for line in p[:10]:
                print(f"           {line}")
            if len(p) > 10:
                print(f"           ... and {len(p) - 10} more")
            bad += bool(p)
        return 1 if bad else 0
    ap.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
