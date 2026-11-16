#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""
kiss_op_decomp.py — the per-op resolution-to-the-primitive-floor table, GENERATED from spec/ops.md.

`conformance/corpus/op_decomposition.json` is the answer a Contract resolver (KISS-CONTRACT-6.4-0005) must
reach for every op, and the outcome vectors a resolver (6.4-0010) must reproduce:

  ops[op].kind       `primitive` (the §6.3 floor) or `non_primitive` (a §6.13 or §6.18 row)
  ops[op].level      0 for a floor op, else 1 + the highest level among its direct references
                     (strictly decreasing along every edge: the termination guarantee, §6.14-0002)
  ops[op].direct     the ops its reference decomposition calls (§6.13 / §6.18 first backtick span)
  ops[op].floor      the primitive-floor ops its full resolution reaches (sorted; the closure)
  outcomes[]         {op, unavailable, outcome, missing?}: `resolved` when the op's chain reaches the floor
                     with every needed contract available; `deferred` (naming the first missing op in
                     sorted order) when a contract the chain needs is unavailable (6.4-0010 b); `decline`
                     when the name is not a KISS-Ops op, so no decomposition reaches the floor (6.4-0010 a).

Nothing is authored: the graph is read from the ops.md tables by kiss_ops.py, whose own checker already
forbids a cycle, a floor row or an unresolved reference; the generator re-checks and refuses to emit
otherwise. Gated by CI as op_facts.json is: regenerate, `git diff --exit-code`.

Usage:
  python tools/kiss_op_decomp.py             # write conformance/corpus/op_decomposition.json
  python tools/kiss_op_decomp.py --stdout    # print it (bytes, LF)
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kiss_ops as K  # noqa: E402

SCHEMA = "kiss-op-decomposition-v1"
CLAUSES = ["KISS-CONTRACT-6.4-0005", "KISS-CONTRACT-6.4-0010", "KISS-OPS-6.3-0001", "KISS-OPS-6.13-0001",
           "KISS-OPS-6.14-0002", "KISS-OPS-6.18-0002"]
NOT_A_KISS_OP = "frobnicate"  # a name no KISS-Ops table lists (the op_dag decline vectors use it too)


def _graph(spec_dir):
    text = open(os.path.join(spec_dir, "ops.md"), encoding="utf-8").read().replace("\r\n", "\n")
    manifest = K.build_manifest(spec_dir)
    primitive = set(K.sec27_primitive(text))
    graph = dict(K.decomposition_graph(text))
    # A §6.13 row whose body is prose naming one op (`avg_pool` -> `reduce_mean`, `im2col` -> a structured
    # `gather`, §6.13-0009) has no call syntax, so kiss_ops reads zero references and its checker passes it
    # vacuously. Here the row's FIRST backtick span is its one reference, and it must be an op.
    region = K.between(text, "### 6.13 Reference decompositions", "\n- **KISS-OPS-6.13-0001**")
    for cells in K.table_cells(region):
        if len(cells) >= 4 and (o := K._op(cells[0])) and not graph[o]:
            m = re.search(r"`([a-z][a-z0-9_]+)`", cells[3])
            graph[o] = [m.group(1)] if m else []
    graph.update(K.complex_decomposition_graph(text))
    problems = []
    K.check_decomposition_graph(graph, primitive, problems)
    for o in manifest["all_ops"]:
        if o not in primitive and o not in graph:
            problems.append(f"{o}: a non-primitive op with no reference decomposition row")
    if problems:
        raise SystemExit("kiss_op_decomp: spec/ops.md decomposition graph is unsound:\n  - " + "\n  - ".join(problems))
    return manifest["all_ops"], primitive, graph


def _resolve(op, primitive, graph, memo):
    """(level, floor closure) of `op`, memoised; the graph is acyclic (checked above)."""
    if op in memo:
        return memo[op]
    if op in primitive:
        memo[op] = (0, [op])
        return memo[op]
    level, floor = 0, set()
    for r in graph[op]:
        lv, fl = _resolve(r, primitive, graph, memo)
        level, floor = max(level, lv), floor | set(fl)
    memo[op] = (level + 1, sorted(floor))
    return memo[op]


def _closure(op, primitive, graph):
    """Every non-primitive op whose contract a resolver must read to resolve `op` (op itself included)."""
    seen, stack = set(), [op]
    while stack:
        o = stack.pop()
        if o in primitive or o in seen:
            continue
        seen.add(o)
        stack.extend(graph[o])
    return sorted(seen)


def build(spec_dir):
    all_ops, primitive, graph = _graph(spec_dir)
    memo, ops = {}, {}
    for o in all_ops:
        level, floor = _resolve(o, primitive, graph, memo)
        ops[o] = {
            "kind": "primitive" if o in primitive else "non_primitive",
            "level": level,
            "direct": [] if o in primitive else sorted(graph[o]),
            "floor": floor,
        }
    outcomes = [{"op": NOT_A_KISS_OP, "unavailable": [], "outcome": "decline"}]
    deep = max((o for o in all_ops if o not in primitive), key=lambda o: (ops[o]["level"], o))
    shallow = min((o for o in all_ops if o not in primitive), key=lambda o: (ops[o]["level"], o))
    for o in sorted({shallow, deep, "matmul", "softmax"} & set(all_ops)):
        need = _closure(o, primitive, graph)
        outcomes.append({"op": o, "unavailable": [], "outcome": "resolved"})
        outcomes.append({"op": o, "unavailable": [need[0]], "outcome": "deferred", "missing": need[0]})
        if need[-1] != need[0]:
            outcomes.append({"op": o, "unavailable": [need[-1]], "outcome": "deferred", "missing": need[-1]})
    prim = sorted(primitive)[0]
    outcomes.append({"op": prim, "unavailable": [prim], "outcome": "resolved"})  # a floor op needs no contract
    return {"schema": SCHEMA, "generated_from": "tools/kiss_op_decomp.py (reads spec/ops.md)",
            "clauses": CLAUSES, "ops": ops, "outcomes": outcomes}


def render(spec_dir):
    return (json.dumps(build(spec_dir), indent=2) + "\n").encode("utf-8")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[1])
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--stdout", action="store_true")
    args = ap.parse_args()
    spec_dir = args.spec_dir or os.path.join(os.path.dirname(HERE), "spec")
    data = render(spec_dir)
    if args.stdout:
        sys.stdout.buffer.write(data)
        return 0
    out = os.path.join(os.path.dirname(spec_dir), "conformance", "corpus", "op_decomposition.json")
    with open(out, "wb") as f:
        f.write(data)
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
