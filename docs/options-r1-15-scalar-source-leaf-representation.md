# Options: how the seven §6.12 scalar-source leaves are represented in the Semantics `op_dag` (R1.15, KISS#411)

Status: **options with a recommendation; nothing here is a clause.** Prepared 2026-10-31 against `origin/main` 0208b57 (conformance 0.7.1). The other half of R1.15 (KISS#412, the verdict for a surfaced shape gap) is done as a clause in the same PR; this half is not, because the choice below changes the serialized `op_dag` bytes.

## What is measured

- KISS-OPS-6.12-0001 names seven leaves: `input(i)`, `const(bits)`, `param(i)`, `coord(axis)`, `reduced(stage)`, `extent(axis)`, `reduced_count`. It says they "are not themselves ops".
- KISS-CONTRACT-6.4-0009 closes the node schema to `Op{op_name, op_attrs, child_edges} | Bind(positional_index)` and forbids any other node kind or field. `op_name` must be a bare KISS-Ops op name. §6.11-0007 renders exactly those two forms to text.
- `Bind` can carry `input(i)`. Nothing states how the other six appear in a DAG, yet §6.4-0005 resolves every non-primitive node to the §6.13 floor, whose decompositions use `const` (about 15 ops) and `reduced_count`/`extent` (means).
- Two implementations diverged (KISS#411): kiss-ref made `const` a node kind (`Node::ConstBits`, semver-breaking); Fuel carries it as an op attribute (`const_bits`) on a `Const` leaf token, declared but not yet projected from any Fuel graph op (Fuel's own comment), so it is a design reference, not evidence that a round trip works.
- The "KISS four-leaf ack, 2026-07-23" cited in Fuel's source: `git grep -i "four-leaf"` over `spec docs rfcs ROADMAP.md` at 0208b57 finds 0 hits (the same grep form finds "scalar-source", which is present). Not found in KISS's record.

## Options

- **A. Node kinds.** Add `Const{bits}`, `Param(i)`, `Coord(axis)`, `Reduced(stage)`, `Extent(axis)`, `ReducedCount` to the closed schema (kiss-ref's route). New forms in §6.11-0007 text; every reader changes.
- **B. Leaf tokens as `Op` nodes.** Each leaf is an `Op` node with a reserved leaf name (`const`, `param`, `coord`, `reduced`, `extent`, `reduced_count`), no children, and its value in `op_attrs` (`bits=0x…`, `index=…`, `axis=…`). `input(i)` stays `Bind`. The schema stays at two node kinds and §6.11-0007's grammar is unchanged; what changes is that `op_name` may be one of six reserved leaf tokens, an exception to "a bare KISS-Ops op name" (they are not ops, §6.12-0001).
- **C. Mixed.** `const` as a node kind (A), the rest as tokens (B). Rejected: two mechanisms for one concept.

## Obligations any option must carry

- `const` bits: the exact dtype bit pattern in the dtype's width, upper bits zero, read back verbatim; ±∞, quiet and signaling NaN, ±0 and subnormals round-trip, and a NaN payload is never quieted or canonicalized (§6.12-0002, §6.2-0008). Without this carried into the Contract side, two readers can disagree on a NaN literal.
- A leaf token must be distinguishable from an op name: reserve the six names so no KISS-Ops op can take one.
- Leaves have no `child_edges`; `reduced(stage)` names a stage that must exist earlier in the DAG (state the rule or leave it to Ops).

## Recommendation: B

It is the only option that adds no node kind and no new serialization form, keeps §6.4-0009's "MUST NOT introduce a node kind" intact, and is what the second implementation already built. Cost: a reserved-name exception to `op_name`, and the Contract-side restatement of the `const` bit rules.

**Byte consequence (why this waits):** under B, a DAG using `recip` now serializes an `Op{const; bits=…; []}` node it could not express before; under A, a new node form. Either changes Contract bytes for those ops. Per the standing rule for byte-altering carriers, no clause until CireSnave's word arrives through the PM.
