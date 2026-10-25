# Options note: R5.7, which open pins in KISS-Ops Appendix D should be decided now?

**Informative. No normative text. For CireSnave's decision (board 279, pair 3 of 5).**
Evidence read at `origin/main`: `spec/ops.md` Appendix D.11 (b)-(i), D.5, D.6, and the ROADMAP R5.7 row.
R1.23 covers the 10-bit mantissa floor only and is not part of this note.

## The question

Appendix D leaves a dozen small items "pending confirmation". They are not one decision; they fall into
four kinds, and each kind wants a different answer. **Which do we close now, and which wait for evidence?**

## The items, grouped

**1. Confirm the model as written (no spec change beyond deleting "confirm").**
D.11 (e) `matmul` carries no free OpAttrs; (f) `monoid` is an identity-bearing OpAttrs field, not folded
into op tokens; (h) `keepdim` and `stability` stay as explicit slots fixed to `1`; (i) `norm_axis` is a
single `u8`. Each is a statement of what the text already does. Reversing one later changes a per-op
field order, which D.11 itself calls ABI, so the cheap time to confirm is before any foreign reader
depends on the layout.

**2. Reserve or pin a value set.**
(b) `permutation` is frozen and reserved but attached to no op: confirm it stays reserved for future
shape/permute ops. (c) `im2col` `output_column_ordering` is provisionally `{1=channel-major,
2=tap-major}`: pin the set, which is a two-value enum.

**3. Needs a survey I have not done.**
(d) padding is one symmetric `u32` per axis; the open question is whether any op needs separate
low/high padding. I have not surveyed the op list for it, so I cannot say whether it is a yes or a no.

**4. Needs measurement.**
D.5: the ULP ceilings (4 for the elementary transcendentals, 8 for `lgamma`, correctly-rounded-or-2 for
`sqrt`) are "reference values pending confirmation against real per-target math libraries". Only a run
against those libraries answers it. D.6: an opt-in byte-exact float reduction profile (pinned
low-to-high order plus an accumulator dtype) is a feature, not a pin.

## Options

**A. Close groups 1 and 2 now; leave 3 to a survey; ceilings stay "reference" until measured; D.6 waits.**
Smallest irreversible step, and the part that is cheap to reverse now becomes costly later.

**B. Close everything that does not need a measurement, including a decision on (d).** Needs the survey
first (a read of the op list, no build).

**C. Leave all of Appendix D open until R4.** Nothing is spent, but the layout questions then get decided
under freeze pressure instead of now.

## Recommendation

**A**, with the (d) survey as a small follow-up I can do without a build. For D.5 I would not pin the
ceilings on the strength of reasoning alone: measure them against the per-target libraries first, then
pin what was seen. D.6 stays open until a consumer asks for it.

## What freezes, and what to decide

Nothing freezes on this note alone. Decisions needed: **A, B or C**; whether to authorise the (d) survey;
and whether the D.5 measurement is a task to schedule after the quiet window.
