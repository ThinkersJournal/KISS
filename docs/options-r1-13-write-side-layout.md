# Options note: R1.13, where does a kernel say what layout it writes? (#316, with #236)

**Informative. No normative text. For CireSnave's decision; the PM turns the question into a board item.**
Read at `origin/main` `2e65634`, 2026-10-26. Paths are `file:line` at that commit. Issues: KISS#316 (the decision) and
#236 (the convergence tracker, not a decision). Roadmap row: R1.13. Nothing here freezes except through Q1's bytes.

---

## Q1. Does a write-side layout guarantee belong in Guarantees, in Interface, or nowhere yet? (#316)

**Example.** A kernel is handed an output buffer with unusual strides, and writes the result densely, row after row,
ignoring the strides. Or it normalises a layout. A caller that reads the buffer needs to know which. Today no section
of a contract says what the kernel **writes**.

**What the specs say.** The structure key's output operand is the **accept predicate**: what the caller must supply
(the issue's words). The Interface carries the output's
strides as a **launch scalar the caller passes in** (`stridesOut`, `spec/contract.md:219`; the strides rule is
KISS-CONTRACT-6.5-0005, `spec/contract.md:927`). That field looks like the answer and is on the other side of it: it
describes the buffer supplied, not the layout produced. The two coincide for every kernel anyone emits today, which is
why the gap is vacuous now (the issue says so, and I did not re-measure it). The Interface has exactly seven fields
(KISS-CONTRACT-6.5-0001, `spec/contract.md:859`) and the Guarantees section exactly eight (6.8-0001, `:1128`), so
**either home changes a field list, which changes `contract_version`.** That version is already moving: the
declared-width change in #694 takes it from 3 to 4 and has not merged. A field added now **rides the same bump**; one
added after it is a second bump.

**Options.**
- **A. A Guarantees field, one entry per output.** For example `output_layout`, whose values say "writes the layout it was
  handed" or "writes contiguously in canonical order". Guarantees is where a claim about what the kernel does belongs, and
  the issue leans there. *Cost:* a ninth field; the value vocabulary has to be defined once (the Classify
  contiguous-layout predicate is the starting point); every contract writer adds it.
- **B. An Interface field beside the output operand it qualifies.** Keeps the output's description in one place.
  *Cost:* the Interface describes what a caller must **supply**, so a claim about what is **produced** goes into the
  section whose whole shape is the other direction, the confusion the issue is trying to avoid.
- **C. Wait.** It is vacuous today and the first party who needs it will say so. *Cost:* the next addition is a
  second schema bump, and an outside implementer at the freeze gate is the likely first user.

**Recommendation: A, while the version is already open; otherwise C.** A claim about behaviour belongs with the other
behavioural claims, and a per-output list matches how the other contract format carries it (`return.outputs[]`, per the
issue). If #694 merges first and nothing here is decided, do C and add it at the next scheduled schema event rather
than spend a bump on a vacuous field. **What I need from you:** A, B or C, and whether to ride #694's bump.

---

## Q2. Is "should a contract replace the other format or be derived from it" still open? (#236) **Not a decision.**

That question was the first line of #236 and was **ruled on 2026-09-05** (CireSnave, verbatim on the issue): the two
formats are supposed to be the same thing, and where they differ the projects converge on the best of both. The
ratified sequence on the issue is four phases: **functional parity** (every capability of the old format has a KISS
equivalent, and no project migrates before this gate passes), a **migration guide**, **migration**, then
**decommissioning** the old format except as a history note. The scope was later widened from the format to every
seam point in Fuel that KISS will regulate. So #236 stays open as the tracker of those phases. Nothing in it waits on a
decision from you; it waits on the parity ledger being complete, which is Fuel's and the architect's work.
Q1 above is one row of that ledger (the layout guarantee has no home), which is why the two issues share a row.

---

## Summary

| Q | Issue | Recommend | Bytes change? | Note |
|---|---|---|---|---|
| 1 | #316 write-side layout | A (Guarantees, per output) if #694's bump is still open; else C | yes (contract field list) | vacuous today |
| 2 | #236 replace or derive | none: already ruled 2026-09-05 | no | tracker of the four phases |
