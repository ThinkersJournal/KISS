# Options: the three seam mechanisms of board 338 (ROADMAP R1.32, R1.33, R1.34)

Status: **options with a recommendation; nothing here is a clause.** Prepared 2026-10-30 against `origin/main` 14aa3a4b. CireSnave's ruling (board 338, relayed by the PM): KISS specifies how a seam learns what it must know; it does **not** specify a backend's capabilities, and it does not set a vector width. So each section asks for a **mechanism**, never a list or a value. Any option that binds a consumer, or moves wire bytes, needs his word before it becomes a clause. Q4 (numeric meaning of conformant output) and Q5 (window padding) are still his and are not touched.

## R1.32 — which base operations only move bits, and which compute

**Measured.**
- KISS-OPS-6.16-0009/-0010/-0011 state the obligations and deliberately give a **predicate on the op's result, not a lookup**. 6.16-0011 traces a fused op to its observable output.
- The readable catalog (`spec/ops.md` §2.7, informative, "where this catalog and §6 differ, §6 governs") sorts ops by **family**, and the family is not the class: `neg` and `abs` are filed under `arithmetic`, while 6.16-0009 treats a sign-bit edit as a move, and `copysign` is `binary_math` with the note "(raw-bit)". A reader who maps family to class gets `neg` wrong.
- `git grep -n -E 'is_bit_move|bit_move' -- spec ROADMAP.md conformance/src` finds nothing (control: `6.16-0009` is found in `spec/ops.md`). Unpopped's `is_bit_move` family is a per-implementation reading.

**Options.**
- **A. Status quo.** Predicate only. A seam derives the class itself. Cheapest; this is what produced the divergence the seams note describes.
- **B. A normative `class` per primitive-floor op and dtype kind** (`move` | `compute`; for `neg` and `abs` the float reading is a sign edit and the integer reading is wrapping arithmetic, so the class is not a property of the op alone, ROADMAP R1.35) in the §6.1 registry, with fused ops still traced by 6.16-0011 from their members' classes. The table is a **consequence** of the predicate and a check on it: an op whose class disagrees with the predicate is a spec defect. Gives a seam a lookup without losing the predicate.
- **C. Informative table only**, in §2.7, beside the family column, with the predicate still governing.

**Recommendation: B for the primitive floor only**, fused ops left to 6.16-0011. Reason: the seam question is "does this base op only move bits", which is exactly a per-op fact; the predicate stays the authority so a new op cannot be mislabelled by a table. **Needs CireSnave's word** because an implementation's `is_bit_move` would then have to agree with the registry (a new obligation on implementers). Candidates to run the predicate over before writing the table: `neg`, `abs`, `copysign`, `select`, `max_prop`, `min_prop`; the existing informative cells (`ops-narrow-move-nan.json`, `ops-move-wide-nan.json`) witness the move side. 6.16-0011's multi-stage case still has no cell shape.

## R1.33 — a backend declares which dtypes it can compute on

**Measured.**
- Announce carries identity, not capability, except the provider-level `capabilities` bitset (`spec/announce.md` §7.2). The FEAT axis is bits [24, 48), of which the core sub-range is [24, 40): 16 bits shared by every provider feature. The §2.6 dtype catalog in `spec/classify.md` has 29 table rows by my count (reserved spellings included). A per-dtype bit set does not fit, and a bit per dtype would also spend the core range.
- The Contract's **Capabilities section** already carries a per-kernel "supported dtype set" (`spec/contract.md`, the Capabilities-section definition near line 371): the shape exists at kernel scope, and a contract travels through the contract-query (FEAT bit 33, `CONTRACT_QUERY`) as an opaque payload Announce does not interpret.
- KISS-CLASSIFY-6.8-0018 is the precedent for a target property declared once by the namespace (its vector-access byte cap), "not per capability-set". It is the wrong grain here: dtype support differs between capabilities of one namespace (the `cuda:sm89` row in `spec/namespaces/cuda.md` is the FP8-tensor-core one), so a per-namespace declaration would state too little.

**Options.**
- **A. Announce bits.** Rejected on the measurement above.
- **B. A provider-level capabilities record answered through the contract-query**, reusing the Capabilities section's dtype-set shape at provider scope. Instance-accurate (a provider knows its device), Announce stays tiny and fixed.
- **C. A per-token table in each namespace vocabulary document** (6.8-0004 keeps vocabularies maintainer-owned). Static, so it cannot say what an actual device does, and it moves the declaration into every namespace file.

**Recommendation: B**, with the record distinguishing **compute** support from **store/move** support (the two sets are exactly what R1.32 separates), and with the dtype set naming only Classify's closed tokens, never a list KISS owns. A new Contract section is a Contract version event and **needs CireSnave's word**. KISS pins the declaration shape and its meaning ("this provider will compute these dtypes"), not any provider's content.

## R1.34 — a kernel contract states its vector width

**Measured.**
- KISS-CLASSIFY-6.5-0009 **derives** a per-operand vector-access width (`v1`..`v8`) from the operand's layout, its alignment (exact-modulo gate) and the target's vector-access byte cap, and the width is a coordinate of the cell identity. The cap is the namespace's (6.8-0018, 16 bytes unless declared otherwise).
- KISS-CONTRACT-6.5-0008 gives the contract **one** width, in `count_unit` (`vectors_x<w>`), saying how `n` is counted.
- No clause relates the two: nothing says which operand's derived width `w` equals, and the contract has no field that states a per-operand width. Operands of one cell can differ (a broadcast operand derives `v1` while another derives `v4`), so one number cannot be the whole statement.
- Related fact (structure_key admissibility): the key says what was **asked for** and the revision hash what was **produced**, so a contract stating a width narrower than the derived one is consistent with a cache hit only if both are checked.

**Options.**
- **A. Clarify only.** Keep `count_unit` as the mechanism; add one clause relating `w` to the derived width of a named operand (the one `n` walks), plus a vector. No new field.
- **B. A per-operand `vector_widths` list in the Contract**, one entry per positional operand, each a statement of the width the kernel was **built** for, constrained to be at most the derived `vL` and to divide it; `count_unit` then must agree with the walked operand's entry. KISS sets no value.
- **C. Let a contract override the derived width** in either direction. Rejected: it breaks the key-says-asked rule, and a width wider than the derived one is a misaligned load.

**Recommendation: A first, B only if A leaves a seam unable to tell the widths apart.** A is a clarification and a vector with no new field; B adds a Contract field and is byte-altering, so it **needs CireSnave's word**. Either way KISS states the relation and the check, never the width.

## What this note does not decide

Whether to adopt any option; the byte-level encoding of a new Contract field or section; the contents of any backend's declaration; Q4 and Q5.
