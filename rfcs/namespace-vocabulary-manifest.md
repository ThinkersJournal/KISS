# RFC: a machine-readable envelope for §6.8-0004 namespace capability vocabularies

**Status:** Accepted in principle (2026-08-14). **Refs:** #171. **Realized by:** the
KISS-CLASSIFY §6.8 clauses this PR adds and their mapped conformance tests.

**Cosignatories (both namespace owners, which was the acceptance gate):**
- **Baracuda** — owner of the `cuda` namespace. Cosigned; confirmed its `cuda.md` TSV
  appendix *"IS the source, not a seed regenerated from upstream … stable enough to consume
  directly,"* so the `enumerated` case stays as simple as the strawman.
- **Vulkane** — owner of the `vulkan` namespace. Cosign given by the maintainer.
- Proposer: Unpopped (strawman and the motivating `sm90` drift measurement).

> **Summary line (Baracuda's, adopted):** *per-namespace **shape** differs by kind; the
> **envelope** is shared.* Admitting two kinds under one envelope **is** respecting that the
> vocabularies differ in kind, rather than overriding it. KISS pins the envelope; the
> maintainer fills it.

## 1. Problem, measured rather than argued

The two registered namespaces publish their capability vocabularies in incompatible shapes:
`cuda` as a **closed enumeration** (`spec/namespaces/cuda.md`'s TSV appendix); `vulkan` as a
**generated** grammar over an open product space (`spec/namespaces/vulkan.md`, prose across
§2.1–2.4 with canonical ordering and typed declines). A consumer supporting both
hand-transcribes one and hand-parses prose for the other.

This is the §6.1 dtype problem on the target axis, and it ends the way that one did.
`unpopped-vocab` carried a 22-member dtype set against KISS's 24 **for weeks, green on both
sides**, because both sides were prose; `conformance/corpus/dtype_manifest.json` turned that
into a thirty-second script and every sk4 byte-match leg has since bound against it rather
than against §6.1's table. **Live drift, not hypothetical:** `ArchSku::Sm90` was wired to
close five `cuda:sm90` byte-match vectors, so `cuda.md`'s SSOT and its own named reference
implementation were out of sync, and the only thing that noticed was a human reading the
annex afterwards. A generated manifest with an emit-and-`git diff --exit-code` gate would
have red-built instead.

## 2. Venue: KISS owns the meta-level; the precedent is already in §6.8

§6.8-0004 delegates vocabulary **content** to the maintainer, and it should. But KISS already
owns the entire meta-level of this axis — §6.8-0001 (grammar), -0002 (byte-exact matching),
-0003 (the machine-readable namespace *registry*), -0005 (charset), -0006 (fixed-width
juxtaposition), -0007 (the FNV-1a digest form). -0006 and -0007 already constrain how a
maintainer may express their own vocabulary, for the benefit of everyone reading it. **An
annex *format* is that same shape.** Content stays delegated; the manifest is the
machine-readable form of an annex the registry already points at (§6.8-0003).

## 3. The envelope

A namespace MAY publish its capability-set vocabulary as a manifest with schema id
`kiss-namespace-vocabulary-v1`:

```json
{
  "schema": "kiss-namespace-vocabulary-v1",
  "namespace": "cuda",
  "vocabulary_version": 1,
  "generated_from": "spec/namespaces/cuda.md",
  "kind": "enumerated",
  "grammar": "cuda:sm<N>[<letter>]",
  "coverage_note": "…what this file does not pin…",
  "members": [ { "token": "cuda:sm80", "notes": "…" } ]
}
```

with `"kind": "generated"` carrying a `field_spec` (documentation) and a `vectors` array (the
contract) in place of `members`. Required envelope fields, all namespaces, both kinds:
`schema`, `namespace` (a namespace `registered` in §6.8-0003's registry), `vocabulary_version`,
`generated_from`, `kind`, `coverage_note`.

The two-question symmetry with the dtype axis is why the discrimination is necessary: *can I
recognize a well-formed token?* and *can I enumerate what exists?* are the same split as the
artifact's **recognition set (24)** vs **usable set (22)**. Both questions have answers for
both kinds; only the second is sometimes "enumeration is impossible, and that is the contract."

## 4. Five envelope requirements (each with in-tree precedent)

### 4.1 `vocabulary_version` is a GATE, not a field

§6.8-0004 freezes each vocabulary independently (§8). A consumer reading a manifest MUST
**assert** its `vocabulary_version` equals the version it was built against and typed-decline
on mismatch — exactly as `dtype_manifest.json`'s `structure_key_schema_version` is asserted,
never merely read. *A field a consumer reads is a field; one the consumer asserts is a gate,
and the first consumer to handle a mismatch **gracefully** has defeated it.*

### 4.2 `kind` is an OPEN set with a typed decline

`kind` is a discriminated union, and a union whose cases were derived from exactly two
examples is a closed list built from n=2. This suite has made that mistake twice in one week —
a mechanism list went four→five the moment someone looked (#164), and the capability-exclusion
axes went one→three across two byte-match legs, forcing the rule to be restated as open. So a
consumer encountering an unrecognized `kind` MUST typed-decline, MUST NOT guess a shape (the
abstention rule, #165). The two kinds named at this schema version are `enumerated` (a closed
`members` list) and `generated` (an open product space, validated by vectors); a third is
admitted **additively**, and §5 sketches what one would look like so the format is not frozen
at two.

### 4.3 Generated, with an emit-and-`git diff --exit-code` freshness gate

A manifest MUST name its source annex in `generated_from` and MUST be regenerable from it
under an emit-and-compare gate, so the machine-readable form cannot drift from the annex it
derives from. `structure_key_vectors.json` (#161) is the working precedent: the file, its
generator, and a byte-equality freshness test all ship together.

### 4.4 A coverage note stating what the file does NOT pin

For a `generated` namespace, that enumeration is impossible and the vectors are the whole
contract; for either kind, which instrument closes what this file leaves open. Precedent:
`coverage_note` (#169), which states both what the artifact misses **and** which instrument
closes it — a note stating only the gap invites the reader to conclude the gap is unclosed.

### 4.5 The declarative / production split is normative, not editorial

A manifest has two halves with two audiences:

| a consumer that… | needs |
|---|---|
| only **parses** foreign tokens | the DECLARATIVE half — grammar, alphabets, orderings, `members` |
| also **produces** tokens | the declarative half **and** the production half — canonicalization algorithms and `vectors` |

Only the emit side resists declaration (the `vulkan:` `<coop>` length-conditional switch, §5).
Recognition stays declarative for both kinds. A conformance profile for a parse-only consumer
MUST NOT require the production half; the clause must not tell parsers they need machinery they
do not.

## 5. The generated case: vectors are the contract

Canonicalization cannot be validated from a grammar, so for `kind: generated` the `field_spec`
is **documentation** and the normative contract is a `vectors` array whose entries a conformant
producer's output MUST reproduce byte-exact. The hardest field is why: `vulkan:`'s `<coop>`
selects its third form by a **length-conditional switch** — the canonical enumeration string
(comma-joined tuples, `cm-` excluded) is emitted inline at `<= 512` bytes and replaced by
`cm-fnv1a64-<hex16>` (the §6.8-0007 digest of *that same string*) above it. No alphabet, regex,
or field spec decides that.

**A sufficient generated vector set IS specifiable at the envelope level** — as coverage of the
canonicalization concerns, with the maintainer filling the values. The vector set MUST include,
each tagged with what it `pins`, at least one vector for each of:

1. **`order`** — a non-canonically-ordered input and its canonical output (V-8's sort).
2. **`dedup`** — a duplicate-bearing input and its deduped output (V-8's dedup).
3. **`threshold`** — each length-conditional switch, presented *at* and *immediately across* its
   boundary, so both forms are pinned at the exact byte count that flips them (V-9's 512).
4. **`digest_input`** — the precise byte string fed to the §6.8-0007 digest: the *same* string
   measured against the threshold, so a producer can disagree about *whether* to digest but
   never about *what* is digested.

KISS checks the vector set **covers** these four `pins`; the actual input→output values are the
maintainer's. A namespace with no length-conditional field omits `threshold`/`digest_input` and
says so in its `coverage_note`.

**A third kind, sketched (per §4.2).** A namespace whose vocabulary is neither a closed list nor
a pure product-with-canonicalization — e.g. one gated by a *runtime probe* whose result is not
enumerable and not a function of a canonical string — would be a `kind` this envelope does not
name. It would carry neither `members` nor a `digest_input` vector, and its contract would be a
*recognition-only* declarative half plus an explicit `coverage_note` that production is not
specifiable. The point of naming it now is that its existence is a reason `kind` is open, not a
reason to widen the closed set to three.

## 6. Out of scope, deliberately

**No vocabulary content is proposed here.** `cuda`'s enumeration is Baracuda's; `vulkan`'s field
spec and canonicalization are Vulkane's; neither is authored in this PR. A convention imposed on
a maintainer's annex by a consumer is not a convention — which is why #171 sought the two owners'
cosignatures rather than editing their files, and both gave them.

## 7. Clauses realized

Provisionally in the KISS-CLASSIFY §6.8 family (final IDs assigned by the editor at integration,
per #171 / #138 — the numbers below are the natural next in sequence and are used so the
`*Test:*` mapping and §9 matrix are complete):

| clause | requirement | test |
|---|---|---|
| §6.8-0008 | the manifest envelope: schema id + required fields; content stays the maintainer's | `test_namespace_vocabulary_envelope_shape` |
| §6.8-0009 | `vocabulary_version` MUST be asserted (a gate), not merely read | `test_namespace_vocabulary_version_is_asserted` |
| §6.8-0010 | `kind` is an OPEN set; an unrecognized `kind` typed-declines, never guesses | `test_namespace_vocabulary_kind_open_set` |
| §6.8-0011 | generated from the annex, under an emit-and-`git diff --exit-code` freshness gate | `test_namespace_vocabulary_freshness_provenance` |
| §6.8-0012 | the declarative (parse) half suffices for a parse-only consumer; production is separate | `test_namespace_vocabulary_declarative_production_split` |
| §6.8-0013 | for `generated`, the vector set is the contract and MUST cover order/dedup/threshold/digest_input | `test_namespace_vocabulary_generated_vectors_cover_canonicalization` |
