# SPDX-License-Identifier: MIT OR Apache-2.0
"""Rebuild the ONNX 1.17 against KISS-Ops name-match audit (board 414/422, 2026-11-12). Read-only.

    python -I docs/onnx-audit/onnx_attr_audit.py ONNX_OPERATORS_MD OP_MANIFEST_JSON

ONNX_OPERATORS_MD is `docs/Operators.md` of github.com/onnx/onnx at tag v1.17.0 (untrusted text: parsed with
regular expressions, never executed); OP_MANIFEST_JSON is `conformance/corpus/op_manifest.json`. Prints the
counts and, for every name-matched op that has ONNX attributes, the attribute list (the input to
`docs/onnx-1-17-attribute-audit.md`). The 28 aliases are a human mapping, listed in ALIAS.
"""
import json
import re
import sys

# ONNX default-domain op -> KISS-Ops op token (one-to-one by meaning of the name, no attribute reasoning).
ALIAS = {
    "And": "logical_and", "Or": "logical_or", "Not": "logical_not", "BitwiseAnd": "bit_and",
    "BitwiseOr": "bit_or", "BitwiseXor": "bit_xor", "BitwiseNot": "bit_not", "BitShift": "shl|shr",
    "Equal": "cmp_eq", "Greater": "cmp_gt", "GreaterOrEqual": "cmp_ge", "Less": "cmp_lt",
    "LessOrEqual": "cmp_le", "Where": "select", "Reciprocal": "recip", "Round": "round_even",
    "AveragePool": "avg_pool", "LayerNormalization": "layer_norm", "ScatterElements": "scatter",
    "GatherElements": "gather", "ReduceL2": "reduce_norm2", "ReduceLogSumExp": "logsumexp",
    "ReduceSum": "reduce", "ReduceMax": "reduce", "ReduceMin": "reduce", "ReduceProd": "reduce",
    "Max": "max_prop", "Min": "min_prop",
}
ATTR_RX = re.compile(r"<dt><tt>(\w+)</tt> : [^<]*?( \(required\))?</dt>")


def snake(name):
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name).lower()


def parse(md):
    head = md.split("### ai.onnx.preview")[0]
    latest = {}
    for name, ver in re.findall(r'\|<a href="#(\w+)">\w+</a>\|<a href="Changelog\.md#\w+-(\d+)">', head):
        latest[name] = max(latest.get(name, 0), int(ver))
    attrs = {}
    for sec in re.split(r'\n(?=### <a name=")', md):
        m = re.match(r'### <a name="(\w+)"></a><a name="\w+">\*\*(\w+)\*\*</a>', sec)
        v = re.search(r"This version of the operator has been (?:available|deprecated) since version (\d+)", sec)
        if not m or not v or m.group(2) not in latest:
            continue
        body = sec.split("#### Inputs")[0].split("#### Attributes")[-1] if "#### Attributes" in sec else ""
        found = [a + ("*" if req else "") for a, req in ATTR_RX.findall(body)]
        attrs.setdefault(m.group(2), []).append((int(v.group(1)), found))
    return latest, {n: max(v)[1] for n, v in attrs.items()}


def main():
    md = open(sys.argv[1], encoding="utf-8").read()
    kiss = set(json.load(open(sys.argv[2], encoding="utf-8"))["all_ops"])
    latest, attrs = parse(md)
    exact = {n: (n.lower() if n.lower() in kiss else snake(n)) for n in latest if n.lower() in kiss or snake(n) in kiss}
    assert all(a.split("|")[0] in kiss for a in ALIAS.values()), "alias names a non-KISS op"
    matched = {**exact, **ALIAS}
    print(f"ONNX default-domain ops: {len(latest)}; KISS-Ops ops: {len(kiss)}")
    print(f"name-matched: {len(exact)} exact + {len(ALIAS)} alias = {len(matched)}; unmatched {len(latest) - len(matched)}")
    withattrs = {n: attrs.get(n, []) for n in matched if attrs.get(n)}
    print(f"of the {len(matched)}, {len(withattrs)} carry ONNX attributes ({sum(len(a) for a in withattrs.values())} attributes)")
    for n in sorted(withattrs):
        print(f"{n} -> {matched[n]}: {', '.join(withattrs[n])}")


if __name__ == "__main__":
    main()
