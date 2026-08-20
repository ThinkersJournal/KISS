# SeamHello seed attestation — a reference that outlives the seeds

**Hand-maintained. Not generated, not freshness-gated.** It records a fact about two
external repositories at a moment, which no gate in this repo can re-derive.

## Why this file exists

`KISS-CONFORM-6.13-0002` requires proof that the two `SeamHello` reference seeds are
**byte-identical via golden hex**. KISS-Announce's own convergence task (informative)
says those two seeds **converge to one** canonical registry-published crate.

The obligation does not expire on convergence. **The reproducible reference does.** Once
there is one seed, nobody can re-run the two-seed comparison — and if the bytes were only
ever a live comparison, the evidence disappears with the second seed.

> **An expiring proof becomes a permanent reference for the cost of one file.**

Freezing the bytes and the provenance means a future implementor still has something to
reproduce **against**, whatever happens to the seeds. This is true under every disposition
of #260, so it does not wait on one.

## The frozen reference bytes

The 56-byte handshake envelope, Announce §2.5 worked example — version 1, one profile
`{1}`, capabilities `0x0000_0003_0000_003F`:

```
53 45 41 4D                          [0]  magic (u32 LE, "SEAM")
01                                   [4]  envelope_version = 1
00 00 00                             [5]  reserved0 (3)
01 00                                [8]  profiles_len = 1
01 00                                [10] profiles[0] = 1
00 00 00 00 00 00 00 00 00 00        [12] profiles[1..16] : 15 * u16 = 30 bytes, all zero
00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00                    [42] reserved1 (6)
3F 00 00 00 03 00 00 00              [48] capabilities (u64 LE)
```

**These bytes are gated in-repo**, independently of this file, by
`conformance/tests/announce_golden.rs::envelope_golden_bytes` under
`KISS-ANNOUNCE-6.1-0002`. This file adds the **attestation**, which that test cannot carry:
*who else produces these bytes, and from where*.

## The two seeds, and what is attested versus owed

| seed | repository path | seed last touched |
|---|---|---|
| `baracuda-seam` | `crates/baracuda-seam` (Baracuda) | `65297136310e44c19d7329ff67c9389bee9f0628`, 2026-08-19 |
| `fuel-kernel-seam-announce` | `fuel-kernel-seam-announce` (Fuel) | `1849bc9adf7c4199a83d9ff8d814360246fe424e`, 2026-07-14 |

Read from each repository's `HEAD` on 2026-08-20. Both crates exist, are tracked, and both
freeze the 56-byte layout in their own sources.

**ATTESTED HERE:** the reference bytes above; that both seed crates exist at the commits
named; and that each declares the frozen 56-byte layout.

**OWED, and deliberately not asserted:** that each seed's encoder *emits exactly these
bytes today*. **Establishing that requires building and running each crate, which is each
project's own conformance leg, not a claim KISS can make on their behalf.** A row is
complete only when its project states the output and the commit it was produced at.

Recording the gap is the point: **a partial attestation that reads as complete is worse
than none**, because the next reader stops looking.

## One provenance detail worth keeping

Fuel's seed was last corrected by:

> `fix(seam): SEAM_MAGIC serializes to "SEAM" not "MAES"; manage reserved1 padding`

**The two seeds were not always byte-identical.** One carried a magic-constant byte-order
defect and a padding defect, fixed 2026-07-14. The byte-identity this file exists to
preserve is an *achieved* property, not an incidental one — which is the strongest argument
for freezing the reference rather than trusting that two implementations will keep agreeing.

## The independence caveat

KISS-Announce §A.2 records the seeds as *"independently reproduced byte-identically in two
project workspaces"*, and names both as Evans Laboratories projects.

**Two workspaces is independence of LOCATION. The clause's stated property is independence
of DERIVATION.** Those are different claims, and this file does not resolve which one the
evidence supports — see #260, where it is under ruling. **The bytes are worth freezing
either way**; what the freeze does *not* do is settle what they are evidence of.
