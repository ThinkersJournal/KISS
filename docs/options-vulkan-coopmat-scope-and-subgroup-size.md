# Options: cooperative-matrix scope and subgroup-size range in the Vulkan vocabulary

Status: **options note, nothing decided, no spec text changed.** Task: pm-kernels `vocab-v8-cooperative-matrix-and-subgroup-size` (asked by vulkane and fishstringer E15). Read at `origin/main` e2d7ef95. Public Vulkan facts below are from memory of the public specification; section numbers are **UNVERIFIED** (not re-read this session), struct and field names are the ones the registry defines.

## 1. What KISS already carries (measured: `spec/namespaces/vulkan.md`)

- **Cooperative-matrix shapes: yes, as a key field.** `<coop>` is `cm-<tuple>[,...]`, a tuple `M-N-K-A-B-C-R[-sat]` (§2.4), canonically ordered (V-8), digest above 512 bytes (V-9); an unnamed component type omits the whole tuple and is reported in `unnamed_types` (V-7, KISS-ANNOUNCE-6.6-0011). Source: `VkCooperativeMatrixPropertiesKHR` (`MSize`, `NSize`, `KSize`, `AType`, `BType`, `CType`, `ResultType`, `saturatingAccumulation`, `scope`).
- **Subgroup width: yes, as a key field**, but only a point: `sg<N>` (pinned) or `sgdyn` (V-4, §2.1).
- Nothing in `spec/` names `scope` for a cooperative-matrix tuple (0 hits for "scope" in `vulkan.md`; the same grep finds `cm-`, so the file is the right one).

## 2. The gaps (what vulkane and fishstringer wait on)

**G1, scope.** `VkCooperativeMatrixPropertiesKHR.scope` (`VkScopeKHR`: Device, Workgroup, Subgroup, QueueFamily) is dropped by the tuple. Two device-reported entries equal in every other field but different in scope spell **the same tuple**, so a kernel built for one is matched to a device that offers only the other. This is a wrong-match risk in the sense of the trigger already recorded for the `c` hint (a real wrong match). Slang's coopmat lowering of Unpopped's TileMma needs the scope to choose the SPIR-V cooperative-matrix scope operand.

**G2, subgroup range.** A device reports a *range* (`VkPhysicalDeviceSubgroupSizeControlProperties`: `minSubgroupSize`, `maxSubgroupSize`, `requiredSubgroupSizeStages`; features `subgroupSizeControl`, `computeFullSubgroups`; per-pipeline `requiredSubgroupSize`) plus the default `VkPhysicalDeviceSubgroupProperties.subgroupSize`. The key holds one width for the *kernel*; no KISS record says what range the *device* can pin. Classify's example text already assumes "a device whose pinnable subgroup range is 32..=64" (`spec/classify.md` ~2084) with no record that states it.

**G3, unknown versus not supported.** Nothing distinguishes "the device was asked and has no such shape/range" from "the provider did not or could not ask". `cm-none` and `sg`/`sgdyn` describe the *kernel*, not the device. A consumer reading an absent shape cannot tell the two apart, which is exactly how an over-claim or an under-claim enters unnoticed.

## 3. Options

**Q1 (scope).** A: add scope to the tuple (`cm-sub-16-16-16-...`, prefix or suffix): byte-moving for every `cm-` token, **vocabulary version 9, Breaking**, needs CireSnave's verbatim word. B: leave the key, record scope only on the Announce device record (a new trailing list, also Breaking for every `DVRS` body). C: pin the key to **Subgroup scope only**, state it in a clause, and a Workgroup/Device-scope tuple is omitted whole like an unnamed type (V-7) and reported (extend `unnamed_types` with `vulkan:scope:<n>` raw ids): under-claims, safe, but the TileMma lowering for a workgroup-scope device stays unreachable. **Recommend C now** (it states what the bytes already mean, moves nothing for devices that report Subgroup scope only, and uses the existing urgent-naming path), A when a consumer needs a non-Subgroup tuple.

**Q2 (subgroup range).** A: a device-record trailing record `subgroup_range` (u32 min, u32 max, u8 required-size-control, u8 full-subgroups): Breaking. B: nothing in KISS; vulkane keeps it in its own capability data (the key's `sg<N>` already says which width the kernel needs; the range only matters to *choosing* a width, a launch-time act and a kernel internal under the SCOPE RULE). **Recommend B**: SCOPE RULE (CireSnave 2026-11-21): KISS does not concern itself with kernel internals; a pinnable range selects an `sg<N>` the consumer then matches byte-exactly against the key, so the outward input/output (the key) is already complete.

**Q3 (unknown versus not supported).** If a record is ever added (Q1 B, Q2 A), carry a u8 per fact: `0` unknown (not queried or query failed), `1` not supported (queried, absent), `2` supported; **a reader MUST NOT treat 0 as 1 or 2**. Not needed under Q1 C + Q2 B, where the question does not arise on the wire. Draft clause text if wanted later: *"A device record fact is reported in one of three states: unknown, not supported, supported. `unknown` MUST NOT be read as either other state; a producer that did not query MUST report `unknown`, never `not supported`."* Nothing minted until a record exists.

## 4. What I need

CireSnave's word (via pm-kernels) on Q1 (C recommended, no byte moves) and Q2 (B recommended). Under the recommendation the only spec change is one clause in `spec/namespaces/vulkan.md` (cm tuples are Subgroup scope; other scopes omitted whole and reported), with its sidecar entry and a red-first test; **no stored byte moves** and it is a Draft change. Under Q1 A, Q1 B or Q2 A the change is Breaking and waits.

— lane: kiss
