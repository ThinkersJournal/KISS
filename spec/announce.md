# KISS-Announce — Provider Discovery, Kernel Availability & Contract-Query Kickoff

**Sub-standard ID:** KISS-ANNOUNCE
**Part of:** KISS — Kernel Interface Standards Suite
**Steward:** ThinkersJournal (non-profit public-standards publisher)
**This document:** First-draft proposal. Not ratified. Not frozen.

> This document follows the KISS dual-doc template defined in the *KISS Design
> Charter*: an **informative Overview** (§0–§5) and a **normative Conformance
> specification** (§6+). Only §6+ is normative. Normative clauses use RFC-2119 /
> RFC-8174 uppercase keywords, carry an append-only clause ID
> `KISS-ANNOUNCE-<section>-<nnnn>`, and each maps 1:1 to at least one named
> KISS-Conform test. The KISS-Conform suite build FAILS on any normative MUST
> without a mapped test.

---

## 0. Front-matter

| Field | Value |
|---|---|
| Title | KISS-Announce |
| Sub-standard ID | KISS-ANNOUNCE |
| Maturity stage | **Draft** (first-draft proposal; wire shape NOT frozen) |
| Editor of record | **Proposed, pending ratification** — a provider/consumer-side project holds the pen and requests comment from interested cosignatories (the ratified governance record in the Charter does not yet name an editor for KISS-Announce). |
| Steward | ThinkersJournal |
| Reference seed crate(s) | two byte-identical seeds (`SeamHello`) living in separate project workspaces, pending convergence to one canonical registry-published KISS-Announce crate (project names given in Appendix A.2 as non-normative provenance) |
| DAG position | **Protocol tier.** Depends (opaquely) on KISS-Classify and KISS-Contract; consumed downstream by KISS-Synth. Not a root. |
| Upstream edges | KISS-Classify (**OPAQUE** — supplies the `structure_key` token, carried without interpretation); KISS-Contract (**OPAQUE** — supplies the contract document returned by a contract-query) |
| Downstream edges | KISS-Synth/provision (**OPAQUE** — contract-query is the already-exists / build-on-miss branch of provision; Synth consumes the query protocol) |
| Spec license | CC0 1.0 Universal (public-domain dedication) |
| Reference-crate license | MIT-OR-Apache-2.0 |

> **Edge-label note (informative).** The Announce↔Classify edge is labeled
> **OPAQUE**: KISS-Announce carries `structure_key` as an uninterpreted,
> length-delimited token and never parses its internals. This label is reconciled
> with the umbrella suite document (§2.1, §2.2), which labels the same edge OPAQUE.

---

## 1. Purpose & Scope

KISS-Announce is the **discovery seam** of the suite: the protocol-tier node across
which a **provider** (a party that holds or can build kernels) and a **consumer** (a
graph, runtime, or compiler that wants a kernel) get talking. It depends opaquely on
the **data** vocabulary (KISS-Classify, source of `structure_key`) and the
**contract** format (KISS-Contract, the document a query returns); it does not define
either. Across it the two sides perform three staged acts, from coarse to fine:

1. **Provider handshake** — the two sides exchange a fixed-size, POD *handshake
   envelope* and negotiate the highest mutually-supported profile plus a set of
   provider-level capabilities (which sub-standards each speaks, whether the
   provider answers contract queries, whether it builds on miss).
2. **Kernel availability** — the provider names the kernels it can serve by
   **identity only** — the pair `(structure_key, revision_hash)` — so a consumer
   can tell a cache **hit** from a **miss** without downloading anything heavy.
3. **Contract-query kickoff** — on a miss, the consumer requests the full contract
   for a `structure_key` (optionally pinned to a `revision_hash`); the provider
   returns it (the already-exists branch of KISS-Synth provision) or returns a typed
   decline.

KISS-Announce owns the *envelope byte layout*, the *version-negotiation algorithm*,
the *capability bitset structure*, the *identity-only availability record*, and the
*framing of every message* (handshake envelope, availability list, contract-query
request/response). It is deliberately small: it gets the two parties talking and
hands off.

**KISS-Announce is NOT:** a kernel library; a compiler or IR; the per-kernel
*contract* format (that is KISS-Contract); the *data* vocabulary
(`structure_key`/`OperandDesc` internals — that is KISS-Classify); the *op*
vocabulary (KISS-Ops); the JIT build mechanics (KISS-Synth); a transport/session
layer (TCP, IPC, shared memory are all out of scope — KISS-Announce specifies
*bytes and message framing*, not the pipe, and does not specify delivery ordering
or reliability); and it does **NOT** carry per-kernel capability, usage, or
semantics in the announce (those live in the queried contract, single source of
truth).

---

## 2. Overview / Rationale (informative)

### 2.1 The mental model

Think of a provider that may hold **thousands** of specialized kernel cells (one per
layout/dtype/arch specialization). Pushing a full contract for every cell into the
first handshake would be wasteful and would duplicate facts that already have a home
in the contract. So KISS-Announce follows an **LSP-style negotiate-then-request**
shape:

- The handshake envelope is *tiny and fixed* — provider-level capability only.
- Availability is *identity only* — just enough for a consumer to de-duplicate
  against its own cache (`structure_key` says "which cell", `revision_hash` says
  "which build of it").
- The heavy per-kernel description is *fetched on demand* on a cache miss.

This is why the announce carries identity, not capability: capability duplicated in
the announce would drift from the contract. The contract is the single source of
truth; the announce is a de-dup index.

### 2.2 Why a fixed-size POD envelope, hard-rejected

The handshake must be reproducible byte-for-byte by a C, SPIR-V, or plain-CPU
implementor who has never seen the reference language. A fixed-size Plain-Old-Data
structure with pinned offsets and little-endian fields is the smallest thing that
survives crossing a language and pointer-width boundary. Because it is POD, the
reader discipline is **hard-reject**: an unknown magic, an unexpected length, a
nonzero reserved region, or an over-cap profile count means the reader **refuses the
envelope with a typed decline** — it never tolerates garbage, never repairs, never
panics. This is the opposite of the soft-skip tier that JSON/text/telemetry channels
use (ignore-unknown); for a byte-pinned POD, ignoring garbage is a security and
correctness hole. The one exception is the `capabilities` bitset (§7.2), which is a
pure forward-compatible *feature-advertisement* field: unknown bits there are ignored,
not rejected, and no bit in it may hard-gate the peer.

### 2.3 The reserved structure padding is real

A reference seed using C-compatible field ordering lays out the envelope so that
between the `profiles` array (which ends at offset 42) and the 8-byte-aligned
`capabilities` field (which must start at offset 48) the compiler inserts **6 bytes
of implicit alignment padding**. That padding is invisible in the native type but is
a first-class, must-be-zero region of the wire format. A foreign implementor who lays
the fields out packed would produce a 50-byte structure and fail interop. This spec
makes the padding **explicit** (§6.1, field `reserved1`) precisely so the
adversarial-outsider / foreign-reader freeze gate can catch it.

### 2.4 4-character codes are wire-order ASCII

Every 4-character code in KISS-Announce (the envelope magic and every message tag)
is defined the **same way**: as four ASCII bytes written **in wire order** (the first
character at the lowest offset), which — when the same four bytes are read as a
little-endian `u32` — yields the numeric constant. The wire bytes are the source of
truth; the numeric value is a derived convenience. The table below gives both for
every code so no reader can drift:

| Code | ASCII (wire order, low→high offset) | Wire bytes | u32 LE value |
|---|---|---|---|
| `magic` | `S E A M` | `53 45 41 4D` | `0x4D414553` |
| query request tag | `C Y R Q` | `43 59 52 51` | `0x51525943` |
| contract response tag | `C R S P` | `43 52 53 50` | `0x50535243` |
| decline response tag | `C D E C` | `43 44 45 43` | `0x43454443` |
| availability-list tag | `S A V L` | `53 41 56 4C` | `0x4C564153` |

### 2.5 Worked handshake example (informative)

The reference provider (project name used only in this non-normative example)
advertises: envelope version 1; one profile (integer `1`); capabilities = the six
external DLPack tokens (EXT bits 0–5) plus provision-on-request (FEAT bit 32) plus
contract-query (FEAT bit 33). Its 56-byte envelope serializes to exactly these bytes:

```
offset  bytes                     field                value
0x00    53 45 41 4D               magic (u32 LE)       0x4D414553  (ASCII "SEAM", wire order)
0x04    01                        envelope_version     1
0x05    00 00 00                  reserved0 (MBZ)      0
0x08    01 00                     profiles_len (u16LE) 1
0x0A    01 00                     profiles[0] (u16 LE) 1
0x0C    00 00 ... 00 (30 bytes)   profiles[1..16]      0
0x2A    00 00 00 00 00 00         reserved1 pad (MBZ)  0
0x30    3F 00 00 00 03 00 00 00   capabilities (u64LE) 0x0000_0003_0000_003F
                                                       (EXT bits 0-5 | FEAT bit 32 | FEAT bit 33)
--------------------------------------------------------------
total   56 bytes; native mirrors 8-byte aligned
```

A consumer reading this: reads `magic` and `envelope_version` from the fixed prefix;
checks `magic == 0x4D414553`; checks that it supports `envelope_version == 1` and that
the input is the 56 bytes version 1 mandates; checks both reserved regions are
all-zero; reads `profiles_len == 1`, `profiles[0] == 1`; intersects `{1}` with its own
profile set; if `1` is mutual, selects profile 1; inspects `capabilities` to learn the
provider provisions on miss (bit 32) **and** answers contract queries (bit 33); then
moves to the availability + contract-query stages.

### 2.6 Terms are joined, not restated

KISS-Announce references `structure_key`, the *contract*, and the KISS-Ops/Classify
vocabularies as **opaque** tokens. It specifies how to *carry* and *frame* them, never
what is *inside* them — those are KISS-Classify and KISS-Contract respectively.

---

## 3. Terms & Definitions

- **Provider** — a party that can serve a kernel identity and, on request, its
  contract; may build on miss.
- **Consumer** — a party that discovers what a provider offers and requests
  contracts for cache misses.
- **Implementation** — any software producing or parsing KISS-Announce wire
  artifacts.
- **Handshake envelope** — the fixed-size 56-byte POD structure defined in §6.1.
- **Profile** — an integer `>= 1` denoting a mutually-versioned seam feature-set;
  the unit of version negotiation (§7.1). The value `0` denotes absence/padding and
  is never a live profile.
- **Capability bitset** — the 64-bit field carrying provider-level, negotiable
  optional features and axis flags (§7.2). **Not** per-kernel. A pure
  feature-advertisement field with no hard-gate bits.
- **structure_key** — an opaque admissibility token owned by KISS-Classify,
  identifying a layout/dtype/arch specialization cell. Treated here as an opaque,
  length-delimited byte token.
- **revision_hash** — a fixed 32-byte, opaque, provider-assigned identifier of a
  specific build/revision of the kernel behind a `structure_key`, compared only for
  byte-for-byte equality (no hash algorithm implied).
- **Availability record** — the identity pair `(structure_key, revision_hash)`.
- **Contract** — the per-kernel document owned by KISS-Contract; treated here as an
  opaque, self-delimiting payload.
- **Typed decline** — a structured refusal drawn from the enumerable decline-code set
  of §6.4, returned in lieu of success. Never a panic, abort, crash, hang, or
  out-of-bounds read.
- **MBZ** — Must Be Zero: a reserved region a producer MUST zero and a reader MUST
  reject if nonzero.
- **Little-endian (LE)** — least-significant byte at the lowest offset.

---

## 4. Normative References

- **RFC 2119 / RFC 8174** — normative keyword interpretation (uppercase only).
- **IEEE 754-2019** — floating-point (referenced by downstream sub-standards; not
  used by the announce wire, which carries only integers/opaque bytes).
- **C-compatible type layout (the reference-language `repr(C)`/`extern "C"`
  guarantee)** — the C-compatible field-ordering and alignment guarantee used to
  *derive* the seed layout. **Normative only as the derivation basis**: the spec is
  the byte layout in §6.1, **not** any language type. Any layout-vs-byte-layout
  disagreement is resolved in favor of §6.1.
- **KISS Design Charter** — the umbrella conventions: RFC-2119 keyword convention,
  the normative/informative split, the clause-ID scheme and 1:1 test mapping, the
  two version axes, the ≥2-dissimilar-implementations-plus-foreign-reader freeze
  gate, the capability/profile/extension model, governance, licensing, and patent
  posture. **Stated once in the Charter; referenced here; never restated.**
- **KISS-Classify** (by version) — DAG edge labeled **OPAQUE**, **upstream**
  dependency: source of the `structure_key` token; the announce carries it without
  interpretation.
- **KISS-Contract** (by version) — DAG edge labeled **OPAQUE**, **upstream**
  dependency: the contract document returned by a contract-query.
- **KISS-Synth** (by version) — DAG edge labeled **OPAQUE**, **downstream**
  consumer: contract-query is the already-exists / build-on-miss case of provision.
- **External interchange-token registry** (DLPack/FDX family, by version) — the
  external source of truth for EXT-axis token meanings, mirrored in the KISS
  capability registry (§7.2). KISS pins only the EXT axis/range, never the external
  vocabulary's semantics.

---

## 5. Conventions

This sub-standard adopts the KISS Design Charter's keyword convention and clause-ID
rules verbatim. Per the Charter: normative §6+ uses **only** uppercase
`MUST`/`MUST NOT`/`SHALL`; `SHOULD`/`MAY` are reserved for governance and consumer-
behavior guidance and never state a byte-level wire fact. Every atomic requirement
carries a stable, append-only ID `KISS-ANNOUNCE-<section>-<nnnn>`, allocated by the
editor of record, never reused after retirement, and mapped 1:1 to ≥1 named
KISS-Conform test. Unquantified adjectives ("well-formed", "reasonable", "neutral")
are banned from normative text. See the Charter for the full statement.

---

# NORMATIVE CONFORMANCE SPECIFICATION (§6+)

## 6. Specification

### 6.0 Determinism / fidelity class

- **KISS-ANNOUNCE-6.0-0001** — Every numeric or byte-valued obligation in §6–§8
  (sizes, offsets, magic, tags, decline codes, hashes, bounds, bitset positions) is
  determinism-class **exact byte compare**; KISS-Conform MUST evaluate each such
  clause with a byte-exact comparator and MUST NOT apply tolerance or order-invariant
  comparison. *Test:* `test_announce_determinism_class_exact_byte`.

### 6.1 Handshake envelope byte layout

The handshake envelope is a fixed-size Plain-Old-Data structure. Its normative
definition is the following byte layout (all offsets in bytes from the start of the
envelope; all multi-byte integers little-endian):

| Offset | Size | Field | Type | Constraint |
|---|---|---|---|---|
| 0 | 4 | `magic` | u32 LE | `== 0x4D414553` (ASCII `SEAM`, §2.4) |
| 4 | 1 | `envelope_version` | u8 | producer writes `1` for this version |
| 5 | 3 | `reserved0` | u8[3] | MBZ |
| 8 | 2 | `profiles_len` | u16 LE | `<= 16` |
| 10 | 32 | `profiles` | u16[16] LE | live entries `>= 1`, ascending; trailing entries zero |
| 42 | 6 | `reserved1` | u8[6] | MBZ (alignment padding) |
| 48 | 8 | `capabilities` | u64 LE | per §7.2 |
| — | **56** | (total) | — | fixed 56 contiguous bytes |

- **KISS-ANNOUNCE-6.1-0001** — The handshake envelope MUST be exactly 56 bytes.
  *Test:* `test_announce_envelope_size_is_56`.
- **KISS-ANNOUNCE-6.1-0002** — A producer MUST serialize the envelope as exactly 56
  contiguous bytes with each field at the offset in the table above (the wire form
  carries no in-memory alignment requirement; native 8-byte alignment of the
  `capabilities` mirror is an informative implementation note, not a wire clause).
  *Test:* `test_announce_envelope_wire_is_56_contiguous`.
- **KISS-ANNOUNCE-6.1-0003** — Every field MUST occupy the exact offset in the table
  above. *Test:* `test_announce_field_offsets_match_table`.
- **KISS-ANNOUNCE-6.1-0013** — Every field MUST occupy the exact size in the table
  above. *Test:* `test_announce_field_sizes_match_table`.
- **KISS-ANNOUNCE-6.1-0004** — The `magic` field MUST equal `0x4D414553` when read as
  a little-endian u32 (on-wire bytes `53 45 41 4D`, ASCII `SEAM`). *Test:*
  `test_announce_magic_constant`.
- **KISS-ANNOUNCE-6.1-0014** — The `magic` field (offset 0, size 4) and the
  `envelope_version` field (offset 4, size 1) MUST occupy those fixed offsets and
  sizes in **every** envelope version, so a reader can dispatch on version before
  applying any version-specific length or field check. *Test:*
  `test_announce_fixed_prefix_stable_across_versions`.
- **KISS-ANNOUNCE-6.1-0005** — A producer conforming to this envelope version MUST
  write `1` to `envelope_version`. *Test:* `test_announce_version_field_is_1`.
- **KISS-ANNOUNCE-6.1-0006** — A producer MUST write all-zero bytes to `reserved0`
  (offset 5, length 3). *Test:* `test_announce_reserved0_is_zero`.
- **KISS-ANNOUNCE-6.1-0007** — A producer MUST write a `profiles_len` value that is
  `<= 16`. *Test:* `test_announce_profiles_len_within_cap`.
- **KISS-ANNOUNCE-6.1-0008** — A producer MUST write zero to every `profiles` entry
  at index `>= profiles_len`. *Test:* `test_announce_trailing_profiles_zero`.
- **KISS-ANNOUNCE-6.1-0015** — A producer MUST write a value `>= 1` to every
  `profiles` entry at index `< profiles_len` (the value `0` is reserved for
  absence/padding and MUST NOT be a live profile). *Test:*
  `test_announce_live_profiles_nonzero`.
- **KISS-ANNOUNCE-6.1-0009** — A producer MUST write `profiles[0..profiles_len]` in
  strictly ascending order with no duplicate values. *Test:*
  `test_announce_profiles_strictly_ascending`.
- **KISS-ANNOUNCE-6.1-0010** — A producer MUST write all-zero bytes to `reserved1`
  (offset 42, length 6, the alignment padding). *Test:* `test_announce_reserved1_pad_zero`.
- **KISS-ANNOUNCE-6.1-0011** — The `capabilities` field MUST occupy offset 48 as an
  8-byte little-endian unsigned integer. *Test:* `test_announce_capabilities_field`.
- **KISS-ANNOUNCE-6.1-0012** — Every multi-byte integer field MUST be encoded
  little-endian. *Test:* `test_announce_all_fields_little_endian`.

### 6.2 POD reader discipline (hard-reject)

- **KISS-ANNOUNCE-6.2-0010** — A reader MUST read `magic` (offset 0) and
  `envelope_version` (offset 4) from the fixed prefix (§6.1-0014) before applying any
  version-specific length or field validation. *Test:*
  `test_announce_reads_prefix_before_length`.
- **KISS-ANNOUNCE-6.2-0001** — A reader MUST reject, with a typed decline, any input
  whose length is not exactly the length mandated by its `envelope_version` (56 bytes
  for version 1). *Test:* `test_announce_reject_wrong_length_for_version`.
- **KISS-ANNOUNCE-6.2-0002** — A reader MUST reject, with a typed decline, any
  envelope whose `magic` is not `0x4D414553`. *Test:* `test_announce_reject_bad_magic`.
- **KISS-ANNOUNCE-6.2-0003** — A reader MUST reject, with a typed decline, any
  envelope whose `envelope_version` it does not support. *Test:*
  `test_announce_reject_unknown_version`.
- **KISS-ANNOUNCE-6.2-0004** — A reader MUST reject, with a typed decline, any
  envelope in which `reserved0` contains a nonzero byte. *Test:*
  `test_announce_reject_nonzero_reserved0`.
- **KISS-ANNOUNCE-6.2-0011** — A reader MUST reject, with a typed decline, any
  envelope in which `reserved1` contains a nonzero byte. *Test:*
  `test_announce_reject_nonzero_reserved1`.
- **KISS-ANNOUNCE-6.2-0005** — A reader MUST reject, with a typed decline, any
  envelope whose `profiles_len` is greater than 16. *Test:*
  `test_announce_reject_profiles_len_overflow`.
- **KISS-ANNOUNCE-6.2-0006** — A reader MUST reject, with a typed decline, any
  envelope whose `profiles[0..profiles_len]` are not in strictly ascending order.
  *Test:* `test_announce_reject_non_ascending_profiles`.
- **KISS-ANNOUNCE-6.2-0012** — A reader MUST reject, with a typed decline, any
  envelope in which a `profiles` entry at index `>= profiles_len` is nonzero. *Test:*
  `test_announce_reject_nonzero_trailing_profiles`.
- **KISS-ANNOUNCE-6.2-0013** — A reader MUST reject, with a typed decline, any
  envelope in which a `profiles` entry at index `< profiles_len` equals `0`. *Test:*
  `test_announce_reject_zero_live_profile`.
- **KISS-ANNOUNCE-6.2-0007** — On any rejection, a reader MUST return a typed decline
  and MUST NOT panic, abort, crash, hang, or read outside the input buffer. *Test:*
  `test_announce_rejection_is_typed_decline`.
- **KISS-ANNOUNCE-6.2-0008** — A reader MUST NOT tolerate, silently ignore, or
  attempt to repair a malformed envelope (this hard-reject discipline is distinct
  from the soft-skip / ignore-unknown discipline of text and telemetry channels and
  from the ignore-unknown rule for the `capabilities` bitset in §7.2-0007). *Test:*
  `test_announce_reader_never_repairs`.

### 6.3 Kernel availability (identity only)

- **KISS-ANNOUNCE-6.3-0001** — A provider MUST announce each available kernel as an
  **availability record** consisting solely of the pair `(structure_key,
  revision_hash)`. *Test:* `test_announce_availability_is_identity_pair`.
- **KISS-ANNOUNCE-6.3-0002** — An availability record MUST NOT carry any per-kernel
  capability, usage/ABI, dispatch, guarantee, or semantics field; such facts are
  carried only by the queried contract. *Test:*
  `test_announce_availability_carries_no_capability`.
- **KISS-ANNOUNCE-6.3-0003** — `revision_hash` MUST be exactly 32 bytes. *Test:*
  `test_announce_revision_hash_is_32_bytes`.
- **KISS-ANNOUNCE-6.3-0007** — A consumer MUST compare `revision_hash` for equality
  byte-for-byte over all 32 bytes. *Test:* `test_announce_revision_hash_compared_bytewise`.
- **KISS-ANNOUNCE-6.3-0008** — `revision_hash` MUST be treated as an opaque
  provider-assigned identifier compared only for equality; an implementation MUST NOT
  assume any particular hash algorithm or recomputable input domain from the announce
  (artifact verification, if any, is defined by KISS-Synth/KISS-Contract, not here).
  *Test:* `test_announce_revision_hash_opaque_identity`.
- **KISS-ANNOUNCE-6.3-0004** — A provider MUST carry `structure_key` as the opaque,
  length-delimited KISS-Classify token and MUST NOT reinterpret, truncate, or
  re-encode its bytes. *Test:* `test_announce_structure_key_is_opaque`.
- **KISS-ANNOUNCE-6.3-0011** — An availability list MUST begin with the 4-byte tag
  `SAVL` (wire bytes `53 41 56 4C`, u32 LE `0x4C564153`) at offset 0. *Test:*
  `test_announce_availability_list_tag`.
- **KISS-ANNOUNCE-6.3-0012** — Immediately after the tag, an availability list MUST
  carry a 1-byte `list_version` (producer writes `1`) followed by 3 MBZ bytes; a
  reader MUST reject a nonzero MBZ byte or an unsupported `list_version` with a typed
  decline. *Test:* `test_announce_availability_list_version`.
- **KISS-ANNOUNCE-6.3-0005** — After the `list_version` block, an availability list
  MUST be framed as a little-endian u32 `record_count` followed by that many
  availability records, each record being a little-endian u32 `structure_key`
  byte-length, the `structure_key` bytes, then the 32-byte `revision_hash`. *Test:*
  `test_announce_availability_framing`.
- **KISS-ANNOUNCE-6.3-0009** — A producer MUST write a per-record `structure_key`
  byte-length in the inclusive range `[1, 4096]` (`MAX_STRUCTURE_KEY_LEN = 4096`;
  empty keys are not permitted), and a reader MUST reject any record whose declared
  length is `0` or `> 4096`, or whose declared length exceeds the remaining input,
  with a typed decline and without allocating on the unchecked length. *Test:*
  `test_announce_structure_key_length_bounds`.
- **KISS-ANNOUNCE-6.3-0010** — A producer MUST write a `record_count`
  `<= 1048576` (`MAX_AVAILABILITY_RECORDS = 2^20`), and a reader MUST reject a
  `record_count` exceeding that maximum, or exceeding what the remaining input can
  contain, with a typed decline and without pre-allocating on the unchecked count.
  *Test:* `test_announce_record_count_bounds`.
- **KISS-ANNOUNCE-6.3-0006** — A consumer MUST treat an availability record as a
  cache **hit** only when both `structure_key` and `revision_hash` match its cached
  entry byte-for-byte, and MUST treat any other case as a **miss**. *Test:*
  `test_announce_hit_miss_by_full_identity`.

### 6.4 Contract-query protocol

- **KISS-ANNOUNCE-6.4-0001** — A contract-query **request** MUST be framed as: the
  4-byte request tag `CYRQ` (wire bytes `43 59 52 51`, u32 LE `0x51525943`); a
  little-endian u32 `structure_key` byte-length in `[1, 4096]`; the `structure_key`
  bytes; then a 1-byte `revision_present` flag (`0` or `1`); and, only when
  `revision_present == 1`, a 32-byte `revision_hash`. A reader MUST reject a
  `revision_present` value other than `0` or `1`, or a `structure_key` length outside
  `[1, 4096]`, with a typed decline. *Test:* `test_announce_query_request_shape`.
- **KISS-ANNOUNCE-6.4-0002** — A provider MUST answer a contract-query request with
  either (a) a contract **response** carrying the KISS-Contract document, or (b) a
  typed decline response. *Test:* `test_announce_query_response_is_contract_or_decline`.
- **KISS-ANNOUNCE-6.4-0006** — A provider handling a contract-query request MUST NOT
  panic, abort, crash, hang, or read outside the request buffer. *Test:*
  `test_announce_query_never_panics`.
- **KISS-ANNOUNCE-6.4-0003** — When `revision_present == 1`, a provider that returns a
  contract MUST return the contract whose `(structure_key, revision_hash)` identity
  matches the request exactly; if it holds no such revision and cannot provision one,
  it MUST return a typed decline (`UNKNOWN_REVISION` or `CANNOT_PROVISION`) rather
  than a mismatched contract. *Test:* `test_announce_query_revision_match_or_decline`.
- **KISS-ANNOUNCE-6.4-0012** — When `revision_present == 0`, a provider that returns a
  contract MUST return the contract for the highest-ordered `revision_hash` it holds
  for that `structure_key` (ordering = byte-for-byte lexicographic descending over the
  32-byte value), or, if it holds none and cannot provision one, a typed decline
  (`UNKNOWN_STRUCTURE_KEY` or `CANNOT_PROVISION`). *Test:*
  `test_announce_query_default_revision_is_highest`.
- **KISS-ANNOUNCE-6.4-0011** — A contract or decline **response** MUST echo the
  `(structure_key, revision_hash)` identity it is answering for: the response carries
  a little-endian u32 `structure_key` length, the `structure_key` bytes, a 1-byte
  `revision_present` flag, and — when `revision_present == 1` — the 32-byte
  `revision_hash` the provider selected. *Test:* `test_announce_response_echoes_identity`.
- **KISS-ANNOUNCE-6.4-0004** — A **contract response** MUST be framed as: the 4-byte
  response tag `CRSP` (wire bytes `43 52 53 50`, u32 LE `0x50535243`); the echoed
  identity block (§6.4-0011); a little-endian u32 payload byte-length; then that many
  bytes of the self-delimiting KISS-Contract document. *Test:*
  `test_announce_contract_response_framing`.
- **KISS-ANNOUNCE-6.4-0007** — A **decline response** MUST be framed as: the 4-byte
  decline tag `CDEC` (wire bytes `43 44 45 43`, u32 LE `0x43454443`); the echoed
  identity block (§6.4-0011); then a little-endian u32 `decline_code` drawn from
  §6.4-0009. *Test:* `test_announce_decline_response_framing`.
- **KISS-ANNOUNCE-6.4-0009** — A `decline_code` MUST take one of the following pinned
  little-endian u32 values, or a value in the experimental/vendor ranges below; a
  producer MUST NOT emit a core code with a meaning other than the one pinned here:

  | Value | Name | Meaning |
  |---|---|---|
  | `0x00000000` | reserved | MUST NOT be emitted |
  | `0x00000001` | `UNKNOWN_STRUCTURE_KEY` | no such cell, and none can be provisioned |
  | `0x00000002` | `CANNOT_PROVISION` | cell known but build-on-miss unavailable/failed |
  | `0x00000003` | `MALFORMED_REQUEST` | request framing invalid |
  | `0x00000004` | `QUERY_NOT_SUPPORTED` | provider does not advertise contract-query (§7.2 bit 33) |
  | `0x00000005` | `VERSION_UNSUPPORTED` | requested/negotiated version unsupported |
  | `0x00000006` | `UNKNOWN_REVISION` | `structure_key` known but requested `revision_hash` not held |

  Values in `[0x40000000, 0x80000000)` are the experimental range; values in
  `[0x80000000, 0x100000000)` are the vendor range (namespaced, registry-registered).
  *Test:* `test_announce_decline_code_enum`.
- **KISS-ANNOUNCE-6.4-0010** — A consumer that receives a `decline_code` it does not
  recognize MUST treat it as a generic decline and MUST NOT panic, abort, crash, or
  hang. *Test:* `test_announce_unknown_decline_code_is_generic`.
- **KISS-ANNOUNCE-6.4-0005** — A provider that advertises the contract-query
  capability bit (§7.2, FEAT bit 33) MUST implement a conforming contract-query
  endpoint answering per §6.4-0002. *Test:* `test_announce_query_bit_implies_endpoint`.
- **KISS-ANNOUNCE-6.4-0008** — A provider that does not advertise the contract-query
  capability bit MUST answer any contract-query request with a `QUERY_NOT_SUPPORTED`
  typed decline. *Test:* `test_announce_no_query_bit_declines`.
- **KISS-ANNOUNCE-6.4-0013** — An implementation MAY carry an OPTIONAL 8-byte
  request-correlation id on a contract-query request frame (`CYRQ`, §6.4-0001) and its
  response frame, so a **concurrent** transport can match a response to the right
  in-flight request. When carried, it is appended after the frame body as a 1-byte
  `correlation_present` flag (`0` or `1`) and — only when `correlation_present == 1` —
  an 8-byte `correlation_id`. When a request carries `correlation_present == 1`, the
  response answering it (a `CRSP` §6.4-0004, a `CDEC` §6.4-0007, or — on the KISS-Synth
  provision path that reuses this request verbatim, KISS-SYNTH-6.1-0002 — a `PRSP`
  provision-success frame, KISS-SYNTH-6.4-0001b) MUST echo the identical 8
  `correlation_id` bytes **unchanged**. The field is OPTIONAL and reserved: a
  synchronous seam MAY omit it entirely (the frames are then exactly §6.4-0001 /
  §6.4-0004 / §6.4-0007), and it MUST NOT be required for v1 conformance — a conforming
  reader MUST accept a frame carrying no correlation field, and a provider MUST NOT
  decline a request solely because it omits one. A reader MUST reject a
  `correlation_present` value other than `0` or `1` with a `MALFORMED_REQUEST` typed
  decline (§6.4-0009), never a panic. *Test:*
  `test_announce_correlation_id_echoed_when_present`.

### 6.5 Zero-dependency budget

- **KISS-ANNOUNCE-6.5-0001** — Producing or parsing the handshake envelope, an
  availability list, or a contract-query request/response frame MUST NOT require
  loading a compute driver, kernel runtime, GPU library, or any backend dynamic
  library. *Test:* `test_announce_zero_dependency_no_driver_load`.
- **KISS-ANNOUNCE-6.5-0002** — An implementation MUST be able to produce and parse
  every wire artifact of §6 using only its language's standard library, with no
  third-party runtime dependency. This obligation binds every implementation
  uniformly; the reference implementation holds no exemption. *Test:*
  `test_announce_impl_std_only`.

---

## 7. Capability, Profile & Extension model

### 7.1 Version-negotiation algorithm

Negotiation operates over the live (`>= 1`) `profiles` entries of the two exchanged
envelopes. Both roles emit the same 56-byte envelope; see §7.3 for role semantics and
sequencing.

- **KISS-ANNOUNCE-7.1-0001** — Given the local live-profile set `L` and the received
  live-profile set `R` (each the nonzero `profiles[0..profiles_len]` of the respective
  envelope), the negotiated profile MUST be `max(L ∩ R)` — the highest integer present
  in both sets. *Test:* `test_announce_negotiate_selects_highest_mutual`.
- **KISS-ANNOUNCE-7.1-0002** — If `L ∩ R` is empty, negotiation MUST return a typed
  decline and MUST NOT panic, abort, crash, or select any profile. *Test:*
  `test_announce_negotiate_empty_intersection_declines`.
- **KISS-ANNOUNCE-7.1-0003** — A producer MUST NOT emit an envelope whose
  `profiles_len` exceeds the 16-profile cap pinned by §6.1-0007. *Test:*
  `test_announce_producer_never_exceeds_profile_cap`.
- **KISS-ANNOUNCE-7.1-0004** — An implementation MUST NOT advertise a profile whose
  integer is below its declared **retirement floor**; profiles at or above the floor
  and at or below the current maximum define the live negotiation window
  (retire-by-floor deprecation). *Test:* `test_announce_retire_by_floor_window`.

### 7.2 Capability bitset structure

The 64-bit `capabilities` field is partitioned into three contiguous axes. All bit
positions are indexed from the least-significant bit (bit 0). The `capabilities` field
is a pure feature-advertisement field: it contains **no hard-gate bits**, and unknown
bits are ignored, never rejected (§7.2-0007).

| Axis | Bit range | Meaning | Registry authority |
|---|---|---|---|
| **EXT** | bits [0, 24) | External interchange tokens (DLPack/FDX and other foreign vocabularies) | external registry, mirrored in the KISS registry |
| **FEAT** | bits [24, 48) | KISS-defined provider-level optional handshake features | KISS registry (PR-gated) |
| **SUB** | bits [48, 64) | Which KISS sub-standards the provider speaks (presence only) | KISS registry (PR-gated) |

**Governance tiers (pinned bit sub-ranges within each axis):**

| Axis | Core | Experimental | Vendor |
|---|---|---|---|
| EXT [0,24) | [0, 16) | [16, 20) | [20, 24) |
| FEAT [24,48) | [24, 40) | [40, 44) | [44, 48) |
| SUB [48,64) | [48, 60) | [60, 62) | [62, 64) |

Assigned bits (first draft):

- **EXT** (core sub-range) — bit 0 `DLPACK_EXT_V1`, bit 1 `DLPACK_EXT_MX`, bit 2
  `DLPACK_EXT_GGML`, bit 3 `DLPACK_EXT_AFFINE`, bit 4 `DLPACK_EXT_SYMBOLIC`, bit 5
  `DLPACK_EXT_GATHER`; bits 6–15 core-reserved. EXT bit *meanings* are defined by the
  external interchange-token registry (§4) and mirrored in the KISS registry; the KISS
  clauses pin only the axis/range and the mirror, not the external vocabulary's
  semantics.
- **FEAT** (core sub-range [24,40)) — bit 32 `PROVISION_ON_REQUEST` (provider builds a
  kernel on miss); bit 33 `CONTRACT_QUERY` (provider answers §6.4 contract-queries);
  bits 24–31 and 34–39 core-reserved.
- **SUB** (core sub-range [48,60)) — bit 48 `SPEAKS_ANNOUNCE`, bit 49
  `SPEAKS_CLASSIFY`, bit 50 `SPEAKS_OPS`, bit 51 `SPEAKS_GRAMMAR`, bit 52
  `SPEAKS_SYNTH`, bit 53 `SPEAKS_CONSUME`, bit 54 `SPEAKS_EMIT`, bit 55
  `SPEAKS_CONTRACT`, bit 56 `SPEAKS_CONFORM`; bits 57–59 core-reserved. Each SUB bit is
  **presence-only** — it advertises *that* the provider speaks a sub-standard, not
  which version; per-version negotiation is carried by the profile mechanism (§7.1),
  which is the single defined "which version do we both speak" channel.

- **KISS-ANNOUNCE-7.2-0001** — An implementation MUST interpret `capabilities` under
  the three-axis partition EXT `[0,24)` / FEAT `[24,48)` / SUB `[48,64)`. *Test:*
  `test_announce_capabilities_axis_partition`.
- **KISS-ANNOUNCE-7.2-0002** — An implementation MUST interpret EXT bits per their
  registered external-token assignments and MUST NOT repurpose an assigned EXT bit.
  *Test:* `test_announce_ext_bit_assignments`.
- **KISS-ANNOUNCE-7.2-0003** — An implementation MUST interpret bit 32 as
  `PROVISION_ON_REQUEST` and bit 33 as `CONTRACT_QUERY`. *Test:*
  `test_announce_feat_bit_assignments`.
- **KISS-ANNOUNCE-7.2-0004** — An implementation MUST interpret SUB bits 48–56 as the
  per-sub-standard presence-only "speaks" flags listed above. *Test:*
  `test_announce_sub_bit_assignments`.
- **KISS-ANNOUNCE-7.2-0010** — An implementation MUST NOT read any per-sub-standard
  version from the SUB axis (SUB bits are presence-only); version negotiation MUST use
  the profile mechanism of §7.1. *Test:* `test_announce_sub_axis_is_presence_only`.
- **KISS-ANNOUNCE-7.2-0005** — An implementation MUST confine each core/experimental/
  vendor meaning to the pinned bit sub-range for that tier and axis in the tier table
  above, and MUST NOT assign meaning to a bit outside its tier's pinned sub-range.
  *Test:* `test_announce_reserved_range_tiers`.
- **KISS-ANNOUNCE-7.2-0006** — A producer MUST write zero to every currently
  unassigned (reserved) capability bit. *Test:* `test_announce_unassigned_bits_zero`.
- **KISS-ANNOUNCE-7.2-0007** — A reader MUST ignore (treat as absent, NOT reject) any
  set capability bit whose meaning it does not recognize; the `capabilities` bitset
  contains no hard-gate bits, so an unrecognized bit MUST NOT cause rejection. This
  forward-compatibility rule applies to the `capabilities` bitset only and MUST NOT be
  applied to the envelope's reserved regions (§6.2), which are hard-rejected. *Test:*
  `test_announce_reader_ignores_unknown_capability_bits`.
- **KISS-ANNOUNCE-7.2-0008** — A KISS-owned (EXT-mirror / FEAT / SUB) capability-bit
  assignment MUST originate from a merged change to the PR-gated KISS capability
  registry under ThinkersJournal; an implementation MUST NOT rely on an unregistered
  bit assignment. *Test:* `test_announce_capability_registry_pr_gated`.

> **Reconciliation note (informative).** The umbrella §6.2 forward-compat rule
> contemplates "required" capability bits that a receiver hard-fails on if unknown.
> KISS-Announce's `capabilities` u64 is a pure feature-advertisement field with **no**
> such hard-gate bits (§7.2-0007); the umbrella's hard-fail-on-unknown-required rule
> is requested to be scoped to a negotiation field other than this bitset.

### 7.3 Handshake symmetry & role semantics

- **KISS-ANNOUNCE-7.3-0001** — Both the provider and the consumer MUST emit the same
  56-byte handshake envelope defined in §6.1; negotiation (§7.1) is computed
  identically from the two envelopes by either side and is independent of transmission
  order. *Test:* `test_announce_both_roles_emit_envelope`.
- **KISS-ANNOUNCE-7.3-0002** — A role that cannot provide a provider-only FEAT feature
  (`PROVISION_ON_REQUEST`, `CONTRACT_QUERY`) MUST write those bits as zero; a consumer
  MUST NOT set a provider-only FEAT bit it does not itself offer. *Test:*
  `test_announce_consumer_zeroes_provider_only_feat`.

---

## 8. Versioning & Lifecycle

KISS-Announce tracks the Charter's **two version axes**: the wire/ABI *envelope
schema version* (`envelope_version`, currently 1) and the published reference-crate
*semver*. They move independently.

- **KISS-ANNOUNCE-8-0001** — The envelope schema version and the reference-crate
  semver MUST be tracked as independent axes; a crate semver change MUST NOT be taken
  to imply an envelope wire change. *Test:* `test_announce_two_version_axes_independent`.
- **KISS-ANNOUNCE-8-0002** — Any change to the envelope *shape* (field offset, size,
  count, alignment, or total length — e.g. raising the profile cap) MUST bump
  `envelope_version`. *Test:* `test_announce_shape_change_bumps_version`.
- **KISS-ANNOUNCE-8-0003** — Assigning a previously-reserved capability bit, EXT
  token, or SUB flag MUST NOT bump `envelope_version` (additive, forward-compatible
  under §7.2-0007). *Test:* `test_announce_additive_capability_no_version_bump`.
- **KISS-ANNOUNCE-8-0004** — KISS-Announce MUST NOT be promoted from Draft to Frozen
  until ≥2 structurally dissimilar implementations have interoperated on the golden
  hex vectors of Appendix A. *Test:* `test_announce_freeze_gate_two_impls` (checklist gate;
  signed by the AUDIT role, not DESIGN).
- **KISS-ANNOUNCE-8-0005** — KISS-Announce MUST NOT be promoted from Draft to Frozen
  until a foreign reader written outside the reference language has consumed the wire,
  with endianness, pointer-width, and the 6-byte `reserved1` structure padding
  explicitly checked (umbrella §5.3 freeze gate). *Test:*
  `test_announce_freeze_gate_foreign_reader` (checklist gate; AUDIT-signed).
- **KISS-ANNOUNCE-8-0006** — KISS-Announce MUST NOT be promoted from Draft to Frozen
  until this sub-standard's KISS-Conform suite exists and passes. *Test:*
  `test_announce_freeze_gate_conform_suite_passes` (checklist gate; AUDIT-signed).

**Convergence task (informative):** the two byte-identical seeds converge to one
canonical registry-published crate via a no-wire-change re-export shim; convergence is
verified by golden hex equality, not language struct-type equality.

---

## 9. Conformance

An implementation conforms to KISS-Announce at a given envelope version if it (a)
produces and parses exactly the wire artifacts of §6–§8 for that version, (b) passes
the KISS-Conform suite for KISS-Announce at that version, and (c) satisfies the DAG
prerequisite closure. Because the Announce↔Classify and Announce↔Contract edges are
labeled **OPAQUE** (§4), claiming KISS-Announce requires only agreement on the meaning
of the `structure_key` token and the contract payload, not a co-claim of those
upstream sub-standards. Un-claimed or malformed inputs yield typed declines, never
panics, per §6.2-0007 and §6.4-0006 (the owning clauses; verified by the
negative-vector modality). The modified-suite prohibition of the mark policy is the
umbrella's rule (umbrella §9.3), enforced via registry listing, and is not restated as
a free-standing Announce clause.

### 9.1 Clause → KISS-Conform test traceability matrix

| Clause ID | Named conformance test |
|---|---|
| KISS-ANNOUNCE-6.0-0001 | `test_announce_determinism_class_exact_byte` |
| KISS-ANNOUNCE-6.1-0001 | `test_announce_envelope_size_is_56` |
| KISS-ANNOUNCE-6.1-0002 | `test_announce_envelope_wire_is_56_contiguous` |
| KISS-ANNOUNCE-6.1-0003 | `test_announce_field_offsets_match_table` |
| KISS-ANNOUNCE-6.1-0013 | `test_announce_field_sizes_match_table` |
| KISS-ANNOUNCE-6.1-0004 | `test_announce_magic_constant` |
| KISS-ANNOUNCE-6.1-0014 | `test_announce_fixed_prefix_stable_across_versions` |
| KISS-ANNOUNCE-6.1-0005 | `test_announce_version_field_is_1` |
| KISS-ANNOUNCE-6.1-0006 | `test_announce_reserved0_is_zero` |
| KISS-ANNOUNCE-6.1-0007 | `test_announce_profiles_len_within_cap` |
| KISS-ANNOUNCE-6.1-0008 | `test_announce_trailing_profiles_zero` |
| KISS-ANNOUNCE-6.1-0015 | `test_announce_live_profiles_nonzero` |
| KISS-ANNOUNCE-6.1-0009 | `test_announce_profiles_strictly_ascending` |
| KISS-ANNOUNCE-6.1-0010 | `test_announce_reserved1_pad_zero` |
| KISS-ANNOUNCE-6.1-0011 | `test_announce_capabilities_field` |
| KISS-ANNOUNCE-6.1-0012 | `test_announce_all_fields_little_endian` |
| KISS-ANNOUNCE-6.2-0010 | `test_announce_reads_prefix_before_length` |
| KISS-ANNOUNCE-6.2-0001 | `test_announce_reject_wrong_length_for_version` |
| KISS-ANNOUNCE-6.2-0002 | `test_announce_reject_bad_magic` |
| KISS-ANNOUNCE-6.2-0003 | `test_announce_reject_unknown_version` |
| KISS-ANNOUNCE-6.2-0004 | `test_announce_reject_nonzero_reserved0` |
| KISS-ANNOUNCE-6.2-0011 | `test_announce_reject_nonzero_reserved1` |
| KISS-ANNOUNCE-6.2-0005 | `test_announce_reject_profiles_len_overflow` |
| KISS-ANNOUNCE-6.2-0006 | `test_announce_reject_non_ascending_profiles` |
| KISS-ANNOUNCE-6.2-0012 | `test_announce_reject_nonzero_trailing_profiles` |
| KISS-ANNOUNCE-6.2-0013 | `test_announce_reject_zero_live_profile` |
| KISS-ANNOUNCE-6.2-0007 | `test_announce_rejection_is_typed_decline` |
| KISS-ANNOUNCE-6.2-0008 | `test_announce_reader_never_repairs` |
| KISS-ANNOUNCE-6.3-0001 | `test_announce_availability_is_identity_pair` |
| KISS-ANNOUNCE-6.3-0002 | `test_announce_availability_carries_no_capability` |
| KISS-ANNOUNCE-6.3-0003 | `test_announce_revision_hash_is_32_bytes` |
| KISS-ANNOUNCE-6.3-0007 | `test_announce_revision_hash_compared_bytewise` |
| KISS-ANNOUNCE-6.3-0008 | `test_announce_revision_hash_opaque_identity` |
| KISS-ANNOUNCE-6.3-0004 | `test_announce_structure_key_is_opaque` |
| KISS-ANNOUNCE-6.3-0011 | `test_announce_availability_list_tag` |
| KISS-ANNOUNCE-6.3-0012 | `test_announce_availability_list_version` |
| KISS-ANNOUNCE-6.3-0005 | `test_announce_availability_framing` |
| KISS-ANNOUNCE-6.3-0009 | `test_announce_structure_key_length_bounds` |
| KISS-ANNOUNCE-6.3-0010 | `test_announce_record_count_bounds` |
| KISS-ANNOUNCE-6.3-0006 | `test_announce_hit_miss_by_full_identity` |
| KISS-ANNOUNCE-6.4-0001 | `test_announce_query_request_shape` |
| KISS-ANNOUNCE-6.4-0002 | `test_announce_query_response_is_contract_or_decline` |
| KISS-ANNOUNCE-6.4-0006 | `test_announce_query_never_panics` |
| KISS-ANNOUNCE-6.4-0003 | `test_announce_query_revision_match_or_decline` |
| KISS-ANNOUNCE-6.4-0012 | `test_announce_query_default_revision_is_highest` |
| KISS-ANNOUNCE-6.4-0011 | `test_announce_response_echoes_identity` |
| KISS-ANNOUNCE-6.4-0004 | `test_announce_contract_response_framing` |
| KISS-ANNOUNCE-6.4-0007 | `test_announce_decline_response_framing` |
| KISS-ANNOUNCE-6.4-0009 | `test_announce_decline_code_enum` |
| KISS-ANNOUNCE-6.4-0010 | `test_announce_unknown_decline_code_is_generic` |
| KISS-ANNOUNCE-6.4-0005 | `test_announce_query_bit_implies_endpoint` |
| KISS-ANNOUNCE-6.4-0008 | `test_announce_no_query_bit_declines` |
| KISS-ANNOUNCE-6.4-0013 | `test_announce_correlation_id_echoed_when_present` |
| KISS-ANNOUNCE-6.5-0001 | `test_announce_zero_dependency_no_driver_load` |
| KISS-ANNOUNCE-6.5-0002 | `test_announce_impl_std_only` |
| KISS-ANNOUNCE-7.1-0001 | `test_announce_negotiate_selects_highest_mutual` |
| KISS-ANNOUNCE-7.1-0002 | `test_announce_negotiate_empty_intersection_declines` |
| KISS-ANNOUNCE-7.1-0003 | `test_announce_producer_never_exceeds_profile_cap` |
| KISS-ANNOUNCE-7.1-0004 | `test_announce_retire_by_floor_window` |
| KISS-ANNOUNCE-7.2-0001 | `test_announce_capabilities_axis_partition` |
| KISS-ANNOUNCE-7.2-0002 | `test_announce_ext_bit_assignments` |
| KISS-ANNOUNCE-7.2-0003 | `test_announce_feat_bit_assignments` |
| KISS-ANNOUNCE-7.2-0004 | `test_announce_sub_bit_assignments` |
| KISS-ANNOUNCE-7.2-0010 | `test_announce_sub_axis_is_presence_only` |
| KISS-ANNOUNCE-7.2-0005 | `test_announce_reserved_range_tiers` |
| KISS-ANNOUNCE-7.2-0006 | `test_announce_unassigned_bits_zero` |
| KISS-ANNOUNCE-7.2-0007 | `test_announce_reader_ignores_unknown_capability_bits` |
| KISS-ANNOUNCE-7.2-0008 | `test_announce_capability_registry_pr_gated` |
| KISS-ANNOUNCE-7.3-0001 | `test_announce_both_roles_emit_envelope` |
| KISS-ANNOUNCE-7.3-0002 | `test_announce_consumer_zeroes_provider_only_feat` |
| KISS-ANNOUNCE-8-0001 | `test_announce_two_version_axes_independent` |
| KISS-ANNOUNCE-8-0002 | `test_announce_shape_change_bumps_version` |
| KISS-ANNOUNCE-8-0003 | `test_announce_additive_capability_no_version_bump` |
| KISS-ANNOUNCE-8-0004 | `test_announce_freeze_gate_two_impls` |
| KISS-ANNOUNCE-8-0005 | `test_announce_freeze_gate_foreign_reader` |
| KISS-ANNOUNCE-8-0006 | `test_announce_freeze_gate_conform_suite_passes` |

Every normative clause above appears in this matrix exactly once; the KISS-Conform
build MUST fail if any clause ID lacks a passing mapped test (bidirectional
traceability). Clause IDs are mirrored in the machine-readable sidecar
(`kiss-announce.validusage.json` analog) kept in sync by the traceability lint.

---

## 10. Governance

- **Editor of record:** the KISS-Announce editor assignment is **proposed, pending
  ratification** in the Charter governance record (which does not yet name an editor
  for this sub-standard). The editor holds the pen, allocates clause IDs (append-only,
  never reused after retirement), and solicits comment from interested cosignatories —
  any project building a provider or consumer that speaks the handshake — before
  deciding a change.
- **Steward:** ThinkersJournal hosts the spec, the capability/profile registry (PR-
  gated), and the conformance registry; it free-certifies self-certified
  implementations on request as resources permit.
- **Ratifier / maturity transitions:** the AUDIT role (not DESIGN) signs each
  maturity transition; the Draft→Frozen transition requires the freeze gate of
  §8-0004 / §8-0005 / §8-0006 (umbrella §5.3).
- **License:** this specification is dedicated to the public domain under CC0 1.0
  Universal; reference crates are
  MIT-OR-Apache-2.0; the KISS-Conform suite is permissive-to-run. Per the umbrella
  mark policy (umbrella §9.3), a modified conformance suite does not back a
  conformance claim; that policy is enforced via steward-registry listing, not
  restated as a normative Announce clause.
- **Patent:** contributors grant a royalty-free license to essential claims on RFC
  contribution, with defensive termination, per the Charter.
- **Conformance posture:** self-certification with published results plus the
  steward-maintained registry is the authoritative record of verified
  implementations.

---

## Appendix A — Golden vectors & provenance (informative)

**A.1 Golden hex vector.** The 56-byte serialization in §2.5 is the first golden
vector for `test_announce_magic_constant`, `test_announce_field_offsets_match_table`,
`test_announce_field_sizes_match_table`, and the reserved-region tests. Its `capabilities`
value is `0x0000_0003_0000_003F` (EXT bits 0–5 | FEAT bit 32 | FEAT bit 33), wire
bytes `3F 00 00 00 03 00 00 00`. Additional negative vectors (nonzero `reserved1`,
50-byte packed layout, big-endian magic, `profiles_len = 17`, non-ascending profiles,
a zero live profile, an over-cap `structure_key` length, an over-cap `record_count`,
an out-of-range `revision_present` flag) drive the §6.2 / §6.3 / §6.4 reject tests and
are the adversarial-outsider battery for the foreign-reader freeze gate. Each message
tag (`SEAM`/`CYRQ`/`CRSP`/`CDEC`/`SAVL`) carries a golden vector with an explicit
"bytes on the wire, left to right" row per §2.4.

**A.2 Provenance / acknowledgments.** The handshake envelope derives from the
`SeamHello` seed independently reproduced byte-identically in two project workspaces
(both Evans Laboratories projects: the Baracuda `baracuda-seam` and the Fuel
`fuel-kernel-seam-announce` seeds), each using C-compatible layout, 56 bytes, magic
ASCII `SEAM`, envelope version 1. Project names in this appendix and in §0/§2.5 are
non-normative provenance and examples only; no normative clause names any project.

**A.3 Migration note.** The two seeds converge to one canonical crate via a
no-wire-change re-export shim; the wire is unchanged (verified by A.1 golden hex, not
by language struct-type equality), so no `envelope_version` bump is required
(§8-0003).