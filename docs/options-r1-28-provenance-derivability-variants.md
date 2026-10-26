# Options note: R1.28, provenance, derivability and variant fidelity (two questions and one already ruled)

**Informative. No normative text. For CireSnave's decision; the PM turns each question into a board item.**
Read at `origin/main` `2e65634`, 2026-10-26. Paths are `file:line` at that commit. Issues: KISS#218, #228, #234
(Unpopped, Vulkane, and the architect). Roadmap row: R1.28. Nothing here freezes. Q1 is **half done**: the clause
was fixed and only the files that carry the old wording are left.

---

## Q1. Do we rewrite the `generated_from` field in the two vector files now? (#218)

**Example.** `conformance/corpus/structure_key_vectors.json` says `"generated_from": "spec/classify.md"`. Its bytes
are written by a program, `cargo run --bin emit_structure_key_vectors`, from `conformance/src/reference_vectors.rs`.
So the field names a document the bytes did not come from. A reader who trusts it goes to the wrong place.

**What is already done.** The clause that compelled this is fixed. KISS-CLASSIFY-6.8-0011 (`spec/classify.md:1818`)
now says a manifest names **what emitted its bytes**, and separately agrees with its annex under a freshness gate.
The two things that remain are in the data: (1) `structure_key_vectors.json` and
`structure_key_derivation_vectors.json` both still say `spec/classify.md`, and both are program output (each has an
`emit_*` binary that is the only writer); (2) `structure_key_vectors.json` carries `"source_commit": "19c3ad7"`, a
literal that is never updated. The other two manifests are honest: `dtype_manifest.json` and `op_manifest.json` are
written by `tools/kiss_dtypes.py` and `tools/kiss_ops.py` **from** the spec, so naming the spec is true.

**Why it was deferred.** Changing the field moves bytes in a file other projects copy. Fuel and Unpopped each vendor
`structure_key_vectors.json` and their tests pin its `source_commit` (`kiss_structure_key_byte_match.rs` in Fuel,
`kiss_byte_match.rs:62` in Unpopped). I did not read whether either also pins `generated_from`.

**Options.**
- **A. Fix both files now.** `generated_from` names the emitting binary and module; the vendors regenerate their
  copy and update the one pin. *Cost:* one small regeneration in two other projects for a field-only change.
- **B. Fix them the next time the file is regenerated for any other reason** (the next key-schema event). *Cost:*
  the false field stays until then.
- **C. Add a new field `emitted_by` and leave `generated_from` alone.** Additive, no vendor change. *Cost:* the
  false field stays and now has a true neighbour, which is more confusing than either A or B.

**Recommendation: B.** The harm is a misleading label, not a wrong byte, and the vendors regenerate on the next
key-schema event anyway, so it costs them nothing extra then. **A if you want the false field gone now**, since the clause already says it must be.
`source_commit` is a **separate** obligation the clause says is not pinned; it needs its own answer and I would
drop it (a literal that no tool updates is the same kind of false provenance), but that is also a byte move.

---

## Q2. Must a named namespace capability be one that some device can produce? (#228)

**Example, from the issue.** Vulkane's namespace document named a set of packed component types. The deriver had no
case for them, so a real device would have emitted `x1000491000` (the catch-all spelling) and every check stayed green:
the name was well formed, listed in the manifest, and unreachable. It was caught in review, not by a test.

**What the specs say.** The manifest envelope is pinned and its content is the maintainer's
(KISS-CLASSIFY-6.8-0008, `spec/classify.md:1779`: "KISS pins this envelope only"). Nothing requires a listed entry to
be producible. A generated manifest is a round trip from the vocabulary crate, so it agrees with itself whether or not
the deriver can emit the entry. `cuda`, the first manifest, has nothing holding it to the standard either.

**Options.**
- **A. A required field per entry** naming the witness: the query or property that makes a device produce the token.
  "Listed but unreachable" becomes impossible to write down. Envelope-level, so content stays the maintainer's.
  *Cost:* every manifest author adds a field, and the field is a claim, so something must still check it.
- **B. A required maintainer-side test,** cited the way clause backings are: the namespace asserts every entry it names is
  producible by its own deriver, and KISS requires the citation to exist. *Cost:* KISS checks the **existence** of a
  citation, not that the test discriminates.
- **C. Both:** the field names the witness, the test exercises it. The hybrid the issue prefers.
- **D. Nothing now;** wait until a second namespace deriver exists.

**Recommendation: C, in two steps: A first.** A field is cheap and makes the gap unrepresentable. B is what proves the
witness is true, but it asks the namespace maintainers for tests, so ask them (Baracuda for `cuda`, Vulkane for
`vulkan`) before it is required. Do it now, while there are two manifests and not more. It waits on Q1 only in the
sense that both change the manifest; it does not depend on it.

---

## Q3. Does KISS give a home to "this variant is bit-identical to the default lowering of the same cell"? (#234) **Already ruled; not a decision.**

**Example.** An implementation produces two schedules of one cell, an unrolled one and the default, and wants to say the
unrolled one computes the same bits as the default: a relation between two variants, local to one backend at one version.
KISS-EMIT-6.6-0005 (`spec/emit.md:816`) forbids declaring an emitted kernel's fidelity anywhere but Guarantees, and Guarantees
holds claims against the kernel's own semantics, so a variant-relative claim filed there would read as a stronger claim than it is.

**What is already decided.** ROADMAP R1.5 was **ruled B on 2026-10-15**: `VariantFidelity` is implementation-private, it stays on
Unpopped's `Variant` and never on the wire, and there is no KISS-Emit clause ("Nothing to mint"). That is the issue's own
alternative ("variant-to-variant relationships are out of scope entirely ... worth saying"). **I wrote the first version of this
note without having read R1.5; the question was already answered.** The issue stays open for one reason: the ruling is in the
roadmap and not in the spec, so an implementer who reads only `spec/emit.md` still cannot tell off-schema from unmentioned.

**One small thing is left, and it is not a ruling.** Add a single informative sentence next to KISS-EMIT-6.6-0005 saying that a
relation between two variants of one cell is implementation-private and is not a KISS field, and that its only possible referent is
the default lowering of the same cell in the same backend at the same version. No bytes change. It needs only the PM's approval.

---

## Summary

| Q | Issue | Recommend | Bytes change? | Note |
|---|---|---|---|---|
| 1 | #218 `generated_from` in the two vector files | B (next regeneration); A to do it now | yes (two files) | clause already fixed; `source_commit` separate |
| 2 | #228 derivable capability | C, field first | manifest field (additive) | ask the two maintainers first |
| 3 | #234 variant fidelity | none: ruled (R1.5, 2026-10-15); add one informative sentence to Emit if the PM agrees | no | not a decision |
