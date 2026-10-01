# Question: where does the closed precision-class token set live, and does Fuel's `AccuracyClass` belong in it?

**Status:** question / draft, not wired · 2026-10-01 · surfaced while reconciling KISS-Ops with Fuel (determinism enum, S12)
**Affects:** KISS-Contract §6.7-0004 / §6.7-0005 / §6.8-0002; KISS-Ops §6.8 (declared accuracy tier)
**Proposes no clause text and allocates no clause id.**

## Observation 1 — KISS-Contract imports a vocabulary KISS-Ops does not contain

KISS-Contract §6.7-0005 requires the Capabilities `precision_class` to be "drawn from the
**closed precision-class token set imported verbatim from KISS-Ops**" and to map to a
per-backend tier by "the KISS-Ops precision-class↔tier correspondence (KISS-OPS §6.8)", with
`correctly-rounded` and `bit-reproducible` mapping to tier 0. A KISS-Contract appendix
example also uses `precision_class = strict` ("maps to accuracy tier 0", `spec/contract.md` ~L1901-1912).

`spec/ops.md` §6.8 (read at base `7624cbb`) contains no such token set and no
class↔tier correspondence. It defines a per-target **declared accuracy tier** (§6.8-0001,
`{max_ulp, ...}` as a tagged quantity, flat in v1 per §6.8-0006) and an informative advisory
floor table. Positive control for the search: the same grep finds "accuracy tier" and
"correctly rounded" in `ops.md`; it finds none of `precision-class`, `precision_class`,
`bit-reproducible`. So the "closed token set imported from KISS-Ops" has no owner today, and
`strict` (used in the Contract example) is not in the token list the Contract itself names.

## Observation 2 — Fuel has a five-rung ordered class derived from the same inputs

Fuel `fuel-dispatch/src/ranker/cost_vector.rs` (Fuel sha `eee8e119d7c0b09b031b9e6703f38ade136675ff`,
line 72) defines `AccuracyClass`, an ordered discrete summary used as a dominance axis:

| rung (rank) | derived from the kernel's precision guarantee |
|---|---|
| `CorrectlyRounded` (4) | `max_ulp == 0` |
| `BoundedTight` (3) | `max_ulp` in `1..=2` |
| `BoundedLoose` (2) | `max_ulp > 2` |
| `Stable` (1) | no ULP bound stated, `bit_stable_on_same_hardware` |
| `Unbounded` (0) | no ULP bound stated, not bit-stable |

It is *derived* (function `accuracy_class`), not declared, and it sits beside, not inside,
Fuel's determinism enum `{bitwise, same_hardware_bitwise, nondeterministic}`.

## The question for the editor of record

1. Is the intended owner of the Contract's closed precision-class token set KISS-Ops §6.8, as
   §6.7-0005 says, or should the Contract own it (and §6.7-0005's "imported verbatim from
   KISS-Ops" be corrected)? Either way one place must list the tokens, and the `strict` /
   `bit-reproducible` / `correctly-rounded` spellings must agree.
2. If a closed ordered class set is wanted, is a *derived* ladder like Fuel's (tier -> rung,
   so a consumer ranks candidates without reading raw ULP numbers) in scope, or does KISS keep
   only the declared tagged tier and leave ranking to consumers? Note Fuel's `Stable` /
   `Unbounded` rungs read a reproducibility fact (`bit_stable_on_same_hardware`), which in
   KISS lives in the Contract's `bit_stability` field (§6.8-0005), not in an accuracy tier.
3. If a ladder is adopted, is it derived (a function of the tier and `bit_stability`) or
   declared? A declared class can contradict its tier; §6.7-0005 already forbids that
   contradiction, which suggests derived.

## Why this is separate from the determinism-enum item

The determinism enum answers "how must the conformance harness compare outputs" and (in
Fuel) "how portable is a bit-identical result". The accuracy class answers "how tight is the
result" and is a ranking axis. Folding the second into the first would change what
`order-invariant/nondeterministic` means for comparator selection (§6.0, KISS-Conform §6.8).
Nothing here edits the enum; no clause, test name, sidecar row or ledger row is added.
