# Options: repairing the reassociation band of KISS-CONTRACT-6.8-0012 (R1.16, KISS#413)

Status: **options with a recommendation; nothing here is a clause.** Prepared 2026-10-31 against `origin/main` 0208b57 (conformance 0.7.1). The band is `k(S,A) · N · eps_A · |max partial sum|`, an absolute ULP-of-A band against the §6.17-0009 per-cell reference. KISS#413 reports three defects, all from kiss-ref (the one implementation that binds the clause). Control: the clause text was read from `git show origin/main:spec/contract.md`, lines 1189-1210.

## Defect 2 first: whose schedule? (measured: the text already half-answers it)

- 6.8-0012 says the tolerance MUST bound reassociation error "against the per-cell reference of KISS-OPS §6.17-0009", and 6.17-0009 pins that reference to the **ascending-index** evaluation with inputs rounded to S, each accumulate rounded to A. But it never says that `|max partial sum|` is *that evaluation's* partials. A reader can still take the candidate's own schedule (unknowable to a comparator) or a supremum over schedules.
- **Fix, no choice needed:** say `|max partial sum|` is the maximum absolute running partial of the §6.17-0009 reference evaluation. It is the only schedule a comparator can compute, and it matches what kiss-ref did by fiat.

## Defect 3: the zero band (reasoned here, not run; please treat as a claim to check)

- For a **sum** in IEEE arithmetic without flush-to-zero: if every ascending partial is 0 then every input is 0 (induction: `p1 = x1`, and `p_i = fl(0 + x_i) = x_i`). Then every schedule also gives 0, so deviation is 0 and a zero band is correct, not vacuous.
- So once defect 2 is fixed, the zero band is reachable only through (a) flush-to-zero or denormals-are-zero hardware, or (b) a contraction whose per-term products all underflow to 0 in A. Both are degenerate, and a nonzero deviation there is real evidence, not noise.
- KISS#413's claim that the zero band is reachable "because of defect 2" holds, and this note adds that it is nearly unreachable once defect 2 is fixed. kiss-ref reports its guard never trips.
- **D1. State it:** a zero band means the comparator requires exact equality; no new term.
- **D2. Floor term:** band := `k · N · eps_A · max(|max partial sum|, tiny_A)` with `tiny_A` the smallest normal of A. Tolerates FTZ hardware; adds a term to the formula.

## Defect 1: the constant `k(S,A)`

- No theory ceiling exists in terms of `|max partial sum|`: under cancellation the sum of absolute values can far exceed it, so a sound `k` can legitimately be large. Any ceiling on `k` in this form is empirical, i.e. a number.
- **K1. Method, not value:** `k` MUST be calibrated by a stated method (worst measured ratio of deviation to `N · eps_A · |max partial sum|` over a published corpus, times a margin of at least 1), and the method, corpus and margin MUST be recorded in the contract's provenance. KISS defines the format; the number stays the implementation's (the line CireSnave drew for R1.36, board 338: format yes, numbers no).
- **K2. Pinned ceiling per cell:** KISS publishes a maximum `k` per (S,A). Rejected: it is a number KISS cannot measure, and it forces a corpus.
- **K3. Change the scale term to the sum of absolute values `Σ|x_i|`** (standard forward-error analysis): then `k ≤ 1` is theory, any schedule obeys it, and the schedule question (defect 2) and the zero band (defect 3) both disappear (`Σ|x_i| = 0` iff all inputs are 0). Cost: a looser band under cancellation, which may stop discriminating; I have not measured it against kiss-ref's calibration data, so it is a hypothesis.

## Recommendation

Fix defect 2 now (no choice). **D1** for defect 3 (nothing to add). **K1** for defect 1. Hold **K3** as the alternative only if kiss-ref's data show the `|max partial sum|` band cannot be calibrated stably. All three together change comparator semantics but no wire bytes; the clause is `[go]` once the PM or CireSnave picks K1 vs K3 and D1 vs D2. Test would extend `test_contract_accumulator_tolerance_band` with a zero-partials case and a named-schedule case.
