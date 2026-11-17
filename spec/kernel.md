# KISS-Kernel — What One Thread Does in a Launch of Many (kernel-body semantics)

**Sub-standard ID:** KISS-KERNEL
**Part of:** KISS — Kernel Interface Standards Suite
**Steward:** ThinkersJournal (non-profit public-standards publisher)
**This document:** First-draft **skeleton**. Not ratified. Not frozen. Its normative section reserves the ranges below and
carries one reference clause; **no barrier, atomic, fence, collective or race rule is stated yet** (§6.0).

> This document follows the KISS dual-doc template defined in the *KISS Design Charter*: an **informative Overview**
> (§0–§5) and a **normative Conformance specification** (§6+). Only §6+ is normative. Normative clauses use RFC-2119 /
> RFC-8174 uppercase keywords, carry an append-only clause ID `KISS-KERNEL-<section>-<nnnn>`, and each maps 1:1 to at
> least one named KISS-Conform test or document lint.

---

## 0. Front-matter

| Field | Value |
|---|---|
| Title | KISS-Kernel |
| Sub-standard ID | KISS-KERNEL |
| Maturity stage | **Draft** (skeleton; created by board 450, option A, 2026-11-16; no kernel-body rule is published yet) |
| Editor of record | **Proposed, pending ratification** — the KISS lane holds the pen; the Unpopped project supplies the source model (`docs/kiss-kernel-body-concept-map.md` in the Unpopped repository) as one implementation. |
| Steward | ThinkersJournal |
| Reference seed crate(s) | none yet. The source model is Unpopped's kernel layer (an interpreter, a serial-C differential and seeded mutants; **never run on a device**). |
| DAG position | **Behavioural tier, beside Emit and Consume.** Depends on KISS-Ops and KISS-Classify; nothing depends on it yet. |
| Upstream edges | KISS-Ops (**STRUCTURAL** — the kernel-body rules fix the thread-level order, scope and race behaviour of the Ops atomic combines (KISS-OPS-6.11-0006, -0010, -0024) and take the Ops monoid set); KISS-Classify (**STRUCTURAL** — the capability and `target_capability` vocabulary the target-stated rows of §6.0 use). |
| Downstream edges | **None in version 1.** The reverse edges into KISS-Emit and KISS-Consume are not added, because version 1 does not change what those lower or lift (`docs/kernel-standard-discoverability.md`). |
| Spec license | CC0 1.0 Universal (public-domain dedication) |
| Reference-crate license | MIT-OR-Apache-2.0 |

> **Edge-label note (informative).** The Ops edge is **STRUCTURAL** because KISS-Kernel cannot be implemented without
> reading the Ops clauses it governs: that is the umbrella's own test (umbrella §2.2). An **OPAQUE** label would let a
> claimant skip KISS-Ops.

---

## 1. Purpose & Scope

KISS-Ops says *what is computed* over whole tensors. It says nothing about *what one thread does in a launch of many*:
when threads must wait for each other, in what order one thread's writes become visible to another, and which programs are
data races. Two kernels can agree on every op and still disagree on what a program may assume. This sub-standard is the
home for those rules.

**In scope (version 1, as ruled by board 450 option A):** the launch shape and ids; the statement and expression core a
thread executes; memory spaces and places; barriers at four reach levels with the rule that every thread of the group must
arrive; atomics with order and reach, fences, and the race rule as a classification of programs; group collectives.

**Out of scope (version 1):** named barriers, the asynchronous barrier and copy features, tensor memory, matrix-multiply
units, and total ordering of every operation (board 450 option A text); any claim about a specific GPU; any
source-language syntax (KISS-CONSUME-6.1-0003 forbids blessing a language).

**What this skeleton is.** Board 450 ruled the sub-standard into existence **on the condition that the references to it are
kept in place, so readers know when recommendations are available**. This document, the umbrella rows, the KISS-Ops
"Kernel-governed ops" table and the `kernel_rules` field of the operation facts are those references. The rules themselves
wait on the six choices filed as board 459; no clause text exists for them (§6.0).

## 2. Overview / Rationale (informative)

### 2.1 Why a separate standard

Its subject is one thread's behaviour in a launch, its conformance evidence differs (programs plus expected verdicts, not
tensor vectors), and KISS-Ops stays free of a machine model. The alternative, a KISS-Ops section, would couple the two
versioning axes.

### 2.2 The reserved map

| Reserved section | Subject | Gated by |
|---|---|---|
| 6.1 | the reference rule (this skeleton's one clause) | none; published |
| 6.2 | launch shape and ids; statements and expressions; memory spaces | the first rulings (none pending) |
| 6.3 | barriers at four reach levels; the uniformity rule | board 459, choices 6 and 7 area |
| 6.4 | atomics with order and reach; fences | board 459, choices 1 and 8 |
| 6.5 | the race rule as a classification of programs | board 459, choice 4 |
| 6.6 | group collectives and their monoids | board 459, choice 2 |
| 6.7 | reading memory nobody wrote | board 459, choice 5 |

A reserved section carries **no normative text**. A reader MUST NOT infer a rule from its heading.

## 3. Terms & Definitions

- **Kernel body** — the code one thread executes in a launch of many threads.
- **Kernel-governed op** — a KISS-Ops operation whose concurrent behaviour (order, reach or race) is fixed by KISS-Kernel
  clauses; listed in the KISS-Ops "Kernel-governed ops" table (§6.22 of `spec/ops.md`).
- **Reserved section** — a section number set aside here whose text is not yet published.

## 4. Normative References

- **KISS-Ops** (by version) — DAG edge labeled **STRUCTURAL**, **upstream** dependency. Supplies the op set, the atomic
  combines and the monoid set the kernel-body rules take.
- **KISS-Classify** (by version) — DAG edge labeled **STRUCTURAL**, **upstream** dependency. Supplies the capability
  vocabulary for the target-stated rows.
- **KISS-Conform** — **test dependency**: Conform tests this sub-standard like every other.
- **KISS Umbrella** — informative; §2.1 and §2.2 list this sub-standard.

## 5. Conventions

Clause IDs are `KISS-KERNEL-<section>-<nnnn>` and are append-only. A section that is **reserved** (§2.2) has no clause IDs
until the rulings that gate it are made. Quoting CireSnave's rulings is by board number, verbatim, in a Draft-change note.

## 6. Specification

### 6.0 Status of this section

Everything below §6.1 is **reserved** and carries no normative text (§2.2). The skeleton exists so that the references board
450 asked for can be built and checked before the rules arrive.

### 6.1 The reference rule

- **KISS-KERNEL-6.1-0001** — Every KISS-Ops operation whose concurrent behaviour a KISS-Kernel section governs MUST be
  listed in the "Kernel-governed ops" table of KISS-Ops (§6.22), against the KISS-Kernel sections that govern it, and
  every operation in that table MUST be named by a `Kernel-governed:` pointer in each KISS-Ops clause that fixes that
  operation's concurrent behaviour; the table MUST name only sections that exist in this document's §2.2 map. *Test:*
  `test_kernel_governed_ops_have_pointers` (the same check is the document lint `tools/kiss_kernel_refs.py`: a governed op
  with no pointer, a pointer with no table row, or a table row naming a section absent from §2.2 fails both).

## 7. Capability, Profile & Extension model

Not applicable in version 1: no capability bit is defined. A later section MAY state target rows (which subgroup sizes
exist, maximum threads, cluster support) that a target declares through KISS-Announce's per-device record, never pinned
here (KISS-CLASSIFY-6.8-0004's line: KISS pins the mechanism, not a backend's list).

## 8. Versioning & Lifecycle

KISS-Kernel versions on its own axis. Adding a clause to a reserved section is additive; changing a published clause's
meaning is a Draft change noted in the clause.

- **KISS-KERNEL-8-0001** — KISS-Kernel MUST NOT be promoted from Draft to Frozen until two structurally dissimilar
  implementations have passed its program-and-verdict vectors; the source model alone (one author's comprehension) does
  not count as two. *Test:* `test_kernel_freeze_gate_two_impls` (checklist gate; signed by the AUDIT role).

## 9. Conformance

An implementation conforms to KISS-Kernel at a given version if it meets every published clause of that version for the
sections it claims and passes the KISS-Conform suite for them. **No section is claimable in this skeleton.** The honesty
line stands until a device run exists: a verdict is "classified under the stated rules", not "behaves so on a GPU".

### 9.1 Clause → KISS-Conform test traceability matrix

| Clause ID | Named conformance test |
|---|---|
| KISS-KERNEL-6.1-0001 | `test_kernel_governed_ops_have_pointers` |
| KISS-KERNEL-8-0001 | `test_kernel_freeze_gate_two_impls` |

## 10. Governance

Changes follow the umbrella's lifecycle (umbrella §5). The scope and sub-standard question was ruled by CireSnave on board
450 (2026-11-16): *"450: A...but only as long as the necessary references are kept in place so readers know when
recommendations are available."* The six open choices are board 459.
