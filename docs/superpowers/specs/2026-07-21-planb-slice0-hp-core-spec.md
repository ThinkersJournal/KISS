# Plan B Slice-0 — T1 `hp-core` `BigFloat<N>` design spec (recovered)

Recovered by a focused agent after the design workflow's hp-core agent hit the schema retry cap. It reconciles a **real cross-spec drift** (the four sibling kernel specs declared the `BigFloat` type with three different exponent/field conventions) onto the **round-ziv §0 substrate** as normative. Difficulty: high.

Satisfies **KISS-CONFORM-6.5-0007** (the "wider than compute dtype, round once" floor IS this core) and feeds **6.5-0009** (`stabilized_precision_bits = 64·N`).

---

## 0. Cross-spec drift to reconcile FIRST (blocking)

The sibling specs disagree on field names / exponent semantics. Reconcile before coding or the kernels won't link:

| Spec | `exp` means | sign | significand field | error |
|---|---|---|---|---|
| **round-ziv §0 (authoritative)** | weight of significand LSB: `value = (−1)^sign·m·2^exp`, `m∈[2^(P−1),2^P)` | `bool` | `mant:[u64;N]` | separate `Evaluated{val,err_ulps:u128}` |
| reduction §0 | MSB binary exponent `Ebin`, significand `[1,2)` | `i8` | `limbs:[u64;L]` | delegates |
| atan/clog §1 | MSB binary exponent, significand `[1,2)` | `i8` | `sig:[u64;W]` | inline `ulp_err:u64` |
| exp/log §0 | via `exp2()` accessor | — | `[u64;W/64]` | `Eval{val,err}` |

**Decision (mandated):** stored form is round-ziv's — `sign: bool`, `exp: i32` = **LSB weight**, `mant: [u64; N]` normalized so bit P−1 set, `class: FpClass`. Its error contract `|val−v_true| ≤ err_ulps·2^exp` is only valid in this convention, so it is normative. Expose accessors so the MSB-exponent specs map mechanically: `ebin()` (= `exp + P − 1`), `from_limbs_ebin(...)`, `sign_i8()`, `mant` doc-aliased `sig`/`limbs`. **Implementer action:** wherever reduction/atan write `exp`/`sig`/`limbs`/`sign:i8`, translate to `ebin()`/`mant`/`sign:bool`.

## 1. Representation & invariants

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FpClass { Normal, Zero, Inf, NaN }   // Zero carries sign; Inf/NaN only from atoms/semantics.rs

/// N=4->256-bit, 8->512, 16->1024. Working precision P = 64*N.
#[derive(Clone)]
pub struct BigFloat<const N: usize> {
    pub sign:  bool,        // true = negative (Normal and Zero)
    pub exp:   i32,         // LSB weight: value = (-1)^sign * (mant as P-bit int) * 2^exp
    pub mant:  [u64; N],    // NORMALIZED for Normal: bit (P-1) set, mant in [2^(P-1), 2^P)
    pub class: FpClass,
}

/// atom -> round-ziv contract: |val - v_true| <= err_ulps * 2^(val.exp)
pub struct Evaluated<const N: usize> { pub val: BigFloat<N>, pub err_ulps: u128 }
```

- **Limb order (pinned):** `mant[0]` = MOST-significant limb; leading normalized `1` is bit 63 of `mant[0]` (`mant[0] & 0x8000_0000_0000_0000 != 0`). Big-endian, matching reduction/atan `limbs[0]`/`sig[0]`.
- **Invariants** (debug `check_invariant`): Normal => bit P−1 set & not all-zero; Zero => `mant==[0;N]`, sign meaningful; Inf/NaN => mant don't-care (canonicalize `[0;N]`); `Ebin = P−1+exp`, `|value| in [2^Ebin, 2^(Ebin+1))`.

## 2. Constructors (exact unless noted)

`from_f64` / `from_f32` (EXACT — 53/24-bit significand ⊂ P bits; handle normals, subnormals via clz-normalize, ±0, ±Inf, NaN), `from_i64` / `from_u64` (exact, |k|<2^63), `one()`, `zero(sign)`, `from_limbs(sign,exp,mant,class)` (LSB-weight native), `from_limbs_ebin(sign,ebin,mant,class)` (MSB-exp for constant tables; stores `exp = ebin−(P−1)`). Helper `leading_f64()` (top ~53 bits back to f64 for Newton seeds; NOT exact).

`from_f64(1.0)@N=4` => `exp=−255, mant=[0x8000...0,0,0,0]`, `ebin()=0`, value = 2^255·2^−255 = 1.

## 3. Normalization & shifting

`clz_limbs(&mant)->u32`; `shl_bits`/`shr_bits(&mut mant, k)` (k may exceed 64, dropped bits lost); `shr_bits_sticky(&mut mant,k)->bool` (sticky OR of dropped bits, for RNE); `renormalize(&mut self)` (all-zero->Zero; else left-shift by clz so bit P−1 set, `exp −= clz`; caller handles carry-above-P by shr 1 + exp+=1). Value-invariant.

## 4. `add` / `sub`

`add`/`sub(&self,&other)->(Self, u128 rounding_err)`, `neg`/`abs` (exact). Algorithm: (1) special classes (NaN/Inf/±0 incl. `(−0)+(−0)=−0`); (2) align to the LARGER `Ebin` in `P+G` (G=64 guard) scratch, smaller operand shr with sticky; (3) combine (same sign add, carry->shr1/exp+=1/sticky; opposite sign subtract smaller, equal-mag->Zero(+0)); (4) **cancellation renormalize** (clz + left-shift, exp drops by cancellation count — where exp can fall ~P); (5) RNE round `P+G`->P (`round_up = R && (S || (q&1))`); (6) `err_ulps += 1` rounding; input error re-scaled to result ULP by the caller (cancellation inflates it).

## 5. `mul`

`mul(&self,&other)->(Self,u128)`: sign = xor; EXACT 2N-limb schoolbook product `Pi = m_a·m_b` (O(N^2) u64×u64->u128; N<=16 => <=256 mults); `L=bitlen(Pi) in {2P−1,2P}`; `shift=L−P`; `m_r = Pi>>shift`, `R=bit(shift−1)`, `S=OR bits[0,shift−1)`; `exp_r = exp_a+exp_b+(L−P)`; RNE; carry->shr1/exp+=1; `err_ulps=+1` (0 if exact). `mul_small_int(n)` for k·ln2, e·ln2.

Tie check (vector M2): `m_a=2^255+1`, `m_b=2^255+2^254`, exp 0/0 => `Pi=2^510+2^509+2^255+2^254`, L=511, shift=255, `m_r=2^255+2^254+1`, R=1,S=0 tie, odd->up => `2^255+2^254+2`, exp_r=255.

## 6. `div` (GENERAL — review-mandated) and `div_small`

> Explicitly resolves the exp/log review blocker: `log`'s `s=(m−1)/(m+1)` and `atan2`'s `|y|/|x|`,`1/w` divide two arbitrary ~P-bit values, NOT by a small int. A general `div` is REQUIRED.

`div(&self,&d)->(Self,u128)`: correctly-rounded RNE, `err_ulps=+1`. **Normative algorithm = restoring long division** (airtight err): classes/sign; `ratio=m_s/m_d in (0.5,2)`; bit-at-a-time long division producing P+1 quotient bits, restoring subtraction, final nonzero remainder = sticky; normalize P+1->P with RNE; `exp_r = exp_s−exp_d−(P−1 or P)` per where the leading 1 lands; `err_ulps=+1`. Cost O(P·N) ~ 1024 u64-ops at N=4 — fine for a dev-time minter. **Optional Newton reciprocal fast path** permitted but its residual `m_s−q·m_d` MUST be folded into `err_ulps` (unproven Newton bound inadmissible). Recommendation: ship long division as normative. `div_small(&self,n:u64)->(Self,u128)` = single-limb long division, exact + sticky, RNE.

`div(1.0,3.0)@N=4` => `exp=−257, mant=[0xAAAA...AA, x3, ...AAAB]`, err_ulps=1 (vector D1); `div_small(1.0,3)` identical (D2).

## 7. Comparison + exact `abs_diff` midpoint primitive

`cmp(&self,&other)->Ordering` (sign-aware, +0==−0; by class->sign->Ebin->cmp_mag); `abs_diff_limbs(&a,&b)->[u64;N]` (EXACT big-int |a−b| with borrow — **this is what round-ziv §4 calls for distance-to-midpoint `D = abs_diff(F,Fmid)`; never truncate D**); `dist_gt_err(&d,err_ulps)->bool` (big-int D vs u128 err, D can reach ~2^(P−1)). Also: `ldexp(k)` (exact ·2^k), `exp2()`/`ebin()`, `round_to_i64()` (nearest int, ties-even, |value|<2^62 only — large-x pre-clamped to ±Inf/±0 by the atom BEFORE reduction, so k-overflow never reaches here), `is_zero`, `sign_i8`.

## 8. Escalation (const-generic N) — re-derive, don't zero-extend

`BigFloat<N>` const-generic over N in {4,8,16}. round-ziv driver dispatches over concrete monomorphizations `try_width::<4|8|16>`; unresolved at 1024 => `panic!` (mis-routed special value or err-bound bug). Conversions: `widen::<M>()` (M>N, exact zero-extend BELOW low limb, `exp −= 64·(M−N)`, err unchanged — ONLY for provably-exact values); `narrow::<M>()` (M<N, keep top M limbs, RECORD dropped tail as err — to read a wide stored constant at working width).

**Critical:** on a Ziv straddle the atom **re-evaluates from the original exact f64 input at the wider width** (`from_f64(x)` is exact at any N>=4), running the reduction against the wider constant view — this adds ~256 genuinely-correct bits per step, shrinking `err_ulps` while true `D` is fixed, so `D>err_ulps` eventually holds. Zero-extending an INEXACT 256-bit result preserves its error and shrinks nothing (the spin-to-1024-panic failure). Corollary: constant tables must be provisioned to the max escalation width (2/pi~2304 bits, ln2~1152 bits) — reduction's obligation; T1's `narrow` is how each width reads the right slice. Note LSB-weight `exp` is width-dependent (1.0: exp=−255@N4, −511@N8); `ebin()` is width-invariant and is what higher kernels compare.

## 9. `err_ulps` accounting

`err_ulps` = integer count of the RESULT's ULP (`2^exp`), so cancellation is auto-penalized (subtract dropping exp by c multiplies inputs' error by 2^c).

| primitive | contribution |
|---|---|
| from_f64/f32/i64, one, zero, neg, abs, ldexp, widen | 0 (exact) |
| narrow(N->M) | +ceil(dropped_tail / result-ULP) |
| add/sub | `err_a·2^(exp_a−exp_r) + err_b·2^(exp_b−exp_r) + 1` |
| mul | `err_a + err_b + 1` |
| mul_small_int | +1 if low bits dropped else 0 |
| div (long div) | `err_num + err_den + 1` |
| div_small | +1 if remainder!=0 else 0 |

Each rounding uses `+1` as an integer bound on the true <=0.5 ULP (round-ziv treats err_ulps as opaque non-negative, `D>err_ulps` strict, so conservative-by-1 is safe). Atoms sum across the chain and **fold in series/asymptotic truncation** before handing round-ziv `Evaluated{val,err_ulps}`. ~50-op exp/log chain => err_ulps < 2^12 (>= P−12 ~ 244 bits @N4), so the curated set decides at 256; escalation is a safety net.

## 10. Public API summary (T2–T8 link against)

construction: `from_f64 from_f32 from_i64 from_u64 one zero from_limbs from_limbs_ebin`
arithmetic (-> `(BigFloat<N>, u128)` unless exact): `add sub mul div div_small mul_small_int neg abs ldexp`
queries/conv: `cmp is_zero exp2 ebin sign_i8 leading_f64 round_to_i64`
exact-int for round-ziv: `abs_diff_limbs dist_gt_err`
escalation: `widen::<M> narrow::<M>`; contract `Evaluated<const N>{val,err_ulps}`.

## 11. Test vectors (each a ready failing test; N=4/P=256, mant MSW-first)

| # | Primitive | Input | Expected | err |
|---|---|---|---|---|
| C1 | from_f64 | 1.0 | exp=−255, mant=[0x8000..0,0,0,0], ebin=0 | 0 |
| C2 | from_f64 | 3.0 | exp=−254, mant=[0xC000..0,0,0,0], ebin=1 | 0 |
| C3 | from_f64 subnormal | 2^−1074 (0x..01) | exp=−1329, mant=[0x8000..0,0,0,0], ebin=−1074 (clz branch) | 0 |
| M1 | mul truncate | (2^255+1,e0)·(2^255+2,e0) | exp=255, mant=[0x8000..0,0,0,3] (R=0 truncates 2^1) | 1 |
| M2 | mul tie->even up | (2^255+1)·(2^255+2^254) | exp=255, mant=[0xC000..0,0,0,2] | 1 |
| M0 | mul exact | 3.0·3.0 | 9.0: exp=−252, mant=[0x9000..0,0,0,0] | 0 |
| S1 | sub cancellation | 1.0 − (1−2^−100) | 2^−100: exp=−355, mant=[0x8000..0,0,0,0], ebin=−100 | 0 |
| A1 | add guard | 1.0 + 2^−300 | 1.0 (b all-sticky, R=0) | 1 |
| D1 | div GENERAL | 1.0/3.0 | exp=−257, mant=[0xAAAA..AA x3, ..AAAB] | 1 |
| D2 | div_small cross-check | from_f64(1.0).div_small(3) | identical to D1 | 1 |
| D3 | div_small exact | 7.0.div_small(7) | 1.0 | 0 |
| D4 | div_small x1/2 | 1.0.div_small(2) | 0.5: exp=−256, mant=[0x8000..0,0,0,0] | 0 |
| X1 | abs_diff_limbs borrow | 2^64 vs 2^64−1 | 1 | — |
| X2 | cmp | 1.0 vs 1+2^−52 | Less | — |
| W1 | widen::<8> | from_f64(1.0)@N4 -> N8 | exp=−511, mant=[0x8000..0, x7 zeros] | 0 |
| R1 | round-ziv seam | mant exactly on an f64 midpoint @width4 with err>0 | T2 escalates (D==0 straddle) | — |

## 12. Review-driven requirements satisfied

(a) **GENERAL `div` exists** (§6, restoring long division, err_ulps=1) — resolves the exp/log review's blocker (prior contract listed only `div_small`); D1/D2 pin agreement. (b) **Type is round-ziv §0's exact substrate** — `sign:bool`, LSB-weight `exp:i32`, normalized `mant:[u64;N]`, `class`; contract `Evaluated{val,err_ulps}`; `Ebin=P−1+exp`, `D=abs_diff(F,Fmid)` consume it unchanged; reduction/atan MSB-exp convention reconciled via `ebin()`/`from_limbs_ebin()`/`sign_i8()`.
