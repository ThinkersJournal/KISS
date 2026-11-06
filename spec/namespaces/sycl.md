# The `sycl:` capability-set vocabulary

**Namespace:** `sycl` · **Maintainer:** [bonefish](https://github.com/ThinkersJournal/bonefish) · **Vocabulary version:** 1 · **Status:** draft (proposed by the Bonefish lane, recorded by KISS; **nothing produces `sycl:` tokens yet**)

> **This is a maintainer-owned annex, not a KISS clause.** Under
> [KISS-CLASSIFY-6.8-0004](../classify.md), the `sycl` namespace's capability-set
> vocabulary is owned by its maintainer (bonefish); KISS pins only the token
> *grammar* and the byte-exact match rule, never this vocabulary. This document is
> normative for the `sycl:` namespace only, is referenced from
> [`conformance/registry/namespaces.json`](../../conformance/registry/namespaces.json),
> is **not** part of KISS-Classify's clause set, and freezes on its own axis. Where it
> appears to contradict KISS-CLASSIFY-6.8-0001 / -0002 / -0005 or the general encoding
> rules, **the KISS clause wins.** Source of this text: Bonefish's
> `docs/sycl-namespace-proposal.md` at bonefish `5e4398c`; the maintainer's copy and the
> `TOKENS` list in `bonefish-sycl-emit/src/target.rs` are authoritative for the token set.

**Reference implementation:** `bonefish-sycl-emit` (`target.rs`, `TOKENS`). Informative.

---

## 1. What the token names

As for `cuda:`, a `sycl:` token names **the specialization a kernel was built for**, not
the maximum capability of the part it runs on. Matching is byte-exact
(KISS-CLASSIFY-6.8-0002): there is no range, prefix or implication.

## 2. Grammar

```text
sycl:spir64                   portable: JIT-compiled SPIR-V, the default `icpx -fsycl` target
sycl:intel-gpu-<arch>         ahead-of-time for one Intel GPU architecture
```

- **S-1** The token is exactly `sycl`, a single `:`, and one capability-set field: a
  **single scalar**, with no `.`-separated fields and no member list. KISS-CLASSIFY-6.8-0006
  (fixed-width juxtaposition) is therefore vacuous here and 6.8-0007 (digest fallback) is
  unreachable.
- **S-2** After the `:` every character is lowercase ASCII from `[a-z0-9-]`. `-` is
  admitted: KISS-CLASSIFY-6.8-0005 forbids only `|`, `;`, `/`, whitespace and control
  bytes, and `vulkan:` uses `-` too (`cuda:` excludes it by its own rule C-3).
- **S-3** `<arch>` is the **canonical** (non-alias) `intel_gpu_*` entry of DPC++'s
  `device_architecture.def` with every `_` replaced by `-`. **Aliases get no token**
  (`intel_gpu_dg2_g10` is another name for `intel_gpu_acm_g10`; a second token would split
  one cell in two under byte-exact matching). 48 of the 143 entries of the 2026.1 table
  are aliases.

## 3. Token set (36 tokens at the time of recording)

`sycl:spir64`, and `sycl:intel-gpu-<arch>` for each canonical Intel GPU entry:

`bdw skl kbl cfl apl glk whl aml cml icllp ehl tgllp rkl adl-s adl-p adl-n dg1 acm-g10
acm-g11 acm-g12 pvc pvc-vg mtl-u mtl-h arl-h bmg-g21 bmg-g31 lnl-m ptl-h ptl-u wcl nvl-s
nvl-u nvl-p` (34, from the DPC++ 2026.1 table) and `cri` (added by DPC++ 2026.2; absent
from 2026.1).

**The set grows with the compiler.** A new architecture is an additive spelling; the
maintainer records, per token, the first DPC++ release that produces it. Out of scope:
`nvidia_gpu_*` and `amd_gpu_*` entries of the same table (they belong to `cuda:` and the
HIP namespace) and Intel CPU entries.

## 4. Derivability witness (KISS-CLASSIFY-6.8-0014)

| Entry | Witness (upstream construct) |
|---|---|
| `sycl:intel-gpu-<arch>` | the `__SYCL_ARCHITECTURE(intel_gpu_<arch>, ...)` line (not `_ALIAS`) in `device_architecture.def` |
| `sycl:spir64` | `-fsycl-targets=spir64`, accepted by `icpx` |

The witness is **upstream** (DPC++'s own table), not code of the emitter. **It proves
REACHABILITY of a token only:** it does not say that any device admits the token, nor that
the emitter's output was run on that architecture. Measured against DPC++ 2026.1 and 2026.2
by compile checks only; no Intel GPU run exists, and only `intel_gpu_acm_g10` was compiled
ahead of time. Gate: `bonefish-sycl-emit/tests/sycl_target_tokens.rs` (maintainer's CI).

## 5. Admitted sets (unmeasured)

Whether a device admits `sycl:spir64` in addition to its own `sycl:intel-gpu-<arch>` token
(by the design of SPIR-V JIT) is **unmeasured on a GPU** and is not claimed here.

## 6. Open items

- `unpopped-vocab`'s `ArchSku` has no SYCL variant; whether the target becomes variants
  there or stays a free token is the Unpopped lane's decision.
- Registration (KISS-CLASSIFY-6.8-0003) precedes production: no party may announce a kernel
  under `sycl:` until the registry row is `registered`, which this change makes so.
