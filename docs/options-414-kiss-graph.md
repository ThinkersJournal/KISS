# Options and clause skeleton: a KISS-Graph sub-standard (board 414 "C")

Status: **options note plus a clause *skeleton*; no clause is binding and no byte is defined.** Written 2026-11-12 against `origin/main`
73c341b5. Ruling (PM relaying CireSnave, 2026-11-12, verbatim): *"414: C."* = a KISS-Graph sub-standard; the PM's brief: value table, node
table, graph inputs/outputs, reuse KISS-Ops names and KISS-Classify tokens, KISS-Grammar unchanged, static shapes, no control flow, typed
refusals. Evidence for what a graph must express: `docs/onnx-1-17-attribute-audit.md` (72 name-matched ONNX ops, 22 with attributes, 7 distinct
gaps). `iota` is board 422, not decided here.

## 1. What a graph is, in one paragraph

A KISS-Graph document names **values** (each a dtype and a static shape), **nodes** (each a KISS-Ops op applied to input values, with that op's
OpAttrs, producing output values) and the graph's **inputs and outputs** (ordered lists of values). It is acyclic, every value is defined
once, and every shape and dtype is checkable from the node table alone by KISS-Ops' own shape and dtype rules. It says *what is computed*;
how a kernel is chosen for a node stays KISS-Contract/Classify's business, so a graph and a `structure_key` meet at the node: the node's op,
OpAttrs and operand dtypes/shapes are exactly what the Classify key is built from.

## 2. Decisions this note puts to CireSnave (each with a recommendation)

| # | Question | Options | Recommendation |
|---|---|---|---|
| G1 | **Encoding** | (a) a text document in the style of Contract's text framing (6.11-0002), (b) a binary blob built from the OpAttrs conventions, (c) both, one canonical | **(c) with text canonical**: a graph is read and diffed by people; the canonical bytes (for a graph hash) come from a pinned canonical text form, so there is one set of bytes |
| G2 | **Where constants live** | (a) a node `const` reusing the Grammar `const` payload rule (Grammar 6.4-0013), (b) a separate initializer table | **(a)**: one mechanism, and Grammar stays unchanged (it already types the bits of a constant) |
| G3 | **Dynamic shapes** | (a) none in v1 (static only), (b) symbolic extents over the Contract launch scalars | **(a)** per the brief; (b) is a later additive minor, the extent-expression grammar (Contract 6.6-0006) is the obvious carrier |
| G4 | **Ops not in KISS-Ops** | (a) the graph standard refuses them with a typed decline, (b) a vendor-namespaced op tier as in Ops 7.3 | **(a) for v1, (b) allowed by reference to Ops 7.3**: a graph can name a vendor op only where Ops 7.3 already allows one |
| G5 | **Relation to ONNX** | (a) KISS-Graph states no ONNX mapping, (b) an informative appendix with the mapping and the gaps | **(b)**, informative, generated from the audit so it cannot drift silently |
| G6 | **Sequencing** | (a) draft all of it, (b) start with the value/node tables and refusals, then I/O and canonical form, then the appendix | **(b)** |

## 3. Clause skeleton (section headings with the shape of each rule; IDs are placeholders `KISS-GRAPH-6.n-nnnn`)

* **6.1 Scope and non-goals.** A graph states computation over tensors of static shape using KISS-Ops ops only. It does **not** state kernel
  selection, devices, layouts of storage, scheduling, or any control flow. It is **not** an execution format.
* **6.2 Value table.** An ordered list; the position is the **value id** (dense, from 0). Each value: a KISS-Classify §6.1 **dtype token**
  (the closed 27, no new spelling) and a **static shape** (a list of non-negative integer extents; rank-0 allowed; the Ops `MAX_RANK`
  bounds it). A value has an optional UTF-8 **name** that carries no meaning (never read by a conforming reader for behaviour; compare
  KISS-CONSUME-6.4-0007, category assignment is structure-based, not name-based).
* **6.3 Node table.** An ordered list. Each node: an **op token** that is a KISS-Ops op (or a vendor op under G4), a list of **input value
  ids**, a list of **output value ids**, and the op's **OpAttrs blob byte for byte as Ops §6.19 defines it** (the graph adds no attribute
  encoding of its own). Arity, attribute completeness and the mandatory-field rules are the op's, checked by the Ops clauses, not restated.
* **6.4 Graph inputs and outputs.** Two ordered lists of value ids. An input is a value no node defines; an output is a value some node
  defines (or an input passed through, stated). A value that is neither an input nor defined by a node is a typed decline.
* **6.5 Well-formedness.** (a) every value is defined at most once (SSA); (b) a node's inputs are defined **earlier** in the node table
  (so the table is a topological order and cycles cannot be written); (c) every node's output dtype and shape equal what the KISS-Ops
  shape rule (§6.20) and dtype rule give for its inputs and OpAttrs; (d) no node is unreachable from an output unless the document says
  it is a debug value. Each failure is a typed decline, never a panic (Contract §6.1-0004, never a panic, applies).
* **6.6 No control flow, no dynamic shape.** `If`, `Loop`, `Scan`-style ops and any extent that is not a literal are refused with a typed decline.
* **6.7 Typed refusals (a closed set, each with a code).** Candidates: `unknown_op`, `unsupported_dtype` (a Classify token the op does not
  take), `dynamic_shape`, `control_flow`, `bad_arity`, `shape_mismatch`, `dtype_mismatch`, `undefined_value`, `redefined_value`,
  `cyclic_or_unordered`, `bad_attr` (carries the Ops decline), `too_deep` (a stated limit). The set is closed: an unknown code is a reader bug.
* **6.8 Canonical form and identity.** One canonical byte string per graph (G1), so a **graph hash** is well defined and two writers
  produce the same bytes; the hash covers values, nodes, inputs and outputs, not value names.
* **6.9 Conformance.** A vector family: accepted graphs with their expected value/shape/dtype tables, and one refused graph per code in
  6.7; plus a differential against the Ops reference for a small closed set of graphs (evaluated node by node). Evidence standard as
  elsewhere: two independent readers before a clause is frozen.
* **6.10 Capability and extension.** A reader declares which ops it can evaluate; a graph naming an op the reader does not know is a
  typed `unknown_op`, not a silent skip (the Announce capability model, never a new bit without CireSnave's word).
* **Appendix A (informative): ONNX 1.17.** The mapping of the 72 name-matched ops, the 7 attribute gaps, and the unmatched families, generated
  from `docs/onnx-audit/onnx_attr_audit.py` so the numbers cannot drift.

## 4. What it reuses and what it adds (so the diff against today is small)

Reused unchanged: the Classify dtype tokens; every Ops op name, attribute blob and shape/dtype rule; the Ops 7.3 vendor tier; the Contract
text-framing and typed-decline conventions; the Consume structure-not-name rule. **KISS-Grammar is not touched.** Added: the tables, the
well-formedness rules, the decline set, the canonical form, the vectors. Nothing here moves an existing clause or a `structure_key` byte.

## 5. Open items that need a ruling before text

G1 to G6 above; whether the 7 audit gaps become Ops additions (`reverse`, a ceil-mode pool, an empty-axes identity, `mul` scatter combine) or
stay refusals; and board 422 (`iota`). Fuel's and Unpopped's use of a graph (Fuel's FKC and Unpopped's IR are the likely first readers) is
a question for those lanes before the sub-standard's first draft, and is not assumed here.

— lane: kiss
