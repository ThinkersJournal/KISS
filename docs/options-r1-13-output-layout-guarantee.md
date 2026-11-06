# Options: where a kernel states the layout it writes (R1.13, KISS#316, board 378)

Status: **options with a recommendation; nothing here is a clause.** 2026-11-04, against `origin/main` 5988cf1 (conformance 0.7.10). CireSnave's instruction on board 378 (via the PM, 2026-11-04): not picked; "the most descriptive answer so nothing is blocked later"; compare Fuel FKC `return.outputs[].layout_guarantee`, what KISS states today, and what Unpopped needs, then propose the widest design.

## 1. What each side has

**Fuel FKC** (`docs/specs/kernel-contract-format.md` section 5.3 at Fuel origin/main): `layout_guarantee` is a per-output string, four forms: `contiguous` (dense row-major), `preallocated` (executor owns the buffer; the default and always true), `same_as(<input>)` (preserves an input's strides), `bundle` (packed multi-slot buffer, section 5.5). A separate field `aliasing` (`none`, `in_place(role)`, `accumulate(role)`, section 5.4) is orthogonal. Measured over `fuel-dispatch` `*.fkc.md` at origin/main: 1012 `contiguous`, 4 `preallocated`, 8 `same_as(...)` (dst 3, x 2, dest 2, base 1), 1 placeholder `_`. The seam inventory (`fuel-seam-inventory.md` line 89) records the KISS side as zero occurrences, a confirmed migration-data-loss row.

**KISS today** (`spec/contract.md`): an output operand has `stridesOut[rank]`, a launch scalar the *caller* passes (line 219), so it describes the buffer supplied, not what is written. "Fully packed" (lines 362, 936) is a property of a *cell*, derived by the Classify contiguous-layout predicate over every operand including the output. No clause lets a kernel say it writes something other than the supplied layout. 0 hits for `layout_guarantee` in spec/ (the same grep form finds `stridesOut`, present).

**Unpopped** (`EvansLaboratories/Unpopped` origin/main): three layout vocabularies already exist.
- `unpopped-vocab/src/layout_algebra.rs`: a hierarchical `shape:stride` `Layout` over nested `IntTuple`s (CuTe style) plus `swizzle.rs` (`Swizzle`). This is a *store layout* in the full sense: nesting, non-unit and negative-free strides, bank swizzles.
- `unpopped-vocab/src/layout.rs`: the closed GEMM `LayoutSku` (`Rcr`, `Rrr`: row/column orientation of A, B, C/D).
- `structure_key.rs` `LayoutOrder`, and `artifact.rs` `LayoutRule` (`Std140`, `Std430`, `Scalar`) for parameter blocks.
The Unpopped lane's relay (PM, 2026-11-04): a kernel should state a **full store Layout, not just strides**; squeeze is a Reshape.

## 2. What a layout guarantee must be able to say (union of the three)

1. dense in canonical order (FKC `contiguous`);
2. dense in an **explicit axis order** (a permutation; column-major is one);
3. the **same layout as a named operand** (FKC `same_as`, Unpopped output-matches-input);
4. **strided** with strides given as expressions over the launch scalars (an output the kernel writes with its own strides, including padded rows);
5. **hierarchical** `shape:stride` with optional **swizzle** (Unpopped `Layout` + `Swizzle`), the tensor-core and shared-memory case;
6. **packed multi-slot bundle** (FKC `bundle`);
7. **no guarantee** (the caller must read the layout from the buffer).
Orthogonal, and kept as a separate field: **aliasing** (`none`, `in_place`, `accumulate`) and **who allocates** (`preallocated`).

## 3. Options

- **A. A closed-sum `layout_guarantee` field in Guarantees (6.8).** Forms 1 to 7 as tagged variants, each variant's operands (axis order, operand name, stride expressions in the 6.6-0006 grammar, a `shape:stride` text with a pinned grammar, swizzle parameters). New optional field; a reader that ignores it is unaffected only if the default is "same as supplied" (state that default). Widest; keeps Interface about what the caller supplies. **Byte-changing** (a new contract field).
- **B. Place the same sum in Interface (6.5) beside each output operand.** Same content, mixes input description and output promise; but the guarantee sits beside the operand it qualifies, and a multi-output kernel needs no index.
- **C. Start with forms 1, 3, 7 (FKC parity) and reserve the rest.** Smallest; closes the FKC row; leaves Unpopped's 4 to 6 as a named post-v1 extension (the way 6.6-0008 reserves the tile mapping). Risk: the board said "so nothing is blocked later"; a closed sum without an extension rule blocks later.
- **D. Decline.** Output layout is always the supplied layout. FKC's field is recorded ABSENT/DECLINED; rejected by the instruction to keep expressibility.

## 4. Recommendation: A, with the extension rule of C

Write the full closed sum (forms 1 to 7) in Guarantees, with **form 1 plus an absent field meaning "the layout supplied"** so every existing contract keeps its meaning, and an explicit extension rule (an unknown variant is a typed decline of the guarantee, never ignorance of it). Reasons: it is the only design that holds all three vocabularies; Guarantees already carries "what the kernel does"; and the Unpopped hierarchical layout needs a pinned text grammar anyway, which is cheaper to write once. **Open points, none blocking the choice**:
- a grammar for `shape:stride` and swizzle (take Unpopped's `Display` as the starting form; check it round-trips);
- whether a stride expression may name a bound symbol (it would be a bind-time check, see `docs/design-r1-22-bind-time-obligations.md`);
- how `bundle` slot specs relate to multi-output (FKC 5.5);
- structure key: the guarantee is something the kernel *produces*, so by the admissibility rule it belongs in `revision_hash`, not `structure_key`; confirm with Classify.

Byte consequence: a new optional contract field and a new reserved name set; no existing vector moves. No clause until CireSnave picks A, B or C (or says "widest" means A).
