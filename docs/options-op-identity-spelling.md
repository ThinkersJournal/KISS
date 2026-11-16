# Options note: the text spelling of `op_identity` (Contract 6.11-0009) has four open points

**Informative. No normative text. For CireSnave's decision (each answer changes bytes of the `op_identity` line, so each needs his word).**
Measured at `origin/main` `8c14914`, 2026-11-16, by reading `spec/contract.md`, `spec/ops.md`, `spec/grammar.md` and
`conformance/corpus/op_facts.json` at that ref.

**Why this is asked.** FireLine asked for vectors that say, per form, which `op_identity` lines a reader accepts and which it
declines (Contract 6.3-0003, 6.3-0004). A vector is a line of bytes plus a verdict, so the verdict must be decidable from the
spec text. Clause 6.11-0009 pins the *shape* `<op_name>[{<op_attrs>}][[<operand_role_tuple>]]` and nothing finer. Four
points are left to the reader's guess. No vector is built until they are answered.

**A defect found on the way (needs no decision, only a fix with the answers).** The two golden lines in the Contract's
Appendix C (`spec/contract.md:2167-2168`) do not match KISS-Ops:

- `gather{axis=k, oob=clamp}`: `k` is a variable, not a value; the field is `oob_policy`, not `oob`; and `gather` also
  carries `index_operand` and `index_dtype` (`op_facts.json`, `gather.opattrs` = `axis, oob_policy, index_operand, index_dtype`).
- `scan{op=add}`: `scan` is not a KISS-Ops op (the op is `prefix_scan`, whose fields are `monoid, reduce_axes, exclusivity,
  accumulator, math_fidelity`), and `op` is not one of them.

KISS-Ops itself spells the key both ways: `spec/ops.md:3727` writes `gather(axis=0, oob=clamp, ...)`, and
`spec/ops.md:3801` writes `oob_policy=skip`. That is point 3 below.

## Point 1: in what order do the `key=value` pairs appear inside `{...}`?

Today the text says "the `key=value` list separated by `, `" (`contract.md:1597`) and does not say the order.

**A. The KISS-Ops canonical (wire) field order**, the order `spec/ops.md` §6.19.3 lists the fields in (`gather` is
`axis, oob_policy, index_operand, index_dtype`). *What it does to bytes:* one rendering per op, so two writers produce the same line.
It is the order the binary OpAttrs blob already uses, so the text and the blob read in step. **B. Alphabetical by key.**
*What it does:* also one rendering, but unrelated to the blob order, so a reader needs a second table. **C. Any order, compared
as a set.** *What it does:* two writers may emit different bytes for the same op, which the clause forbids
("Two dissimilar implementations MUST render the identical `op_identity` bytes").
**Recommendation: A.** It needs no new table, and `op_facts.json` already carries each op's field list in that order.

## Point 2: which attributes appear, and are defaulted ones left out?

The clause says `{...}` is present "iff the root carries a non-empty identity-bearing OpAttrs set" and calls the
rendering "default-eliding" (`contract.md:1597`, `1619`). KISS-Grammar says the opposite about the tag it renders:
"defaults MUST NOT be elided" (`grammar.md:657-675`, 6.1-0007 (b)). Both are in force, so a reader cannot tell whether
`gather{axis=0, oob_policy=skip, index_operand=1, index_dtype=u32}` or `gather` is the line for a default gather.

**A. Always write every field of the op's OpAttrs record, defaults included.** *What it does:* the line is longer, never
ambiguous, and matches the Grammar rule; the golden `add` stays bare because `add` carries no OpAttrs. *Price:* the
"default-eliding" words in 6.11-0009 are struck. **B. Write only the fields that differ from their default.** *What it does:*
shorter lines; a reader must know every default of every op to rebuild the record, which makes the text a second source of
the defaults. **C. Write only fields the producer set explicitly.** *What it does:* two producers of one op disagree on the line;
this is the case 6.11-0009 itself warns against, so we should not choose it.
**Recommendation: A.** The text line is a non-authoritative render of a default-resolved form (`contract.md:1611-1619`),
so it should carry the resolved form, and a reader needs no defaults table.

## Point 3: how is each value spelled, and what are the key names?

The clause says "spelled exactly as KISS-Ops spells them" (`contract.md:1598`), but KISS-Ops has no text spelling table. It
has the wire bytes (§6.19.3) and worked examples that disagree (`oob=` at `ops.md:3727`, `oob_policy=` at `ops.md:3801`;
`combine=atomic-add` at `ops.md:3736`; `reduce_axes=0x0000` at `ops.md:2661` for an axis set in hex).

**A. Key = the §6.19.3 field name exactly; value = the sub-vocabulary name for an enum (`clamp`, `skip`, `atomic-add`), a
decimal integer for a count or index, the dtype token for a dtype, `true`/`false` for a bool, and a bracketed
comma list of decimal integers (`[0,1]`) for an axis set.** *What it does:* the line can be written from the §6.19.3 table
and the §6.19.2 vocabulary with no other source; golden lines and the `oob=` examples are corrected. **B. Value = the wire
byte in hex** (`oob_policy=02`). *What it does:* trivial to generate from the blob; unreadable, and it ties the text to the
byte tables. **C. Leave it to each op's reader.** *What it does:* no vector can be written for any attribute value.
**Recommendation: A.** It is the form every worked example except the hex ones already uses.

## Point 4: can a reader tell form (a) from form (b)?

Form (a) is a Grammar advertisable-op tag (the line may end in `[role:dtype, ...]`); form (b) is the bare op name plus
attributes with no such suffix (`contract.md:652-668`). 6.3-0004 says that when several advertisable ops differ only by
attributes, the line "MUST NOT be reduced to the bare op name". Whether a given line was reduced is knowable only by knowing
which advertisable ops exist, and the document does not carry that list.

**A. Decide only what the line itself shows:** a role-tuple suffix is a form (a) line, no suffix is either form, and a line
whose attribute list omits a field (after Point 2 A) is declined as malformed. *What it does:* vectors cover spelling and
completeness; the "reduced to the bare name" fault is caught only through the missing-field rule. **B. Add to the Identity
section a field naming the advertisable-op tag, or its absence.** *What it does:* decidable, at the cost of a new field in the
Identity schema (6.3-0001) and so new golden bytes. **C. Make 6.3-0004 a producer-side rule, untestable from a document.**
*What it does:* the clause stays and the vectors for it are dropped.
**Recommendation: A**, with C's wording added to 6.3-0004 for the part A cannot see. B moves bytes in a pinned section for
one check.

## Decisions needed

Point 1 A, B or C; Point 2 A, B or C; Point 3 A, B or C; Point 4 A, B or C. If A, A, A, A: 6.11-0009 gains one ordering
sentence and loses its "default-eliding" words, the two Appendix C lines are rewritten (`gather{axis=0,
oob_policy=skip, index_operand=1, index_dtype=u32}`, `prefix_scan{monoid=sum, reduce_axes=[0], exclusivity=inclusive,
accumulator=..., math_fidelity=...}` with the real defaults read from §6.19.3), and the vectors for 6.3-0003/-0004 follow.

— lane: kiss
