# Kernel body: scope-and-process note (draft for CireSnave, via the PM)

Status: **a proposal, informative, no clause.** Nothing here binds a reader; every ruling below waits on CireSnave
(CIRESNAVE-EXPECTATIONS: a new normative layer is his call). Source material:
`docs/kiss-kernel-body-concept-map.md` in the Unpopped repo (Unpopped#274, read at its `origin/main`; section numbers
below are that map's). Part of KISS#768 and KISS#769, which this note scopes but does not close.

## 1. The question

KISS-Ops says *what is computed* over whole tensors. It says nothing about *what one thread does in a launch of many*:
barriers, atomics' order and scope, fences, subgroup collectives, the race rule. Two kernels can agree on every op and
still disagree on what a program may assume. Unpopped has built a model of that layer (an interpreter, a serial-C
differential and seeded mutants; **never run on a device**, concept map sec 10). CireSnave agreed (relayed by the PM,
2026-11-03) that Unpopped's IR is the source material for a new KISS normative layer, Unpopped being one implementation.

## 2. Proposed scope

**In (v1):** the launch shape and ids (sec 1); the statement and expression core a thread executes (secs 2-3); memory
spaces and places (sec 4); barriers at Subgroup/Block/Device/Cluster with the uniformity rule (sec 5); atomics with order
and scope, fences, and the race rule as a *classification of programs* (sec 6, KISS#769); collectives and the two extra
reduce monoids (sec 7, KISS#768).

**Out (v1):** named/subset barriers, `mbarrier` and async copies, tensor memory, MMA atoms (sec 8, "honest status"), a
total-order `SeqCst`, `Constant` memory initializers, any claim about a specific GPU, and any source-language syntax
(Consume 6.1-0003 forbids blessing a language).

**Not this layer:** the ops layer, the wire formats (Contract/Classify/Announce) and every existing clause. All additions
are additive under the capability model; no `structure_key` byte moves.

## 3. Shape of the layer

Recommendation: **a new sub-standard (working name KISS-Kernel), not a section of KISS-Ops**, because its subject is one
thread's behaviour in a launch, its conformance evidence differs (programs plus verdicts, not tensor vectors), and Ops
stays free of a machine model. KISS-Ops keeps the `reduce` monoid additions of #768 (they are ops). The alternative
(a KISS-Ops section 6.20) is smaller but couples the two layers' versioning. Either way the layer follows the house
pattern: a Common/Target split (below), clause IDs, a sidecar entry per clause, and a named test per clause.

**Common vs Target.** The concept map already sorts each row. Common rules are normative (every reader shares them).
Target rows (which subgroup sizes exist, max threads, cluster support, what a Device barrier needs) are *stated by a
target*, reusing the Announce per-device record, never pinned by KISS (the 6.8-0004 line: KISS pins the mechanism, not a
backend's list).

## 4. Process (what must be true before text lands)

1. **CireSnave rules the scope and the sub-standard question** (sections 2-3). Nothing is drafted as a clause before.
2. **Decisions list (concept map sec 12, eleven items)** are ruled one by one, as options notes with a recommendation, in
   the usual way. Items that move bytes (float atomic add as a band versus the pinned KISS-CONTRACT-6.8-0012 form) are
   marked and go to him verbatim.
3. **Second implementer.** The map names Fuel, Baracuda, Vulkane (Slang/SPIR-V), and the soft-stopped SYCL/OpenCL
   emitters as candidates, unverified. A clause is *frozen* only when two independent codebases pass its vectors; until
   then it is `proposed`. The oracle-independence caveat applies: one author's comprehension is not two.
4. **Evidence standard:** a program-plus-verdict vector family (message passing with and without release/acquire, a
   grid-wide sum with and without a Device barrier, a Block-scope atomic used by two blocks), with seeded mutants for each
   clause, as Unpopped already does. Unpopped contributes its hand-written expected arrays and oracle tests.
5. **Honesty line:** the layer states "no device run" until a device run exists. A verdict is "classified under the stated
   rules", not "behaves so on a GPU".

## 5. Recommendations on the eleven Unpopped choices (sec 12), for his ruling

| # | Unpopped chose | Recommendation |
|---|---|---|
| 1 | `SeqCst` treated as `AcqRel` | Keep v1; state it; a total order is v2 |
| 2 | Shuffle out of range keeps own value | Adopt (CUDA's rule); say a target may differ only by stating it |
| 3 | Integer division/shift total | Adopt as the reference; targets state their own |
| 4 | Race = classification, named trap | Adopt: refuse or flag, never "undefined" |
| 5 | Unwritten shared/private read traps | Adopt in the reference; target-stated otherwise |
| 6 | Conservative divergence analysis | Adopt as a *sufficient* rule; a reader may be less conservative only if it never accepts what the rule rejects |
| 7 | Device barrier assumes residency | Adopt; residency is a Target row |
| 8 | Atomic old value = any consistent serialization | Adopt |
| 9 | Float atomic add is a band | **Byte-affecting**: needs his word against 6.8-0012 |
| 10 | `Constant` memory shape-only | Adopt for v1 |
| 11 | Named barriers, mbarrier, async, tensor memory, MMA absent | Keep out of v1 |

## 6. Sequence if scope is accepted

(a) rule scope and sub-standard question; (b) one options note per byte-affecting or contested item (9, 6, 1); (c) a
skeleton sub-standard with section 5 barriers and section 6 atomics first (KISS#769, smallest, Unpopped has the most
evidence); (d) collectives and monoids (KISS#768); (e) vectors from Unpopped's oracle tests; (f) a second implementer
pass. ROADMAP rows and the CONTRIBUTING affected-set rule (#484) are separate docs work.

## 7. What is not claimed

The concept map's evidence is self-consistency, not agreement with hardware. This note adds no measurement of its own and
reads the map as written; whether each Common row is truly common across vendors is exactly what step 3 tests.

— lane: kiss
