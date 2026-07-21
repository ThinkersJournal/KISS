#!/usr/bin/env python3
"""Dev-time generator for the Plan B Slice-0 `hp::consts` wide constant tables.

NOT SHIPPED with the crate (the conformance crate is stdlib-only). It mints the
`[u64; N]` big-endian literals for:

  * TWO_OVER_PI  [u64;36]  = floor((2/pi) * 2**2304)   (Payne-Hanek bit table)
  * LN2          [u64;18]  = floor(ln2   * 2**1152)     (significand, ebin=-1)
  * INV_LN2      [u64;18]  = floor((1/ln2)* 2**1151)    (significand, ebin=0)
  * PI_OVER_2    [u64;18]  = floor((pi/2) * 2**1151)    (significand, ebin=0)
  * SQRT2        [u64;18]  = floor(sqrt2  * 2**1151)    (significand, ebin=0)

Each constant is computed TWO mutually-independent ways and required to agree
bit-for-bit (the spec's 3-engine rule; MPFR/gmpy2 is not installed on this box,
so leg 3 is noted as pending in the provenance header):

  leg 1: mpmath at 3000-bit precision.
  leg 2: pure-Python big integers (Machin for pi, atanh for ln2), library-free
         — the true-independence leg, uses only `int`.

The published fdlibm ipio2 / libm high-words are asserted as pinned anchors so a
correct-but-mis-scaled table cannot pass. The SHIPPED crate re-verifies all of
this from scratch (a raw big-int Machin/atanh oracle) in tests/hp_consts.rs.
"""

import sys

# --- leg 2: pure-integer, library-free -------------------------------------


def atan_recip_int(x: int, q: int) -> int:
    """floor(atan(1/x) * 2**q) via the alternating arctan series (ints only)."""
    x2 = x * x
    pw = (1 << q) // x          # floor(2**q / x**1)
    acc = 0
    j = 0
    while pw:
        term = pw // (2 * j + 1)
        acc += term if j % 2 == 0 else -term
        pw //= x2               # floor(2**q / x**(2j+3))
        j += 1
    return acc


def atanh_recip_int(x: int, q: int) -> int:
    """floor(atanh(1/x) * 2**q) via the (all-positive) arctanh series."""
    x2 = x * x
    pw = (1 << q) // x
    acc = 0
    j = 0
    while pw:
        acc += pw // (2 * j + 1)
        pw //= x2
        j += 1
    return acc


def isqrt_int(n: int) -> int:
    return __import__("math").isqrt(n)


def pi_int(q: int) -> int:
    """floor(pi * 2**q) by Machin: 16 atan(1/5) - 4 atan(1/239)."""
    return 16 * atan_recip_int(5, q) - 4 * atan_recip_int(239, q)


def ln2_int(q: int) -> int:
    """floor(ln2 * 2**q) = 2 atanh(1/3)."""
    return 2 * atanh_recip_int(3, q)


def leg2_tables():
    Q = 3200  # generous internal precision, past every table LSB
    pi = pi_int(Q)
    ln2 = ln2_int(Q)
    # (2/pi)*2**2304 = 2**2305/pi = 2**(2305+Q)/pi_int  (pi_int = floor(pi*2**Q)).
    two_over_pi = (1 << (2305 + Q)) // pi          # floor((2/pi)*2**2304)
    ln2_sig = ln2 >> (Q - 1152)                    # floor(ln2*2**1152)
    inv_ln2 = (1 << (1151 + Q)) // ln2             # floor((1/ln2)*2**1151)
    pi_over_2 = pi >> (Q - 1150)                   # floor((pi/2)*2**1151)=floor(pi*2**1150)
    sqrt2 = isqrt_int(1 << 2303)                   # floor(sqrt2*2**1151)
    return {
        "TWO_OVER_PI": (two_over_pi, 2304, 36),
        "LN2": (ln2_sig, 1152, 18),
        "INV_LN2": (inv_ln2, 1152, 18),
        "PI_OVER_2": (pi_over_2, 1152, 18),
        "SQRT2": (sqrt2, 1152, 18),
    }


# --- leg 1: mpmath ----------------------------------------------------------


def leg1_tables():
    try:
        import mpmath as mp
    except ImportError:
        return None
    mp.mp.prec = 3200
    two_over_pi = int(mp.floor((mp.mpf(2) / mp.pi) * mp.mpf(2) ** 2304))
    ln2 = int(mp.floor(mp.log(2) * mp.mpf(2) ** 1152))
    inv_ln2 = int(mp.floor((1 / mp.log(2)) * mp.mpf(2) ** 1151))
    pi_over_2 = int(mp.floor((mp.pi / 2) * mp.mpf(2) ** 1151))
    sqrt2 = int(mp.floor(mp.sqrt(2) * mp.mpf(2) ** 1151))
    return {
        "TWO_OVER_PI": (two_over_pi, 2304, 36),
        "LN2": (ln2, 1152, 18),
        "INV_LN2": (inv_ln2, 1152, 18),
        "PI_OVER_2": (pi_over_2, 1152, 18),
        "SQRT2": (sqrt2, 1152, 18),
    }


# --- rendering + anchors ----------------------------------------------------


def to_words_msw(value: int, nwords: int) -> list:
    return [(value >> (64 * (nwords - 1 - i))) & 0xFFFF_FFFF_FFFF_FFFF for i in range(nwords)]


ANCHORS = {
    "TWO_OVER_PI": {0: 0xA2F9_836E_4E44_1529, 1: 0xFC27_57D1_F534_DDC0, 2: 0xDB62_9599_3C43_9041},
    "LN2": {0: 0xB172_17F7_D1CF_79AB, 1: 0xC9E3_B398_03F2_F6AF},
    # NOTE: the widely-published 64-bit log2(e) word is 0x…F0BC — but that is the
    # ROUND-to-nearest of the leading 64 bits. A full-width *floor*-truncated
    # significand (which the reduction error analysis §3 requires: stored ≤ true)
    # has word0 = 0x…F0BB (its 65th bit is 1, so round bumps the last bit). ln2 /
    # pi/2 / sqrt2 anchors are already floor; INV_LN2 is kept floor for
    # consistency and a one-sided error bound.
    "INV_LN2": {0: 0xB8AA_3B29_5C17_F0BB},
    "PI_OVER_2": {0: 0xC90F_DAA2_2168_C234, 1: 0xC4C6_628B_80DC_1CD1},
    "SQRT2": {0: 0xB504_F333_F9DE_6484},
}


def render(words: list, name: str, per_line: int = 3) -> str:
    lines = []
    for i in range(0, len(words), per_line):
        chunk = ", ".join(f"0x{w:016X}" for w in words[i : i + per_line])
        lines.append("    " + chunk + ",")
    return f"pub const {name}: [u64; {len(words)}] = [\n" + "\n".join(lines) + "\n];"


def main():
    leg1 = leg1_tables()
    leg2 = leg2_tables()

    if leg1 is None:
        print("// WARNING: mpmath not importable — only the pure-int leg ran.", file=sys.stderr)

    out = {}
    for name, (val2, bits, nwords) in leg2.items():
        assert val2.bit_length() == bits, f"{name}: leg2 width {val2.bit_length()} != {bits}"
        if leg1 is not None:
            val1 = leg1[name][0]
            assert val1 == val2, (
                f"{name}: mpmath and pure-int DISAGREE\n  mpmath={val1:x}\n  int   ={val2:x}"
            )
        words = to_words_msw(val2, nwords)
        for idx, want in ANCHORS[name].items():
            assert words[idx] == want, (
                f"{name}[{idx}] = 0x{words[idx]:016X} != published anchor 0x{want:016X}"
            )
        out[name] = words

    print("// engines: pure-int Machin/atanh (library-free) + "
          f"{'mpmath ' + __import__('mpmath').__version__ if leg1 else 'mpmath ABSENT'}")
    print("// all constants floor()-truncated; anchors verified against fdlibm/libm.\n")
    for name in ("TWO_OVER_PI", "LN2", "INV_LN2", "PI_OVER_2", "SQRT2"):
        print(render(out[name], name))
        print()

    # A couple of tail spot-words for the anchor test's lowest-limb checks.
    print("// tail spot-words (for anchors_lowest_limbs):")
    for name in ("TWO_OVER_PI", "LN2"):
        w = out[name]
        print(f"//   {name}[last]=0x{w[-1]:016X}  {name}[last-1]=0x{w[-2]:016X}")


if __name__ == "__main__":
    main()
