# Design: a bind-time obligation class, reader obligations, and the instrument that tests them (R1.22, KISS#424)

Status: **design with draft clause text; nothing here is a clause yet.** Written 2026-11-04 against `origin/main` 5988cf1 (conformance 0.7.10). Ruling: CireSnave, board 379, 2026-11-04 via the PM, verbatim "1": add a realize-time obligation class **and** build an instrument that binds symbols and re-validates, with reader obligations stated explicitly. This note turns that ruling into clauses, a harness and a vector family. Clause IDs below are proposals; the PM and the sk7 batch decide the order they land in.

## 1. What is already there (measured at 5988cf1)

KISS already evaluates things against a bound environment and already declines typed. It has no *name* for it, no clause class, and no instrument that is aimed at it.

- `KISS-CONTRACT-6.6-0006`: a Dispatch expression evaluated "against concrete launch-scalar values MUST yield a non-negative integer"; an out-of-bounds subscript, an absent symbol and a non-grammar field are typed declines. `-0009`: zero divisor, non-positive `ceil_div` divisor and `i64::MIN / -1` are typed declines, never a wrapped value. These are bind-time checks in all but name.
- `KISS-CONFORM-6.7-0001/-0002`: negative vectors pin exactly one decline code. This is the right shape for a bind-time verdict; it just has no field for the bound environment.
- `UNBACKED.tsv` already lets a clause be present and honestly untested.
- Absent: no clause says *when* a check runs (build or bind), no clause says *who* it binds (producer or reader), and the `§6.6-0006/-0009` tests build their environment by hand inside one test each.

The issue's claim "a static vector corpus cannot supply a resolved symbol environment" is true of a corpus that stores only a produced artifact and its golden. It is not true of a vector that stores an **input environment** beside the artifact: `(contract bytes, environment, pinned decline-or-ok)` is still a vector, still byte-exact, and still compared. What KISS lacks is the **function under test** (`bind` then re-validate) and the vector family that feeds it.

## 2. FDX's realize-time checks, mapped (Fuel `fuel-ir/src/dlpack/validate.rs`; corrected 2026-11-04 after the Fuel lane's read at origin/main 2247bfb23)

The issue's four are **three checks producing three error variants**: V17 affine evaluation (`UnboundSymbol`, `AffineOverflow`, `ExtentOutOfRange`), V14 extent bound, and V21d live sequence length (which returns `ExtentOutOfRange`, not a fourth variant).

| FDX check | Where | KISS-level obligation it implies | Already in KISS? |
|---|---|---|---|
| `UnboundSymbol` | `validate.rs:1166,1217` | A bound environment MUST assign every symbol the checked expression names; a missing one is a typed decline. | Partly: 6.6-0006 declines "a symbol absent from the vocabulary", not "present in the vocabulary but unassigned". |
| `ExtentOutOfRange` | `:1181,1191,1235` | A bound extent MUST be non-negative and within the declared range or capacity; a negative value is rejected before any narrowing. | Partly: 6.6-0006 non-negative result; no declared-range rule. |
| `AffineOverflow` | `:1174,1177` | Evaluating a declared expression MUST NOT wrap; overflow is a typed decline. | Partly: 6.6-0009(d) names only `i64::MIN / -1`; no general overflow rule for `+ - *`. |
| gather live sequence length | V21d `check_v21d_seq_live_length` | A bound length L is in `0 <= L <= max_seq_capacity` and `ceil(L / block_size) <= max_blocks_per_seq`; `L == 0` is legal; the verdict is `ExtentOutOfRange`. A bound, not an agreement. | No: gather is not in the contract's launch-scalar model. Mapping row only. |

FDX's own count is 4 of 27 variants (a floor of 104+ conditions, KISS#424 comment of 2026-09-05); the three checks no orchestrator runs (V12, V15, V21c) are a *different* gap, not covered here.

## 3. Proposal

### 3.1 The class (Contract, new section 6.12 "Bind-time obligations", normative)

- **-0001 (definition).** A **bound environment** assigns an integer (signed 64-bit for KISS Dispatch expressions; Fuel's `FDXSymBinding.value` is unsigned 64-bit, so a negative bound symbol is inexpressible there and a negative extent arises only from an affine result) to each launch-scalar symbol of the contract's vocabulary (§6.5-0004a) and a length to each rank-length array symbol. **Bind time** is the moment a consumer launches against a bound environment; **build time** is the moment the contract is produced or read. A **bind-time obligation** is one whose verdict cannot be computed before an environment exists.
- **-0002 (marking).** Every clause whose verdict depends on a bound environment MUST say "at bind time" in its first sentence, and MUST name its typed decline code. A clause that does not say so is a build-time clause; a reader MUST NOT defer it to bind time.
- **-0003 (unbound).** Binding MUST decline `unbound_symbol` when an expression names a vocabulary symbol the environment does not assign; an implementation MUST NOT substitute zero or a default.
- **-0004 (range).** Binding MUST decline `extent_out_of_range` for a negative extent, and for a value outside a range a field declares, before any narrowing to a smaller integer type.
- **-0005 (overflow).** Evaluating a Dispatch expression MUST decline `affine_overflow` when any `+`, `-` or `*` leaves the signed 64-bit range; it MUST NOT wrap or saturate. This generalizes §6.6-0009(d). **This width is KISS's, not Fuel's:** Fuel evaluates FDX affine extents in i128 (i64 coefficient times u64 symbol, at most 4 terms), so a value past i64 but inside i128 is `ExtentOutOfRange` there (it exceeds `usize::MAX`), not an overflow. The two surfaces differ on purpose; a KISS vector at the i64 edge is a KISS vector, not an FDX one.
- **-0006 (order).** Evaluation is left to right; at each operand the symbol lookup comes first, then the arithmetic step, and the **first failing step** names the decline (so an overflow in term 1 beats an unbound symbol in term 2); the non-negative range gate runs last, after all steps. This is Fuel's `check_v17_affine_eval` order, verified by the Fuel lane; vectors pin it.

### 3.2 Reader obligations (explicit, any sub-standard)

Today a clause says "MUST NOT" to a producer or "a reader MUST reject" case by case (65 hits for "reader MUST" across spec/ at 5988cf1 per the issue's measurement; not recounted here). The ruling asks that reader duties be explicit. Proposal, in Conform §6.1 (traceability): every normative clause carries a **party** tag, one of `producer`, `reader`, `both`, in the sidecar entry (not the prose, so no spec bytes move for the tag alone). A `reader` or `both` clause whose test needs a second reader is listed in `UNBACKED.tsv` as `needs-second-reader`, with the reason, rather than omitted. The one reader rule the issue names, "honour a flag that says the base does not carry the meaning" (FDX `MEANING_REQUIRES_EXT`), is **enforced producer-side only in Fuel at build time** (V19/V22 `check_v22_meaning_bearing_implies_ext`); Fuel has no reader-side check for it, so the reader obligation is untested there. It has no KISS carrier today (KISS has no tensor-exchange header); the equivalent in KISS is the capability model: **a reader that does not understand a capability the contract marks required MUST decline, and MUST NOT decode as if it were absent** (checked against §7.1/§7.2 before it is written).

### 3.3 The instrument

- `conformance/src/bind.rs`: `pub fn bind(contract: &Contract, env: &Env) -> Result<Bound, BindDecline>`, which evaluates every Dispatch expression and re-runs each bind-time check. `BindDecline` is a closed enum with the three codes above. It is the **reference** the same way the region codec is.
- A vector family `conformance/corpus/bind_vectors.json`: `{contract, env, expect: ok | decline_code}`, byte-exact, emitted by a reference emitter (like `emit_contract_vectors_json`), with `generated_from` naming that emitter. Includes: the §2.5 `add` contract bound at three sizes, an unbound symbol, a negative extent, a `*` overflow, `i64::MIN / -1`, and precedence cases for -0006.
- Conform: no sixth modality. This is Modality 4 (§6.7) with an input-environment field, so **KISS-CONFORM-6.7-0002's "pin exactly one output" is unchanged**; a new clause `KISS-CONFORM-6.7-0006` states that a decline vector MAY carry a bound environment and that the harness MUST pass it.
- Negative controls (the R1.27 rule, `KISS-CONFORM-6.5-0011`): a deliberately wrong `bind` (wraps instead of declining; substitutes zero for unbound) must fail the family, each with a named mutant test.

### 3.4 What it costs to be wrong

- A second reader: the vectors are Rust-reference-only until Fuel's `validate_realize` and one other implementer run them. State it in `UNBACKED.tsv` as `PARTIAL` (a reference passes; no second implementation yet), as R1.27 does, rather than claim two-implementation coverage.
- Byte impact: a new corpus file and new clauses move no existing vector or key; Contract §6.12 is additive. Every pushed change still bumps the version; the PM allocates the number.

## 4. Not covered here

- Gather sequence length and the three unrun FDX checks (V12 alignment, V15 pointer zero, V21c): no KISS carrier; recorded as DECLINED rows in the Phase-1 ledger (#236), not as clauses.
- Data-dependent output sizes (KISS-OPS §6.20-0004): the verdict is a runtime value, not an environment; out of scope.
- Board 358 (DimExpr clamp for `6.13-0010`) is a related **bind-time** rule for a window extent; it should land inside §6.12 once ruled.

## 5. Staging

1. This note (docs only) for the PM and Fuel's FDX lane to check the table in section 2 against `validate.rs`.
2. Clause PR: Contract §6.12 (-0001..-0006), Conform §6.7-0006, sidecar party tags, `UNBACKED.tsv` rows, each with a `// Backs:` test.
3. Instrument PR: `bind.rs`, `bind_vectors.json`, emitter, mutants.
Steps 2 and 3 can be one PR; the PM allocates versions.
