# R2.6: linalg factorization ops (SVD and kin): options note

Status: **informative, no clause, builds nothing.** ROADMAP R2.6 ("SVD / linalg ops: absent from the op vocabulary; needed
only if low-rank KV compression is wanted"). Unblocked 2026-11-20: CireSnave, via pm-kernels, *"we prioritize by consumer
demand...we don't avoid work because of a lack of it."* Ranked low. Read at `origin/main` dd51c2c.

## 1. What is and is not missing

- **Applying** a low-rank factorization is already expressible: `spec/ops.md` Appendix A.6 composes a low-rank adapter
  delta from existing ops (no new op). So low-rank KV *use* needs nothing new.
- **Computing** the factorization (SVD, and its kin QR, Cholesky, LU, symmetric eigen) is what no op names. A
  case-insensitive grep of `spec/ops.md` for svd, linalg, cholesky, eigen and qr returns 0 lines; control: the same
  command on `sort_network|scatter` returns 108.
- The only recorded driver is lightbulb's former `kv_compression.rs` (low-rank r=32/64 in `docs/KV_COMPRESSION_INTEGRATION.md`),
  which the 2026-09-24 audit says now lives in Fuel (`fuel-inference::kv_compress`, name-matched, not independently verified).
  **No consumer has asked for a factorization op.** Fuel answered (2026-11-20, ref c84ea4d8d): `kv_compress.rs`
  `LowRankCompressor` calls no factorization (a mean-centred 1-D demonstration projection whose doc comment says "a
  production implementation would use incremental SVD"); grep for svd, cholesky, qr, eigh, lstsq finds only that comment and two
  name strings (control: `kv_cache` matches 13 files). SVD is a stated future option, not a consumer.

## 2. Why this is not a routine op addition

1. **No unique answer.** Singular vectors are defined up to sign (and up to rotation for repeated singular values);
   eigenvectors likewise. Two correct kernels return different bytes, so a golden-vector comparison of U and V is meaningless.
   Singular values are unique and comparable.
2. **Iterative.** Results depend on the algorithm (Jacobi, Golub-Kahan, randomized), so bit-exactness across backends is
   unattainable and the determinism class of the Contract Guarantees section has nothing to say beyond "run-to-run".
3. **Several results.** An SVD returns three tensors. Ops precedent for a second result exists (the `sort_network` index
   output takes an operand slot, KISS-OPS-6.11-0019; the output takes one of `MAX_OPERANDS`), but U, S and V with ranks
   `k` and a thin/full choice stress it harder.
4. **Accuracy declaration is still open.** R1.36 designs the format for per-op, per-backend accuracy; a factorization's
   guarantee is a backward-error bound (`||A - U S V^T|| / ||A||`) plus orthogonality defect, neither of which is a ULP tier.

## 3. Options

| | Shape | Cost | Risk |
|---|---|---|---|
| A | **Do nothing**: leave factorizations to the provider (a library call outside the op vocabulary) | 0 | a consumer that needs one has no portable name |
| B | **One op `svd`** with `k` (rank), `full_matrices`, three outputs, a **postcondition contract**: singular values to a stated tolerance; U, V checked only by reconstruction and orthonormality, never bytewise | an Ops clause family, a new vector kind (property checks, not golden bytes), R1.36 dependency | sign/rotation ambiguity must be stated as the *contract*, or two honest backends diverge |
| C | **A family** (`svd`, `qr`, `cholesky`, `lu`, `eigh`) under one "factorization" clause group sharing the postcondition style | larger; Cholesky and LU have unique results up to pivoting rule, QR up to sign | scope creep with no consumer |
| D | **Randomized low-rank sketch only** (`rsvd`: range finder + small SVD) because that is what KV compression actually wants | smaller; still needs a seed and determinism class | ties the standard to one algorithm |

## 4. Recommendation

**A until a consumer asks; if one does, B** (one op, postcondition contract), because it is the smallest step that makes the
ambiguity explicit, and the vector kind it needs (property checks) is the same one R1.36 needs. Do **not** do C or D
speculatively. This is demand-gated and Fuel's answer is **none**, so **option A stands**; reopen only with a real low-rank KV row
naming the op, shape and tolerance.

## 5. If it is ruled B later: what has to be decided (not decided here)

Output order and naming; whether rank `k` is an attribute or a result shape; the tolerance as an attribute versus a
declared per-backend accuracy (R1.36); sign convention (none, or "first non-zero component of each U column positive" as an
optional canonicalization a backend MAY claim); the thin/full split; behavior on NaN or non-finite input. Any wire byte it
adds goes to CireSnave verbatim.

— lane: kiss
