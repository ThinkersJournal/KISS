# KISS RFC — a shape-expression vocabulary as the shape-side oracle

**RFC:** (number to be assigned on filing to ThinkersJournal) · **Status:** Draft — **held, propose-first** (pending the premise correction in §9) · **Date:** 2026-07-18
**Affects:** KISS-Ops §6 (new §6.20) and KISS-Contract §6.4 (new §6.4-0011 tie to §6.5 Interface). Complements §6.4-0006 (value oracle), §6.4-0009 (op_dag node schema), §6.12 (scalar-source leaves), §6.19 (canonical OpAttrs serialization).
**Category:** Standards-track, backward-compatible, additive.

## Summary

KISS pins each op's **value** behaviour — §6.13 gives every non-primitive op a reference decomposition, and KISS-Contract §6.4-0006 makes the fully-lowered primitive form the **value oracle** for a kernel under its determinism class. KISS pins **no companion for shape**: nothing binds a contract's declared output `rank`/extents (§6.5 Interface) to its operands' shapes via the op's semantics, so the return-contract shape check has no evaluator and a shape-inconsistent contract is not caught.

This RFC adds that companion: a small, closed **shape-expression vocabulary** (§6.20), its **evaluator**, and its **§6.19-canonical serialization**, plus a KISS-Contract clause (§6.4-0011) making the op's shape rule the **shape-side oracle** — the shape analog of §6.4-0006. One shape-semantics layer serves both the op DAG (§2.3 / §6.4) and the return contract (§5-facing check, realized at §6.5).

## Motivation

The abstraction **output-shape = f(operand shapes, attrs)** is already how KISS ops work: an axis-based `reduce` drops its `reduce_axes`, a `matmul` derives `[M,N]` from its M/N/K axis roles (§6.6-0016) and K-reduction (§6.13), elementwise is a broadcast of its operands. But KISS never writes that function down as a checkable rule, so:

- A verifier **cannot check** a contract's declared output shape. §6.4-0006 checks *values*; there is no clause that says the Interface output shape MUST equal what the op's semantics produce from the operand extents. A contract could declare `rank = 3` for a non-keepdim single-axis `reduce` over a rank-3 input (which is rank-2) and no KISS clause would reject it.
- A **decomposition author** has no shared surface for the two irreducible shape parameters — a broadcast target and a slice/iota offset — that a portable op-DAG needs when it expresses these from the live shape rather than a baked constant.

Both need the same thing: a shape expressed **relative to operand shapes**, evaluable against concrete inputs, and serializable so it is byte-stable.

**Scope note (what this is *not*).** A KISS contract is **monomorphized per `structure_key`** — it is already specialized to a concrete shape class, and the Interface `rank` is a compile-time constant. So this RFC does **not** make the return contract polymorphic; the polymorphism KISS wants already lives in the op DAG (§6.4) and the Classify `structure_key` abstraction. The value here is (a) a machine-checkable **shape oracle** for the Interface-vs-Semantics tie, and (b) a shared, portable surface for op-DAG interior-node shapes. This is a smaller, sharper claim than a "polymorphic return contract."

## The vocabulary (§6.20-0002)

```
ShapeExpr := SameAs(operand)                      // the operand's whole shape
DimExpr   := Extent(operand, axis)                // the size of the operand's axis
           | Const(i64)
           | Param(field)                          // a value from the op's declared params
           | DimExpr BinOp DimExpr                  // BinOp ∈ {+, −, ×, ÷}; ÷ is floor division

axis      := signed integer  (−1 = last; resolved against the operand's rank at eval)
operand   := a positional operand index  (KISS-Classify canonical operand order)
```

- **Positional operand references.** An op_dag interior node carries no operand-role tuple (§6.4-0009), so the shape oracle references operands **positionally**. A role name is a KISS-Grammar/Contract surface alias defined by the position mapping — never a second wire form.
- **Reserved.** `Reduce(operand, axis, keepdim)`, `WithDim(operand, axis, DimExpr)`, and `Dims([DimExpr, …])` are reserved (tags allocated, never emitted, rejected by a reader) pending extension-registry promotion (umbrella §6.4). The core `SameAs` + `DimExpr` suffices because keepdim-reductions and rank-inserting reshapes are already-polymorphic primitives (`reduce{…,keepdim}`, `unsqueeze`) whose shape the oracle derives from their attrs (§6.20-0007).

## Evaluator contract (§6.20-0003 / -0004)

- **Input:** the concrete shapes (and, for `Param`, the param values) of the node's operands.
- **Output:** a concrete shape / dimension.
- **Axis resolution:** a negative axis is `rank + axis`; an axis outside `[−rank, rank)` is a typed decline.
- **`÷` is floor division** (toward −∞); a `÷` by zero is a typed decline. A producer relying on exact division (e.g. an even head dim) owns that invariant.
- **Symbolic extents → surfaced gap.** If an operand extent is symbolic / data-dependent, the expression resolves to a surfaced gap — never a decline, never a panic — and the gap propagates through arithmetic and through a whole-shape `SameAs`. A consumer surfaces it as an opaque-op / telemetry gap.

## Layer boundary — shapes vs. values

`ShapeExpr`/`DimExpr` describe **shapes** only. A value a decomposition needs as an **operand** — e.g. a reduction divisor equal to an axis-extent product — is not a shape descriptor; it is a **scalar-source leaf** inside the op body (KISS-Ops §6.12: `extent(axis)`, and `reduced_count` = the product of extents over the `reduce_axes` set — this standard's Mean divisor, §6.12-0001). The two "extent" notions share the signed-axis convention: `DimExpr::Extent(op, axis)` is a single-axis **shape** parameter; the value-side `reduced_count` is the product of the reduced axes as a **runtime value**; a multi-axis product on the shape side is `Extent(op,a) × Extent(op,b)`. Keeping the boundary explicit stops a shape rule and an operand value from being confused across the seam.

## Serialization (§6.20-0005)

A shape expression serializes as a recursive, tag-prefixed, definite-length-prefixed positional blob under the §6.19 canonical discipline: a one-byte tag (`0` reserved, §6.19-0006; `SameAs=0x01`, `Extent=0x02`, `Const=0x03`, `Param=0x04`, `Add=0x05`, `Sub=0x06`, `Mul=0x07`, `Div=0x08`; reserved `Reduce=0x09`/`WithDim=0x0A`/`Dims=0x0B`), fixed-width LE fields (§6.19-0007), and each child expression `u16`-LE length-prefixed (§6.19-0010). This keeps a shape-bearing blob hashable and byte-comparable under the shared canonicalization; a reader declines a malformed blob with a typed decline, never a panic (§6.20-0006).

## Normative realization (drafted in this branch)

| Clause | Requirement | Conformance test |
|---|---|---|
| KISS-OPS-6.20-0001 | every op has a shape rule; companion to the §6.4-0006 value oracle | `test_shape_rule_exists_and_matches` |
| KISS-OPS-6.20-0002 | the closed vocabulary; positional operand refs; reserved constructors | `test_shape_expr_vocabulary_eval` |
| KISS-OPS-6.20-0003 | axis resolution; floor `÷`; divide-by-zero decline | `test_shape_expr_axis_and_floordiv` |
| KISS-OPS-6.20-0004 | symbolic extent → surfaced gap, never decline/panic | `test_shape_expr_symbolic_gap` |
| KISS-OPS-6.20-0005 | §6.19-canonical serialization; byte-deterministic | `test_shape_expr_serialization_golden` |
| KISS-OPS-6.20-0006 | typed-decline reader; round-trip | `test_shape_expr_decode_declines` |
| KISS-OPS-6.20-0007 | primitive-floor shape rules (elementwise `SameAs`, reduce drop/keepdim, DimExpr offsets) | `test_shape_expr_primitive_floor_rules` |
| KISS-CONTRACT-6.4-0011 | Interface output shape MUST equal the op's shape rule; the shape-side oracle | `test_contract_output_shape_consistency` |

The reference evaluator + serializer is `conformance/src/shape_expr.rs`; the golden/decline/behaviour vectors are `conformance/tests/shape_expr.rs`. All eight clauses are harness-backed (`tools/kiss_trace.py` reports them backed; `cargo test` green).

## Relationship to adjacent sections

- **§6.4-0006 value oracle:** §6.20 is its shape-side companion — same "resolve the op's semantics over concrete inputs" move, applied to shape rather than value.
- **§6.4-0009 op_dag node schema:** unchanged. Nodes still carry only `{op_name, op_attrs, child_edges} | Bind`; the shape rule is a property *of the op* the node names, resolved on demand, not a new node field.
- **§6.19 OpAttrs:** the serialization reuses §6.19's canonical machinery; it does **not** add a shape field to the closed §6.19-0003 carrier set, and the OpAttrs blob stays opaque to Grammar/Contract (§6.19-0012).
- **Contraction (matmul) axis roles:** complementary. KISS-Classify §6.6-0016 M/N/K axis roles are the *contraction* descriptor that lets `matmul`'s output shape derive from its operands; a `matmul` carries axis roles, not a `ShapeExpr`. Both sit under **output-shape = f(operand shapes, attrs)**.

## Backward compatibility

Purely additive. No existing clause changes; no wire-format break to §6.4-0009 or §6.19 beyond the additive shape-expression blob. A consumer that does not implement the evaluator degrades as today (it does not run the §6.4-0011 shape check). All existing conformance tests remain green.

## Provenance and the corrected premise (§9)

This RFC reframes Fuel's outreach draft `kiss-rfc-shape-rule-expression-vocabulary.md` (2026-07-18) and the parallel `baracuda-shape-expression-grammar-ask.md`. Those drafts attribute the field `OutputDesc.shape_rule` (with `same_as(role)` / `from_params(...)`) to KISS-Contract §5. **That field is a Fuel FKC field** (`fuel-dispatch/src/fkc/schema.rs`), already parsed and evaluated on Fuel's side; KISS-Contract has no `OutputDesc` and no `shape_rule`, and its §5 is *Conventions*. The KISS-side gap is not "an unevaluated §5 string" but the **missing shape oracle** described above. This RFC is therefore **held** (propose-first) until that premise is corrected in the Fuel drafts (see `../Fuel/docs/outreach/kiss-shape-expression-rfc-reply.md`); on correction it routes through umbrella §7.2 to the KISS-Ops and KISS-Contract editors-of-record with cosignatory comment. The vocabulary and the shape/value boundary carry over unchanged; only the KISS-side framing is corrected.

## Open questions

1. Is `SameAs` + `DimExpr` sufficient, or does a real decomposition force `Reduce`/`WithDim` into the shared surface? (KISS read: sufficient; keep reserved.)
2. Should the §6.4-0011 tie extend to a **full** primitive-floor shape-rule table (every op), or stay at the representative + irreducible-case coverage drafted here until a consumer needs more?
3. Spelling reconciliation: pin `DimExpr::Extent` and the §6.12 `reduced_count`/`reduce_axes` anchor to one signed-axis convention in the text, so "shared axis convention" is literally one anchor.
