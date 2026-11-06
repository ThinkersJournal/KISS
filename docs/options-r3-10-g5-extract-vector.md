# Options note: how is Appendix A.1 vector G5 (the DAG-shared-target extract vector) authored? (ROADMAP R3.10)

**Informative. No normative text. For the PM and CireSnave; no clause is proposed, only the shape of the choice.**
Read at `origin/main` `ee4553c9` (conformance 0.6.38). The PM ruled (2026-10-30) that G5 is a design note with options
first, not code, because authoring a DAG and a least-path derivation is a spec decision.

## The question

Appendix A.1 declares G5 and gives it one paragraph (`spec/grammar.md`, the "Vector G5" paragraph): *a fusion in which a single
scalar-bearing node is reachable from the root by more than one sequence of canonical operand indices; the extract record uses the
lexicographically-least such path (§6.4-0007), and extract records are emitted in ascending `param_slot` order (§6.8-0011).*
It carries no DAG, no bytes and no op names. G1 and G4 were rendered because the appendix gave their bytes; G3 is rendered by
#738. **What would G5 be, concretely, and who decides?**

## What is already pinned (so the note does not re-open it)

- KISS-GRAMMAR-6.4-0007: extract paths are anchored on the root, are sequences of **canonical operand indices** over the
  already-canonicalized tree, and for a shared target the path MUST be the lexicographically-least sequence under unsigned
  comparison of the step indices.
- KISS-GRAMMAR-6.8-0006: a record is `path_len` (`u16`), `path_len` steps (`u16`), `param_slot` (`u32`). 6.8-0011: records
  ascend by `param_slot`; a tie on `param_slot` (which an implementation MUST NOT normally produce) breaks by least path.
- The codec (`conformance/src/grammar.rs`, the `Extract` type) **takes the path as given** and encodes it; its own comment says
  canonical-path derivation for shared targets is out of scope. So today nothing in the crate computes a least path, and
  `test_grammar_extract_channel_on_root` / `test_grammar_wire_extract_record_order` exercise encoding and ordering, not derivation.

## What is not pinned (the gaps G5 runs into)

1. **No definition of a "scalar-bearing node".** Grammar has only `Op` and `Bind` nodes (R1.25: `runtime_scalar` and
   `scan_placeholder` "have no spec home", a `[rule]` row, KISS#28/#67/#125). The extract record names a path to a node and a
   `param_slot`; nothing in Grammar says which ops have scalar params or what a slot means. A G5 that picks an op is inventing
   that vocabulary.
2. **Sharing exists in the wire form only through dedup.** §6.4-0011 collapses structurally identical subtrees to one table
   entry, so "reachable by more than one path" means the same subtree appears under two operand positions (or two parents).
   Any G5 must be a region whose canonical node table has a node with in-degree two or more *and* is the extract target.
3. **Canonical operand order interacts with the path.** Steps are indices into the **canonicalized** operand list (a `Bind`
   sorts before an `Op`, then by subtree key, §6.4-0010), so the "least path" is computed after reordering. A hand-authored
   vector can get this wrong in exactly the way an implementation can; that is the vector's value and its risk.
4. **Which side is the oracle.** If the codec derives the path and the test asserts the codec's answer, the vector is the
   codec's own output (the failure mode R3.9 lists for tests that cannot fail). The derivation needs a second, independent
   method.

## Options

**A. Wire-level G5, no scalar vocabulary.** Author one region whose canonical table has a shared interior node, point an
extract record at it with two slots (so 6.8-0011 ordering is exercised), and state in the appendix that the target's op is
chosen for sharing, not for carrying a scalar (the wire form does not check what the target is). Pin the least path by hand from
§6.4-0010 and state the other (non-least) path in the prose as the one an implementation MUST NOT emit. Render the bytes in
`grammar_vectors.json` like G1/G4. **Cost:** small; adds appendix text and one vector. **Risk:** a reader may take the op as
a scalar-bearing example; the appendix must say it is not. Does not settle gap 1.

**B. Wait for R1.25.** Leave G5 declared and unrendered until Grammar has a defined notion of a scalar-bearing node. **Cost:**
none now. **Risk:** G5 stays unrendered indefinitely (R1.25 is `[rule]`), and 6.4-0007's least-path obligation has no
vector, so a conforming-looking implementation can emit a non-least path unnoticed.

**C. A least-path derivation in the codec plus an independent check, then A.** Implement the derivation (enumerate the
root-to-target index sequences over the canonicalized tree, take the unsigned-lexicographic minimum) as a library function,
and have the test compute the same answer by a second method (for example, enumerate all paths by brute force over the decoded
node table and compare), with a seeded control that a non-least path is rejected. **Cost:** the largest (new code, a decoder
check, two methods). **Risk:** the codec decoder would start to *validate* extract paths, a new obligation not in §6.8 today,
so that part is itself a spec decision (does a reader MUST reject a non-least path, or only a writer MUST NOT emit one?).

**D. Demote G5 to prose.** Say in Appendix A.1 that G5 is illustrative and carries no bytes, and remove it from the declared
set. **Cost:** a spec edit. **Risk:** the least-path clause stays untested at the vector level, which is the gap this row is
about.

## Recommendation

**A, then C if CireSnave wants the reader obligation.** A closes the missing vector with no new vocabulary and states its own
limit; C is only worth building once the reader-side question (is a non-least path a decline?) is ruled. Do B or D only if the
answer is that the least-path rule itself should move.

## What I have not checked

- Which KISS-Ops ops (if any) carry runtime scalar params and would be the natural target; I did not read the op catalog for
  this, and gap 1 says Grammar has no vocabulary for it.
- Whether Fuel, Baracuda or Unpopped emit extract records today and whether they pick the least path (I read none of their
  code); a consumer that already emits non-least paths would turn A into a Breaking finding for it.
- The decoder's current handling of extract paths (`grammar.rs`, the extract parse near line 533): it may already validate
  step bounds; I did not trace it.
