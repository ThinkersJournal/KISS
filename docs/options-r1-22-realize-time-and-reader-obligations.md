# Options note: R1.22, does KISS regulate the tensor seam at realize time and bind its readers? (#424)

**Informative. No normative text. For CireSnave's decision; the PM turns the question into a board item.**
Read at `origin/main` `2e65634`, 2026-10-26. Paths are `file:line` at that commit. Issue: KISS#424 (Fuel, part of the #236
parity work). Roadmap row: R1.22. The numbers about the other format (FDX) are the Fuel architect's measurements in the
issue at `fe4fb8cd` and `4dd37b9a`; **I did not re-measure them.** Nothing here freezes.

---

## The question

**Example.** A descriptor says a tensor's extent is a symbol, `n`. At build time nobody knows `n`. At run time `n` is
bound to 5000 and the buffer holds 4096 elements. Checking that is only possible once `n` is known. A reader that
validates only at build time never runs that check, and returns success on a tensor it cannot read. Separately, a
descriptor carries a flag meaning "the base descriptor alone does not carry this tensor's meaning". A producer must set
it (a golden can check that). A reader that ignores it decodes the tensor wrongly and also returns success. Nothing
today binds the reader.

**What KISS has.** One validation phase, on the producer's output, compared against golden vectors. Re-searched at
`origin/main` with controls: `realize-time`, `validate_realize`, `symbol environment` and `MEANING_REQUIRES_EXT` appear in
**no** file of `spec/`, while `fdx` appears in three lines (all in `spec/announce.md` and `spec/classify.md`, the capability
bit and registry lines such as `spec/announce.md:244` and `:626`, which say only that a side speaks the external
family). So the other format's obligations are not in KISS at all; KISS names the token and stops. The harness has
five modalities (`spec/conform.md:154`), none of which resolves a symbol environment, so a realize-time check has no
instrument today.

**The size of it, per the issue (unverified here).** The other format validates at three call surfaces; 27 is its error
vocabulary and **104 or more** its distinct failure conditions; three checks are run by neither top-level validator, so
"call the validator" does not conform; and of nine flags, four are decode-affecting (a reader that ignores one returns
success and misreads), three are self-protecting, and two have no producer-side check.

## Options

- **A. Take the tensor seam into KISS as its own sub-standard,** with build-time, realize-time and reader obligations.
  This is what the Phase 1 parity gate asks if the seam is in scope ("KISS includes everything needed"). *Cost:* the
  largest item on the board: a new standard with a 100-plus obligation floor, and a new harness modality (a harness that
  resolves symbols and revalidates) before any realize-time clause can be backed.
- **B. Keep the status quo and record it:** KISS regulates kernel contracts, and the tensor seam is owned by the
  external interchange registry whose token KISS advertises (`spec/announce.md:244`). The parity ledger marks these rows
  "out of scope by ruling" instead of "missing". *Cost:* retires the claim that KISS covers the tensor seam, so the
  Phase 1 gate's wording must say which seams it means. The issue itself says this option must stay on the table.
- **C. A narrow middle:** say, in Announce next to the capability bits, that a side which sets an external token
  bit MUST honour that registry's decode-affecting flags, and mark the clause **untestable until a second reader
  exists** (the ledger already has `untestable` and `attested` rows, `conformance/UNBACKED.tsv`). *Cost:* it makes KISS bind
  semantics that `spec/announce.md` says it does not own (the EXT bullet under the 7.2 table says the KISS
  clauses pin only the axis and the mirror, not the external vocabulary's meaning), so it needs that sentence qualified.

**Recommendation: B now, with C as the first step if you want any reader obligation.** A is the right answer only if you
want KISS to be the home of the tensor seam, and that is a decision about the project, not about a clause. B costs
nothing and makes the parity ledger honest today. C is cheap and is the one piece of the issue that is a **wrong answer
with a success return**, which is why it is worth a sentence even if A is declined.

## What waits, and what to decide

A waits on a modality that does not exist. Decide: **A, B or C**, and if A or C, who owns the external flags' meaning
(the external registry, or KISS).
