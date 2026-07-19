#!/usr/bin/env python3
"""validate_corpus.py — the dev-time-only oracle-vector validation gate.

Reads a frozen bundle and independently re-derives every `provenance: "oracle"`
cell, asserting bit equality. Exact-byte cells are recomputed in Python here (the
3-source stack's trivial leg). The ULP/split transcendental legs — mpmath, MPFR
(gmpy2) driven by CORE-MATH hard cases, and Lefevre-Muller anchors — are declared
below and filled in Plan B; they are only reached for ULP/split cells, of which a
Plan A bundle has none. NEVER shipped; ordinary devs never run this.
"""
import json, struct, sys

def clean_bytes(hexstr):
    digits = "".join(c for c in hexstr if c in "0123456789abcdefABCDEF")
    assert len(digits) % 2 == 0, f"odd hex: {hexstr!r}"
    return bytes(int(digits[i:i+2], 16) for i in range(0, len(digits), 2))

def f32_of(b):  # big-endian bytes -> python float
    return struct.unpack(">f", b)[0]

def f32_bytes(x):
    return struct.pack(">f", x)

class TranscendentalLeg:
    """Plan B fills these; unreachable for a Plan A (exact-byte-only) bundle."""
    def value(self, op, inputs):
        raise NotImplementedError(f"transcendental leg for {op} lands in Plan B")

def recompute_exact_byte(op, input_byte_lists):
    if op == "add":
        a = f32_of(input_byte_lists[0]); b = f32_of(input_byte_lists[1])
        return f32_bytes(a + b)
    raise NotImplementedError(f"exact-byte recompute for {op} not defined")

def validate(path):
    data = json.loads(open(path, encoding="utf-8").read())
    fails = 0
    for cell in data["vectors"]:
        op = cell["op"]
        cls = cell["class"]
        inputs = [clean_bytes(i["bits"]) for i in cell["inputs"]]
        expected = clean_bytes(cell["expected"]["bits"])
        if cls == "exact-byte":
            got = recompute_exact_byte(op, inputs)
        else:
            got = TranscendentalLeg().value(op, inputs)  # Plan A: never taken
        if got != expected:
            fails += 1
            sys.stderr.write(
                f"MISMATCH tcId {cell['tcId']} {op}: expected {expected.hex()} got {got.hex()}\n")
    if fails:
        sys.stderr.write(f"{fails} cell(s) failed validation\n")
        return 1
    print(f"validated {len(data['vectors'])} cells: OK")
    return 0

if __name__ == "__main__":
    sys.exit(validate(sys.argv[1]))
