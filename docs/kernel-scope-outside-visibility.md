# KISS-Kernel scope: what is outside-visible (options note, board 474)

Status: **informative, no clause.** Answers the pm-kernels TASK of 2026-11-20 under board 474, which CireSnave ruled
"474: 2" (reframe KISS-Kernel as the outside-visible contract; drop what matters only inside a thread). He rules on the
lists in sections 3 and 4. No clause text is written, and board 459 stays on hold. Read at `origin/main` dd51c2c:
`spec/kernel.md` (skeleton), `docs/kernel-body-scope-note.md`, `spec/umbrella.md`, `spec/ops.md`, `spec/contract.md`.
I did **not** re-read Unpopped's concept map for this note: the item list below is `kernel.md` section 1 plus the eleven
choices as `docs/kernel-body-scope-note.md` section 5 states them.

## 1. The test

CireSnave, 2026-11-20, quoted in the TASK: *"…what must be true inside each kernel… Is all of that 'kernel internals
that cause effects in other kernels' part of things already defined elsewhere in KISS sufficiently? If not, this
sub-standard could become that specification...but if it isn't needed for that, KISS shouldn't be defining what happens
inside an individual kernel…"*

**Working predicate.** A rule is **outside-visible** if some party other than the executing thread (another thread of the
launch, another kernel, or the caller) may rely on it or must obey it. It is **internal-only** if it fixes only what one
thread computes or how a kernel is written, so that no outside party could tell two conforming kernels apart by it.
**Mixed** means an outside-visible obligation sits on top of an internal mechanism: keep the obligation, drop the mechanism.

## 2. What exists elsewhere (verified at dd51c2c)

- Outside `spec/kernel.md`, the words barrier / race / memory order are used only as pointers **to** KISS-Kernel
  (`ops.md`: the "Kernel-governed" tags on the scatter combine, KISS-OPS-6.11-0006/-0010/-0024; `umbrella.md`: the
  KISS-Kernel row). Control: `git grep -i barrier` finds those, so the query works; it finds no definition anywhere else.
  A grep for happens-before, enqueue, submission, in-order or hazard finds nothing relevant outside `kernel.md`.
- Ops pins the **result** of the scatter atomic combines, not their order or reach.
- **Umbrella 1.2 excludes "in-ecosystem kernel loading and dispatch"** as the execution ecosystem's own business
  (command submission, streams, events). This bounds any cross-launch rule: KISS may state what a kernel *requires and
  guarantees at its boundary*; it may not define how launches are ordered.
- Contract states one footprint fact: `in_place` (KISS-CONTRACT-6.5-0009), whether the output aliases an input. It states
  no per-operand read/write set, so a caller cannot tell from the Contract which buffers a kernel writes.
- The draft's Device barrier spans the blocks of **one** launch, not several kernels (kernel.md section 1; PM finding).

## 3. Every v1 in-scope item

| Item (kernel.md section 1 / choice #) | Class | Why | Recommendation |
|---|---|---|---|
| Launch shape and ids | mixed | The group/grid extents are caller-set and fix who can synchronize; thread-id arithmetic is internal | **Reframe**: keep only the terms the reach levels need (thread, subgroup, block, device); drop id derivation |
| Statement and expression core | internal-only | What a thread executes; values are KISS-Ops' business; umbrella 1.2 says KISS never dictates how a kernel computes | **Drop** (back to Unpopped as the source model) |
| Memory spaces and places | mixed | Which spaces are visible to whom (device memory: other kernels and host; shared: one block; private: nobody) is a contract; addressing and layout of private memory is internal | **Reframe** as a visibility table; drop the rest |
| Barriers, four reach levels | outside-visible | The partners are other threads, and the uniformity rule (every thread arrives) is a promise those partners rely on for freedom from deadlock | **Keep**; state as obligation plus guarantee |
| Atomics: order and reach | outside-visible | Order and scope decide what another thread or kernel may observe | **Keep**; scope must grow to name cross-launch reach (section 4b) |
| Fences | outside-visible | Same | **Keep** |
| Race rule (classification of programs) | outside-visible | Which concurrent accesses are errors is what every co-running party needs | **Keep**; extend beyond one launch (section 4c) |
| Group collectives | mixed | Participation (all arrive) is a barrier-class obligation; the result values are op semantics | **Reframe**: keep participation here; values stay in Ops (the monoid additions of #768 already sit there) |
| Reading memory nobody wrote | mixed | For device memory it is visible to the caller and other kernels; for shared/private it is internal | **Reframe**: device memory only |
| #1 `SeqCst` treated as `AcqRel` | outside-visible | An ordering guarantee | **Keep** (state it; total order stays v2) |
| #2 shuffle out of range keeps own value | internal-only | A value rule of one collective inside a group | **Drop** (Ops or a target row) |
| #3 integer division/shift total | internal-only | Arithmetic of one thread | **Drop** |
| #4 race = classification, named trap | outside-visible | See race rule | **Keep** |
| #5 unwritten shared/private read traps | internal-only | One thread or one block; no outside party sees it | **Drop** for shared/private; device memory per the row above |
| #6 conservative divergence analysis | internal-only (mechanism) | The outside-visible part is the uniformity obligation; the analysis is how a checker enforces it | **Reframe**: keep the obligation; state the analysis, if at all, as an informative sufficient check |
| #7 Device barrier assumes residency | outside-visible | The caller or launcher must guarantee co-residency; a barrier partner can be starved otherwise | **Keep** as a requirement on the launch (a Target row); see 4a |
| #8 atomic old value = any consistent serialization | outside-visible | What a racing observer may see | **Keep** |
| #9 float atomic add is a band | outside-visible (value) | **Already ruled A and landed** as KISS-OPS-6.11-0024 | nothing to do |
| #10 `Constant` memory shape-only | internal-only | An initializer detail | **Drop** |
| #11 named barriers, mbarrier, async copy, tensor memory, MMA | out of v1 already | | unchanged |

Tally: 8 keep or reframe-to-keep on the outside-visible spine (barriers, atomics, fences, race rule, #1, #4, #7, #8),
5 reframe (ids, memory spaces, collectives, unwritten reads, #6), 5 drop (expression core, #2, #3, #5 private part, #10).

## 4. CireSnave's three cross-kernel cases

| Case | Covered by the draft? | Verdict |
|---|---|---|
| (a) a barrier spanning several kernels | **No.** The draft's Device barrier spans blocks of one launch | **New clauses.** Kernels need not be co-resident, so the rule is probably negative or conditional: a barrier MUST NOT span launches unless the launch is stated co-resident (a cooperative-launch Target row, same family as #7). Question for him: does the Contract get a field saying a kernel waits on another's progress? |
| (b) memory order across parallel or reorderable kernels | **No.** Atomic scope stops at the device within one launch; ordering between launches is the ecosystem's (umbrella 1.2) | **New clauses, boundary-only.** State what a kernel assumes at entry (it sees prior launches' writes only if the caller ordered them) and guarantees at exit (its writes are visible to anything ordered after it). Do not define the ordering mechanism |
| (c) data races between two concurrently running kernels on the same memory | **Partly.** The race classification exists but ranges over threads of one launch | **New clause, extended rule.** Accesses by threads of two launches race unless the caller ordered the launches. Needs a footprint the Contract does not carry (only `in_place` exists), so either a new Contract field (read set, write set) or the rule stays conditional on caller knowledge |

Cases (a) to (c) are all **new**; none is already defined elsewhere in KISS (section 2). Each would also need program-and-verdict
vectors and, per the freeze gate (KISS-KERNEL-8-0001), two dissimilar implementers; Unpopped's model is thread-within-one-launch,
so it supplies no cross-launch evidence.

## 5. What this does to the reserved map (kernel.md 2.2)

6.2 shrinks to the launch terms and the visibility table; 6.3 to 6.5 stay and gain a cross-launch reach; 6.6 keeps
participation only; 6.7 narrows to device memory; **a new section for the launch boundary** (entry and exit guarantees,
co-residency, footprint) is the only addition. Title and section 1 would change from "What One Thread Does in a Launch of
Many" to a contract about what kernels may rely on from each other.

## 6. Decisions for CireSnave

1. Accept the keep / reframe / drop column of section 3 (the 5 drops go back to Unpopped as source material).
2. Cases (a) to (c): which become clauses, given umbrella 1.2 keeps ordering mechanisms with the ecosystem. Recommend **(b) boundary-only and (c) conditional**, and **(a) as the negative rule only**.
3. Whether the Contract grows a read/write footprint (the price of (c); it changes Contract bytes, so his word).

## 7. Not claimed

No device run exists. Section 3 classifies items by the predicate in section 1, which is mine; a different predicate moves
rows (the shuffle rule, #2, is outside-visible to a thread in the same group, if "outside" is read as "another thread").

— lane: kiss
