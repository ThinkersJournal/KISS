# Options: three gaps in out-of-range index behaviour (pm-kernels TASK oob-options-note; fuel GAP-413, baracuda E4b)

Status: **options with a recommendation; nothing here is a clause.** 2026-11-27, against `origin/main` 8828b194 (conformance 0.24.3). Semantics only: **no stored byte moves** under any option (no codec field, token, corpus vector or schema version).

## 1. What KISS already states (not a gap)

Per op, `spec/ops.md`: `gather` takes an `oob_policy` in `{skip, clamp, zero-fill, reject}`, a negative index is always out of bounds, and `reject` ends in a typed decline naming the first OOB index in row-major order with no partial output as its result (6.11-0004, `:1049-1059`). A `skip` gather with no `base` operand hits a typed decline on an actual OOB read (6.11-0015, `:1178-1192`); `index_select` is `gather(oob=skip)` with no base (`:1486`, 6.13-0009 `:1569`), so its OOB is a typed decline. `embedding` is `gather(oob=zero-fill)` (`:1487`). `scatter` MUST skip an OOB write (6.11-0005, `:1060-1063`) and a skipped write retains `dest` (6.11-0016, `:1194-1200`); `scatter_add` is `scatter(atomic-add, oob=skip)` (`:1488`). A typed decline is "never a panic, abort, crash, hang, or out-of-bounds read" (`:466-471`; KISS-CONFORM-6.7-0004). Rule of thumb: **reads decline, writes skip.** Today clamping in `index_select` (Vulkan, Metal per fuel's measurement) and an unchecked `gather`/`scatter` (Vulkan) violate these clauses; CUDA `scatter_add` skip-and-Ok conforms.

## 2. Gap (a): `index_add` is absent

0 hits in `spec/`. A consumer that needs it today has no pinned OOB rule.
- **A. `index_add` := the `scatter_add` wrapper family** (`scatter(atomic-add, oob=skip)`, a 6.13-0009 structured op): an OOB position is skipped, `dest` retained. One rule for every write.
- **B. `index_add` rejects**: an OOB index is a typed decline. Would make it the only write op that declines, and needs gap (b)'s mechanism on a write kernel.
- **C. Stay absent**; express it as `scatter_add` with a broadcast index. Needs checking that `scatter`'s index operand shape (6.11-0005) admits the broadcast; not verified here.
- Torch's `index_add_` behaviour on an OOB index: **UNVERIFIED, not read** (no claim made).

## 3. Gap (b): a device that cannot decline at launch

A read-decline (6.11-0004 `reject`, 6.11-0015) is "a structured refusal returned in lieu of a result"; a GPU launch cannot return one synchronously, and no clause says how a decline reaches the caller.
- **B1. Sticky flag, surfaced at completion.** The kernel records an out-of-range flag (an atomic-min of the flat row-major index position also yields the "first OOB position" 6.11-0004 requires); the runtime turns it into the typed decline when the launch's stream or event completes, before any consumer may treat the buffer as a result.
- **B2. Pre-validation pass.** A bounds check (a reduction over the index operand) runs before the gather and declines at launch. One extra pass; no flag.
- **B3. Not offered.** A device that can do neither does not advertise the op (plan-time decline / reference decomposition, 6.14-0004).
- Clamp or unchecked access is not an option: it is a violation today, not a design.

## 4. Gap (c): the value at a declined lane

Once a decline is reported the buffer is not a result (6.11-0004: no partial output *as its result*). Options: **C1** the buffer content is unspecified and MUST NOT be read as a result, but the lane MUST NOT fault (no out-of-bounds read or write, 6.7-0004); **C2** pin a fill value (e.g. zero). C2 costs every device a pass for a value no conforming consumer may read.

## 5. Recommendation

(a) **A** (skip): it keeps the one rule reads-decline/writes-skip that the baracuda CUDA and Metal write paths already follow; B puts the only declining write on the least capable path. If torch's `index_add_` is later read and errors, that argues for B only if CireSnave wants torch parity over internal consistency. (b) **B1, with B2 and B3 both conformant**: the clause states the *outcome* (a typed decline is delivered before the result can be consumed, naming the first OOB position) and not the means, matching KISS's stated scope (outward I/O only). (c) **C1**: unspecified, never a fault.

## 6. Who it affects

fuel GAP-413 (the contract row and its measured four-way split) and baracuda E4b (CUDA `scatter_add` conforms; its gather path needs B1 or B2); vulkane, Metal and Vulkan `index_select` clamp (a violation of 6.11-0015 today) and Vulkan unchecked `gather`/`scatter` (violates 6.11-0004/-0005 and the no-OOB-access rule). Adopting this changes **no existing clause**: gap-filling only, and fuel's earlier "OOB = error everywhere" idea would instead have changed 6.11-0005 (not proposed here).

## 7. Open

Which options are a CireSnave decision via the PM. Torch `index_add_` and `index_select` OOB behaviour are unread; nothing here relies on them.
