# Options note: R1.23, precision floors and guarantees (five questions)

**Informative. No normative text. For CireSnave's decision; the PM turns each question into a board item.**
Read at `origin/main` `b9e08e7f`, 2026-10-26. Paths are `file:line` at that commit. Issues: KISS#1, #4, #13, #56,
#293 (Baracuda, Fuel, and the architect). Roadmap row: R1.23. Nothing here freezes. Q3 is **half answered** already.

---

## Q1. Is 10 bits the right floor for "reduced-mantissa-permitted"? (#1)

**Example.** A GPU multiplies two `f32` matrices after rounding each input to a shorter mantissa to use a faster unit.
The spec lets a target do this only if it keeps **at least 10 explicit mantissa bits** (`spec/ops.md:1691`,
KISS-OPS-6.17-0003). TF32 keeps 10, so it fits. Rounding `f32` inputs to bfloat16 keeps **7**, so it does not.

**Where it stands.** Ops Appendix D.1 calls the 10 "a reference value pending confirmation against real
reduced-precision hardware" (`spec/ops.md:3052-3057`). Fuel has added evidence on the issue: the cuBLAS compute type
that rounds `f32` inputs to bfloat16 (`CUBLAS_COMPUTE_32F_FAST_16BF`, next to the TF32 and FP16 ones) is a shipped,
widely used mode that the 10-bit floor rejects; such a kernel cannot be `bit-stable` (it rounds) and cannot be
`reduced-mantissa-permitted` (7 is under 10), so the two-value attribute cannot describe it at all. I have not
re-checked the cuBLAS enumerants; this is Fuel's report.

**Options.**
- **A. Confirm 10.** bfloat16-input rounding stays inexpressible for `f32` cells.
- **B. Lower the single floor to 7.** TF32, FP16 and bfloat16 emulation are all admitted; FP8-input modes (3 bits)
  are not. One number, still quantified.
- **C. Replace the global floor by a declared width.** A kernel declares its guaranteed mantissa width ("at least 7",
  "at least 10") and Conform checks the declaration with the per-atom width probe that already exists
  (`spec/ops.md:1712`, KISS-OPS-6.17-0005). Closest to the incumbent's own tiers; widens what a contract carries.

**Recommendation: B now, C if a further tier appears.** B fixes the real gap with a one-number change, keeps the
"no unquantified adjective" rule, and the attribute is a permission, never an obligation (6.17-0003), so lowering the
floor obliges nobody. C is the cleaner end state but adds a contract field before a second tier exists. **Changes
bytes?** No (the `st`/`rm` key tokens stay); the floor is a value in a clause.

---

## Q2. How does a consumer feed a contract's tolerance to the order-invariant comparator? (#4)

**Example.** A scatter-add kernel's result depends on the order the additions land. Its contract declares how far
results may differ. A consumer that verifies the kernel must read that declared number and use it, not invent one.

**Where it stands.** The comparator is required to use the contract's tolerance and "never an implementation-chosen
implicit default" (`spec/conform.md:963`, KISS-CONFORM-6.8-0004). The contract declares tolerance as an accuracy
tier (`spec/contract.md:1136`, KISS-CONTRACT-6.8-0002) and, for accumulator-keyed reductions, as a band
`k(S,A) * N * eps_A * |max partial sum|` (`spec/contract.md:1168`, KISS-CONTRACT-6.8-0012). The consumer-verify check
exists (`spec/conform.md:1216`, KISS-CONFORM-6.12-0002). What is **not** specified is the step in between: which
declared field becomes which comparator parameter. The reference harness offers a helper to *derive* a bound
(`conformance/src/structural.rs:345`, `reassoc_bound_f32`) but nothing reads a parsed contract into a comparator.

**Options.**
- **A. Specify the mapping** in KISS-Conform (declared tier or band to comparator threshold, with one worked vector),
  and build the reference wiring. Makes the ban on an implicit default checkable.
- **B. Leave it to each consumer-verify implementation.** Two verifiers may then derive different thresholds from one
  contract.
- **C. Build the reference wiring only** and keep the spec silent.

**Recommendation: A.** Without a mapping, 6.8-0004's prohibition cannot be tested, and two verifiers can disagree
about the same contract. It is additive. **What waits:** the same decision as Q5 (what the bound is measured against),
because the threshold is only as meaningful as its referent.

---

## Q3. Can a provider say "reproducible on this device" versus "varies run to run", and "more accurate than the reference"? (#13)

**Example.** A split-K matrix multiply gives the same bits every time on one GPU but not the same bits as a naive
sum. A cache wants to know it can reuse the result on that device. Separately, a variant that is *more* accurate than
the default wants to say so; today the tolerance bands are symmetric.

**Where it stands: the first half is already answered.** `bit_stability` (`{portable, same-hardware, none}`,
`spec/contract.md:1230`, KISS-CONTRACT-6.8-0013) states exactly "bit-identical on the same hardware" versus
"run-to-run variation is possible", orthogonal to the determinism class. **The second half, a directional "more
accurate than the reference" guarantee, is not in the specs**: tiers are tagged quantities against a reference and
carry no direction (`spec/contract.md:1136`, 6.8-0002).

**Options for the directional half.**
- **A. Do nothing.** A more-accurate kernel declares its (smaller) tier; a consumer cannot tell "tighter" from "just
  differently rounded".
- **B. Add a Guarantees field** for direction (for example `refinement: none | bounded`). Changes Contract bytes:
  Breaking (`contract_version` bump).
- **C. Fold it into Q4**: a refinement is what a "wider intermediate precision" kernel is; solve both together.

**Recommendation: A for now, and decide with Q4.** The measured case that motivated it (Q4) turned out to be
within 1 ULP in **both** directions, not a strict refinement, so a one-directional field would not have described it.
**Not a decision yet** for the first half: it is done.

---

## Q4. Should a kernel computed with wider intermediate precision be allowed to say so? (#56)

**Example, measured by Baracuda on an RTX 4070 over 536,870,912 input pairs.** The kernel `out = g/(g+1)`,
`g = a*b`, is emitted with the constant `1.0` as a C `double`, so the add and divide run in double and round once to
`f32`. That differs from rounding every step in `f32` on **13.9%** of inputs, by **exactly 1 ULP**, and the double form is
closer to the true value 73% of the time and farther 27% of the time. The op is forced into the `exact-byte` class by
its content (`spec/ops.md:561`, KISS-OPS-6.0-0002), and `exact-byte` forbids removing an intermediate rounding
(`spec/ops.md:596`, 6.0-0006). So the emitted kernel does not honestly claim the class it must claim.

**Options.**
- **A. Fix the generator**: emit `1.0f`, so the kernel is genuinely `exact-byte`. No standard change. The kernel
  loses its extra accuracy on about three quarters of the inputs where it differed.
- **B. Add a third MathFidelity value** ("wider intermediate precision permitted") for kernels that carry the class
  `ULP/tolerance` and a declared 1 ULP tier. Enlarges the two-member enum (`spec/ops.md:1667`, KISS-OPS-6.17-0001),
  needs an exception to 6.0-0002's class-by-content rule, adds a third `<mp>` code (additive key token), and is
  imported by Contract 6.8-0004.
- **C. Do A now and take B together with the fused-multiply-add ruling** (a kernel that contracts cannot claim
  `bit-stable`: the same question in another form).

**Recommendation: A now (Baracuda-side), then decide B with the contraction ruling.** Both questions ask "how does a
kernel honestly declare that it computed with fewer rounding steps than the reference?", and answering them separately
risks two enum members that mean nearly the same thing. A needs nothing from the standard, and the measured divergence
is small and symmetric, so there is no urgency. **What waits:** the contraction ruling (ROADMAP R1.11).

---

## Q5. What is a class-level ULP bound measured against? (#293)

**Example.** Two implementations of `exp` each stay within 4 ULP of the *true* value. They can differ from each
other by up to 8 ULP (the triangle inequality, twice). A test that says "implementation X agrees with implementation Y
within 4 ULP" is a weaker claim than "X is within 4 ULP of the truth", and the weaker one is easier to satisfy.

**Where it stands.** For a single transcendental **atom** the text says the bound is measured "against an audited
wide-precision reference" (`spec/ops.md:871`, KISS-OPS-6.8-0001). For an **op** that merely *contains* a transcendental,
the class clause names the comparator and not the referent (`spec/ops.md:572`, KISS-OPS-6.0-0003;
`spec/conform.md:948`, KISS-CONFORM-6.8-0002 says "within the op's declared per-target ULP bound" with no referent). The
Contract does name a reference function for every kernel (`spec/contract.md:1136`, 6.8-0002), but does not say that
function's own precision must be wide.

**Options.**
- **A. Propagate the atom's sentence** to 6.0-0003 and the Conform comparator: the declared bound is measured against
  an audited wide-precision reference, and agreement with another implementation is never recorded as a ULP result.
- **B. Leave the atom-only anchor.**
- **C. As A, plus a vector that fails a two-implementation differential claiming a ULP result.**

**Recommendation: A, with C as its test.** One sentence closes a gap where a shared defect would be filed as agreement
(the failure direction nobody investigates). No bytes change. **This decides what Q2's threshold means**, so rule on it
first.

---

## Summary table

| Q | Issue | Recommend | Bytes change? | Note |
|---|---|---|---|---|
| 1 | #1 mantissa floor | B (floor 7), C later | no | Fuel's bf16 evidence |
| 2 | #4 tolerance wiring | A (specify the mapping) | no | rule Q5 first |
| 3 | #13 reproducibility / direction | A, decide with Q4 | no | `bit_stability` already answers half |
| 4 | #56 wider intermediate precision | A now, B with the contraction ruling | no | measured, symmetric 1 ULP |
| 5 | #293 anchor the class bound | A (+C vector) | no | rule before Q2 |
