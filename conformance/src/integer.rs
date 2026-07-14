//! From-scratch CPU oracle for the pinned **integer** primitive semantics
//! (KISS-Ops §6.4 arithmetic atoms on the integer path, and §6.10 bitwise atoms).
//!
//! This is the integer companion to [`crate::semantics`]: a Conform §6.5
//! differential oracle whose functions are each the op's reference behavior
//! *transcribed directly* (Rust's own `wrapping_*` / `>>` / `count_ones` etc.),
//! sharing no lowering code with any generator. Its purpose is to make the
//! load-bearing integer distinctions the spec pins in prose — two's-complement
//! **wrapping** at the boundary (`neg(INT_MIN)=INT_MIN`, §6.4-0005), the
//! **arithmetic-vs-logical** `shr` split keyed on signedness (§6.10-0003), and
//! the out-of-range shift being **target-defined, not pinned** (§6.10-0004) —
//! executable, so an implementation under test can be differenced against them.
//!
//! Dtype coverage is the §6.16-0006 / §6.2-0002 **ordinary** integer set:
//! `s8`→[`i8`], `u8`→[`u8`], `i32`→[`i32`], `u32`→[`u32`], `i64`→[`i64`]. The
//! pinned set has no `u64` and no 16-bit token; `i32` is the signed-32 token.
//! Sub-byte `s4`/`u4` (packing owned by the data-vocabulary sub-standard,
//! §6.16-0006) and `b1` binary-GEMM (a §6.16-0006 reduction, not an atom) are
//! deliberately out of this slice.
//!
//! `div`/`rem` are **excluded** from the integer op set (§6.4-0002): division is
//! float-only and integer division-by-zero is target-defined. There is therefore
//! deliberately **no** integer division oracle here — the exclusion is asserted in
//! the tests as a load-bearing NON-op, not implemented.
//!
//! §6.10-0001 requires the bitwise atoms to reject a float/complex/bool operand
//! with a typed decline; in this scalar oracle that rejection is enforced *by the
//! type system* — every function is generic only over [`IntDtype`], so a
//! non-integer operand does not typecheck and there is no runtime decline to
//! model here (that is a dtype-tagged-frontend / Conform §6.7 concern).

use crate::differential::SplitMix64;

/// The bit-pattern operations KISS-Ops §6.4 / §6.10 pin over an integer dtype,
/// transcribed one-for-one onto a concrete Rust primitive. Implemented for the
/// pinned ordinary integer set only (`i8`/`u8`/`i32`/`u32`/`i64`); there is no
/// public constructor beyond these impls, so the free functions below are total
/// over exactly the pinned dtypes and reject everything else at compile time
/// (§6.10-0001, enforced structurally).
///
/// The signed-vs-unsigned `shr` split (§6.10-0003) and the signed-vs-unsigned
/// `abs` behavior (§6.4-0005 vs the unsigned no-op) are realized by each type's
/// *own* trait method — Rust's `>>` is arithmetic on signed and logical on
/// unsigned, and `wrapping_abs` exists only on signed types — so the semantics is
/// transcribed from the language, never re-derived.
pub trait IntDtype: Copy + Eq + core::fmt::Debug {
    /// The dtype bit width (`s8`→8 … `i64`→64). The pinned shift domain is
    /// `[0, BITS)` (§6.10-0004) and the zero edge of `clz`/`ctz` is `BITS`.
    const BITS: u32;

    /// The KISS-Ops §6.16-0006 dtype token for this Rust type (for stable
    /// diagnostics): `"s8"`, `"u8"`, `"i32"`, `"u32"`, `"i64"`.
    fn dtype_token() -> &'static str;

    /// Whether this dtype is signed (selects arithmetic vs logical `shr`, and the
    /// wrapping-`abs` vs identity-`abs` path). §6.10-0003 / §6.4-0005.
    fn is_signed() -> bool;

    /// Draw a value from the low `BITS` bits of a splitmix draw (reproducible
    /// corpus construction; the high bits are discarded by the narrowing cast).
    fn from_seed(bits: u64) -> Self;

    /// The value's raw bit pattern in `BITS` bits, zero-extended to `u64`, for
    /// stable printing of a divergence.
    fn to_bits_u64(self) -> u64;

    // ---- §6.4 arithmetic atoms, integer path (wrapping, §6.2-0002) ----------
    fn add(self, rhs: Self) -> Self;
    fn sub(self, rhs: Self) -> Self;
    fn mul(self, rhs: Self) -> Self;
    fn neg(self) -> Self;
    /// Two's-complement wrapping absolute value on signed dtypes (`abs(INT_MIN)=
    /// INT_MIN`, §6.4-0005); the identity on unsigned dtypes (`|x| = x`, provably
    /// a no-op — there is no `wrapping_abs` on unsigned types).
    fn abs(self) -> Self;

    // ---- §6.10-0002 bitwise logic -------------------------------------------
    fn bit_and(self, rhs: Self) -> Self;
    fn bit_or(self, rhs: Self) -> Self;
    fn bit_xor(self, rhs: Self) -> Self;
    fn bit_not(self) -> Self;

    // ---- §6.10-0003/0004 shifts (checked into the pinned domain) ------------
    /// Logical left shift by `amt` (same for signed and unsigned, §6.10-0003),
    /// defined only for `amt` in `[0, BITS)`; `None` for any out-of-range `amt`.
    fn checked_shl_n(self, amt: u32) -> Option<Self>;
    /// Right shift by `amt`: **arithmetic** (sign-replicating) on signed dtypes,
    /// **logical** (zero-filling) on unsigned dtypes (§6.10-0003), realized by
    /// the concrete type's `>>`. Defined only for `amt` in `[0, BITS)`.
    fn checked_shr_n(self, amt: u32) -> Option<Self>;

    // ---- §6.10-0005 bit-count atoms -----------------------------------------
    fn popcount(self) -> u32;
    fn clz(self) -> u32;
    fn ctz(self) -> u32;
}

// One macro arm per signedness. `$uty` is the same-width *unsigned* sibling used
// by `to_bits_u64` to reinterpret (not sign-extend) the two's-complement pattern.
// The signed arm's `abs` is `wrapping_abs` (pins `abs(INT_MIN)=INT_MIN`,
// §6.4-0005); the unsigned arm's `abs` is the identity (`|x|=x`; no `wrapping_abs`
// method exists on unsigned types). Everything else is identical across arms.
macro_rules! impl_int_common {
    ($ty:ty, $uty:ty, $token:literal, $signed:expr) => {
        const BITS: u32 = <$ty>::BITS;
        fn dtype_token() -> &'static str { $token }
        fn is_signed() -> bool { $signed }
        fn from_seed(bits: u64) -> Self { bits as $ty }
        fn to_bits_u64(self) -> u64 { (self as $uty) as u64 }

        fn add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }
        fn sub(self, rhs: Self) -> Self { self.wrapping_sub(rhs) }
        fn mul(self, rhs: Self) -> Self { self.wrapping_mul(rhs) }
        fn neg(self) -> Self { self.wrapping_neg() }

        fn bit_and(self, rhs: Self) -> Self { self & rhs }
        fn bit_or(self, rhs: Self) -> Self { self | rhs }
        fn bit_xor(self, rhs: Self) -> Self { self ^ rhs }
        fn bit_not(self) -> Self { !self }

        fn checked_shl_n(self, amt: u32) -> Option<Self> { self.checked_shl(amt) }
        fn checked_shr_n(self, amt: u32) -> Option<Self> { self.checked_shr(amt) }

        fn popcount(self) -> u32 { self.count_ones() }
        fn clz(self) -> u32 { self.leading_zeros() }
        fn ctz(self) -> u32 { self.trailing_zeros() }
    };
}

macro_rules! impl_signed_int {
    ($ty:ty, $uty:ty, $token:literal) => {
        impl IntDtype for $ty {
            impl_int_common!($ty, $uty, $token, true);
            fn abs(self) -> Self { self.wrapping_abs() }
        }
    };
}

macro_rules! impl_unsigned_int {
    ($ty:ty, $token:literal) => {
        impl IntDtype for $ty {
            impl_int_common!($ty, $ty, $token, false);
            fn abs(self) -> Self { self } // |x| = x for unsigned; provable no-op
        }
    };
}

impl_signed_int!(i8, u8, "s8");
impl_unsigned_int!(u8, "u8");
impl_signed_int!(i32, u32, "i32");
impl_unsigned_int!(u32, "u32");
impl_signed_int!(i64, u64, "i64");

// ---- §6.4 arithmetic atoms (integer path) -----------------------------------
//
// Each is ONE generic free function delegating to the trait method, so the same
// transcription covers every pinned width.

/// `add` (integer path) — wrapping two's-complement `a+b` modulo `2^BITS`
/// (§6.4-0001, §6.2-0002). Signed `MAX+1` wraps to `MIN`; unsigned `MAX+1` wraps
/// to `0`; never UB, never saturating.
pub fn add<T: IntDtype>(a: T, b: T) -> T { a.add(b) }

/// `sub` (integer path) — wrapping two's-complement `a-b` modulo `2^BITS`
/// (§6.4-0001). Unsigned `0-1` = unsigned `MAX`; signed `MIN-1` = `MAX`.
pub fn sub<T: IntDtype>(a: T, b: T) -> T { a.sub(b) }

/// `mul` (integer path) — wrapping two's-complement `a*b` modulo `2^BITS`, high
/// bits discarded (§6.4-0001). Signed `MIN*-1` = `MIN` (wraps, not saturates).
pub fn mul<T: IntDtype>(a: T, b: T) -> T { a.mul(b) }

/// `neg` (integer path) — two's-complement wrapping negate `-x` (§6.4-0003,
/// §6.4-0005). `neg(INT_MIN)=INT_MIN` for every signed width (the pinned wrap);
/// unsigned `neg(0)=0`, `neg(1)=` unsigned `MAX`. Distinct from the float
/// sign-bit-flip `neg` in [`crate::semantics::neg`].
pub fn neg<T: IntDtype>(a: T) -> T { a.neg() }

/// `abs` (integer path) — two's-complement wrapping absolute value (§6.4-0004,
/// §6.4-0005). `abs(INT_MIN)=INT_MIN` for every signed width (the pinned wrap);
/// unsigned `abs(x)=x`. Distinct from the float clear-sign-bit
/// [`crate::semantics::abs`].
pub fn abs<T: IntDtype>(a: T) -> T { a.abs() }

// NOTE — integer `div`/`rem` are intentionally NOT provided. §6.4-0002 excludes
// integer division and remainder from the op set (division is float-only), and
// integer division-by-zero is target-defined (target-UB), not pinned by KISS-Ops.
// A later implementer must not add an integer-div oracle here; the exclusion is a
// load-bearing NON-op asserted by `test_ops_div_float_only` in the test crate.

// ---- §6.10-0002 bitwise logic -----------------------------------------------

/// `bit_and` — `a & b` over the integer bit pattern (§6.10-0002).
pub fn bit_and<T: IntDtype>(a: T, b: T) -> T { a.bit_and(b) }
/// `bit_or` — `a | b` over the integer bit pattern (§6.10-0002).
pub fn bit_or<T: IntDtype>(a: T, b: T) -> T { a.bit_or(b) }
/// `bit_xor` — `a ^ b` over the integer bit pattern (§6.10-0002). The basis of
/// the deferred `b1` xor+popcount binary-GEMM (§6.16-0006).
pub fn bit_xor<T: IntDtype>(a: T, b: T) -> T { a.bit_xor(b) }
/// `bit_not` — `~x` (ones-complement of the bit pattern) (§6.10-0002). Purely
/// bitwise: signed `~x = -x-1`, `~0 =` all-ones (unsigned `MAX` / signed `-1`).
pub fn bit_not<T: IntDtype>(a: T) -> T { a.bit_not() }

// ---- §6.10-0003/0004 shifts -------------------------------------------------

/// `shl` — logical left shift `a<<amt` (same for signed and unsigned, §6.10-0003).
///
/// Returns `Some(a<<amt)` for `amt` in the pinned domain `[0, BITS)`, and `None`
/// for any `amt < 0` or `amt >= BITS`. The `None` is the harness's explicit,
/// never-panic representation of "**target-defined, NOT pinned** by KISS-Ops"
/// (§6.10-0004): the oracle declines to pin an out-of-range shift, and the
/// differential loop skips it — rather than triggering a Rust shift overflow
/// panic. Bits shifted past the top (including through the sign bit) are dropped.
pub fn shl<T: IntDtype>(a: T, amt: i64) -> Option<T> {
    let n = in_domain::<T>(amt)?;
    a.checked_shl_n(n)
}

/// `shr` — **arithmetic** (sign-replicating) right shift on signed dtypes and
/// **logical** (zero-filling) right shift on unsigned dtypes (§6.10-0003) — the
/// load-bearing signed/unsigned split, realized by the concrete type's `>>`.
///
/// Returns `Some(a>>amt)` for `amt` in `[0, BITS)`; `None` for out-of-range
/// `amt` (target-defined, §6.10-0004; see [`shl`]). This is the single most
/// bug-prone atom — an implementation that always logical-shifts is wrong on
/// negative signed inputs (`shr(-1, k)` must stay `-1`, not become a large value).
pub fn shr<T: IntDtype>(a: T, amt: i64) -> Option<T> {
    let n = in_domain::<T>(amt)?;
    a.checked_shr_n(n)
}

/// Map a signed shift amount to the pinned domain `[0, BITS)`: `Some(amt as u32)`
/// when in range, `None` (target-defined, §6.10-0004) when `amt < 0` or
/// `amt >= BITS`. Collapsing negative and too-large into one `None` is deliberate
/// — §6.10-0004 makes both the single "not pinned" case, not two distinct pins.
fn in_domain<T: IntDtype>(amt: i64) -> Option<u32> {
    if amt < 0 || amt >= T::BITS as i64 {
        None
    } else {
        Some(amt as u32)
    }
}

// ---- §6.10-0005 bit-count atoms ---------------------------------------------

/// `popcount` — set-bit count over the integer bit pattern, `-> u32` (§6.10-0005).
/// `popcount(0)=0`; `popcount(all-ones)=BITS`; a single power of two → `1`.
pub fn popcount<T: IntDtype>(a: T) -> u32 { a.popcount() }

/// `clz` — count leading (most-significant) zeros over the integer bit pattern,
/// `-> u32` (§6.10-0005). `clz(0)=BITS` is the pinned bit-pattern answer even
/// though many hardware CLZ leave the all-zero input undefined — the spec's
/// "over the integer bit pattern" definition forces the width. `clz(sign-bit-
/// set)=0`; `clz(1)=BITS-1`.
pub fn clz<T: IntDtype>(a: T) -> u32 { a.clz() }

/// `ctz` — count trailing (least-significant) zeros over the integer bit pattern,
/// `-> u32` (§6.10-0005). `ctz(0)=BITS` (same width-forcing pin and hardware-
/// undefined-on-0 caveat as [`clz`]); `ctz(1)=0`; `ctz(high-bit-only)=BITS-1`.
pub fn ctz<T: IntDtype>(a: T) -> u32 { a.ctz() }

// ---- integer differential utilities (Conform §6.5) --------------------------
//
// Kept in THIS module so `differential.rs` is not edited. Integers have no NaN
// class, so `agree_int` is plain equality and every corpus value is comparable.

/// The edge-case values every integer corpus includes: the boundaries and the
/// bit patterns where a sloppy implementation diverges — `0`, `1`, `MAX`, `MIN`,
/// `MIN+1`, `MAX-1`, all-ones, the sign-bit-only value, the `0x55…`/`0xAA…`
/// alternating patterns, each single power of two, and (signed only) `-1`.
pub fn int_edge_vec<T: IntDtype>() -> Vec<T> {
    let mut v = Vec::new();
    let zero = T::from_seed(0);
    let one = T::from_seed(1);
    let all_ones = zero.bit_not(); // ~0: unsigned MAX / signed -1
    // MIN / MAX from the sign bit. MIN = sign-bit-only for signed; = 0 for
    // unsigned. MAX = ~MIN. We build the sign-bit-only pattern directly.
    let sign_bit = one.checked_shl_n(T::BITS - 1).unwrap(); // 1 << (BITS-1)
    let (min, max) = if T::is_signed() {
        (sign_bit, sign_bit.bit_not()) // MIN=100..0, MAX=011..1
    } else {
        (zero, all_ones) // unsigned MIN=0, MAX=all-ones
    };

    v.push(zero);
    v.push(one);
    v.push(max);
    v.push(min);
    v.push(min.add(one)); // MIN+1
    v.push(max.sub(one)); // MAX-1
    v.push(all_ones);
    v.push(sign_bit); // high-bit-only pattern
    v.push(T::from_seed(0x5555_5555_5555_5555)); // 0x55… alternating
    v.push(T::from_seed(0xAAAA_AAAA_AAAA_AAAA)); // 0xAA… alternating
    // every single power of two representable in BITS
    let mut k = 0;
    while k < T::BITS {
        v.push(one.checked_shl_n(k).unwrap());
        k += 1;
    }
    if T::is_signed() {
        v.push(one.neg()); // -1 (distinct emphasis even though == all-ones)
    }
    v
}

/// A reproducible integer corpus: the edges, then `n` splitmix-seeded draws
/// narrowed to `BITS` bits. Same `seed`/`n` → the identical corpus (the
/// fuzzer-reproducibility guarantee, Conform §6.5), reusing the crate's
/// [`SplitMix64`] so `differential.rs` need not change.
pub fn int_corpus<T: IntDtype>(seed: u64, n: usize) -> Vec<T> {
    let mut rng = SplitMix64::new(seed);
    let mut v = int_edge_vec::<T>();
    for _ in 0..n {
        v.push(T::from_seed(rng.next_u64()));
    }
    v
}

/// Two integer results **agree** iff they are equal — there is no NaN class for
/// integers, so this is plain `==` (contrast [`crate::differential::agree`],
/// which relaxes for NaN). The right comparator for differencing integer op
/// semantics.
pub fn agree_int<T: IntDtype>(a: T, b: T) -> bool { a == b }

/// One divergence found by the integer differential runner, printed via the
/// stable `to_bits_u64` pattern of each operand.
#[derive(Debug, Clone, Copy)]
pub struct IntDivergence<T: IntDtype> {
    pub a: T,
    pub b: T,
    pub oracle: T,
    pub candidate: T,
}

/// Difference a **unary** integer candidate against the oracle over the corpus;
/// return every divergence (empty ⇒ they agree on every input). The `b` field of
/// each record is the same as `a` (unused second operand).
pub fn diff_int_unary<T: IntDtype>(
    oracle: impl Fn(T) -> T,
    candidate: impl Fn(T) -> T,
    corpus: &[T],
) -> Vec<IntDivergence<T>> {
    let mut out = Vec::new();
    for &a in corpus {
        let (o, c) = (oracle(a), candidate(a));
        if !agree_int(o, c) {
            out.push(IntDivergence { a, b: a, oracle: o, candidate: c });
        }
    }
    out
}

/// Difference a **binary** integer candidate against the oracle over all ordered
/// pairs of the corpus; return every divergence (empty ⇒ the candidate matches
/// the oracle on every pair). The §6.5 "catches a wrong impl" engine for integers.
pub fn diff_int_binary<T: IntDtype>(
    oracle: impl Fn(T, T) -> T,
    candidate: impl Fn(T, T) -> T,
    corpus: &[T],
) -> Vec<IntDivergence<T>> {
    let mut out = Vec::new();
    for &a in corpus {
        for &b in corpus {
            let (o, c) = (oracle(a, b), candidate(a, b));
            if !agree_int(o, c) {
                out.push(IntDivergence { a, b, oracle: o, candidate: c });
            }
        }
    }
    out
}
