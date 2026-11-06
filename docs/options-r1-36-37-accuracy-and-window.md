# Options: guaranteed-accuracy declaration (R1.36) and the window-overlap rule (R1.37)

Status: **options with a recommendation; nothing here is a clause.** Prepared 2026-10-31 against `origin/main` aa0cc223 (conformance 0.6.51). CireSnave's rulings (board 338, 2026-10-30, relayed by the PM, quoted in the ROADMAP rows): Q4 "Yes. Each backend (and likely each kernel through each backend) specify the accuracy it guarantees." (KISS defines the format, never the numbers); Q5 "Yes." (a window must overlap at least one real input element; shapes violating it are not expressible). Anything that binds a consumer or moves wire bytes waits for his word before it becomes a clause.

## R1.37 — the window-overlap rule (measured; do this one first)

**Measured at `origin/main` aa0cc223.**
- KISS-OPS-6.13-0004 requires `avg_pool`, `max_pool` and `im2col` to declare per-axis `window_size`, `stride`, `dilation` and `padding`; out-of-bounds taps are skipped, and pad-fill is `oob_policy` (6.19-0015). `padding` is **one `u32` per spatial axis** (6.19-0023), so padding is symmetric. No clause bounds `padding` against the window, and the **output extent formula is not in `spec/ops.md`** (`git grep -n -i -E 'output extent|out_extent|output spatial' -- spec/ops.md` finds nothing; control: the same file contains "OOB taps skipped" twice).
- So a window that lies wholly in the padding is expressible, and its result is an empty reduction: `max_pool` has no tap, and `avg_pool` with `count_include_pad` false divides by a zero valid-tap count.

**Brute-force check** (scratch script, not committed; n ≤ 12, k ≤ 6, d ≤ 4, s ≤ 4, p ≤ 11, with the assumed output extent `floor((n + 2p - D)/s) + 1`, `D = d(k-1)+1`, taken only where at least one output window exists; control: `n=4, k=3, d=1, s=1, p=3` has an all-padding window and is reported as such).
- **Dilation 1:** `2*pad <= window_size` leaves **no** window without a real tap (0 violations); every empty window found has `2*pad > window_size` (smallest margin 1). So for `d = 1` the relayed rule is exactly sufficient.
- **Dilation above 1:** the same bound is **not** sufficient: 12 counterexamples (for example `n=1, k=2, d=2, s=1, p=1`: the one output window has taps at -1 and 1, and the only real position is 0, so it has no real tap). Using the dilated extent as the span (`2p <= D`) is weaker still: 78 counterexamples. `p <= D-1` leaves 362.
- So an attribute-only bound cannot, by itself, guarantee overlap once `dilation > 1`; the guarantee needs the operand extent.

**Options.**
- **A. `2*pad <= window_size`, attribute-only** (the relayed wording, span := `window_size`). Checkable on the OpAttrs vector alone. Exact for `dilation = 1`; leaves a hole for `dilation > 1`.
- **B. The same bound with span := dilated extent.** Rejected: weaker than A, 78 holes.
- **C. A shape-level condition:** every output window has at least one in-bounds tap, stated over (OpAttrs, operand extents). Exact. Needs the output extent formula pinned first (it is not in the spec).
- **D. A and C together:** A as the cheap OpAttrs-level validity rule, C as the exact condition a reader checks once it holds the cell's extents.

**Recommendation: D**, with the output extent formula stated in the same change. A violating shape is **not expressible**: a producer MUST NOT write it and a reader MUST decline it (a clause on validity, not a runtime result). It constrains values, not encodings, so it should not be byte-altering. **Needs CireSnave's word on the span:** his words were "2*pad <= span" and the choice of `window_size` (A) over a shape-level condition (C) decides what is expressible for dilated pools. A vector witness belongs in `grammar_vectors.json` or an OpAttrs vector set, one accepting and one declining shape per axis class.

## R1.36 — a declaration format for guaranteed accuracy

**Measured.**
- The per-kernel half largely exists. The Contract **Guarantees** section carries the named `reference_function`, per-backend ULP tiers, the determinism class, the MathFidelity attribute and bit-stability (`spec/contract.md`, Guarantees section; KISS-CONTRACT-6.8-0009/-0010 derive `audited_status` from it). The **precision mode** of Q4 is KISS-Ops 6.17's MathFidelity enum `{bit-stable, reduced-mantissa-permitted}`.
- Missing: a **backend-level** declaration keyed by (op, precision mode, dtype), and a stated relation between a kernel's Guarantees and the accuracy its backend declares for the same key.

**Options.**
- **A. A provider-level record** keyed by (op token, MathFidelity mode, dtype) with a value of reference function plus ULP tier (or bound) plus determinism class, in the vocabulary the Guarantees section already uses. It travels where R1.33's dtype record travels (the contract-query), so one provider record can carry both.
- **B. Per-kernel only** (what exists): a backend's accuracy is whatever its kernels declare. Needs no change, but a consumer cannot learn a backend's accuracy before it holds a kernel.
- **C. A per-token table in each namespace vocabulary document** (6.8-0004). Static; cannot say what an actual device guarantees.

**Relation between the two levels (the open design point).** The mode is part of the key, so a looser kernel (a fast-math kernel) belongs under the looser mode row, not under the strict one. Within one mode the natural rule is that the backend's declaration is the **floor** a kernel through that backend may not claim to undercut: a kernel may state a tighter guarantee, never a looser one under the same key. How `audited_status` treats a kernel that claims better than its backend is not decided here.

**Recommendation: A**, with the floor rule above, KISS pinning the shape and meaning and never a backend's numbers (the 6.8-0004 line). It is a new record (a Contract or provider-record version event) and **needs CireSnave's word**; the kernel-side change, if any, is only the stated relation.

## What this note does not decide

Whether to adopt any option; the byte-level encoding of a new record; any backend's numbers; whether the output extent formula lands in the same PR as R1.37.
