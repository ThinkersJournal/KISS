#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""kiss_kernel_refs.py: the shipped spec is clean and each way the references can rot is caught (mutation tests)."""
import os
import shutil
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kiss_kernel_refs as KR  # noqa: E402
import kiss_ops as K  # noqa: E402

SPEC = os.path.join(os.path.dirname(HERE), "spec")
OPS = set(K.build_manifest(SPEC)["all_ops"])


def mutated(name, old, new):
    d = tempfile.mkdtemp()
    for f in ("ops.md", "kernel.md"):
        shutil.copy(os.path.join(SPEC, f), os.path.join(d, f))
    p = os.path.join(d, name)
    text = open(p, encoding="utf-8", newline="").read().replace("\r\n", "\n")
    assert text.count(old) == 1, f"mutation anchor must hit exactly once, hit {text.count(old)}"
    open(p, "w", encoding="utf-8", newline="").write(text.replace(old, new))
    return d


class KernelRefs(unittest.TestCase):
    def test_shipped_spec_is_clean(self):
        self.assertEqual(KR.check(SPEC, OPS), [])
        self.assertEqual(sorted(KR.kernel_governed(SPEC)), ["scatter", "scatter_add"])

    def test_a_governed_op_with_no_pointer_fails(self):
        d = mutated("ops.md", "**NaN-propagating** (*Kernel-governed:* KISS-KERNEL-6.4, KISS-KERNEL-6.5; §6.22),", "**NaN-propagating**,")
        self.assertTrue(any("no `Kernel-governed:` pointer" in p for p in KR.check(d, OPS)))

    def test_a_pointer_that_omits_a_section_fails(self):
        d = mutated("ops.md", "*Kernel-governed:* KISS-KERNEL-6.4, KISS-KERNEL-6.5 (the order", "*Kernel-governed:* KISS-KERNEL-6.4 (the order")
        self.assertTrue(any("omits ['6.5']" in p for p in KR.check(d, OPS)))

    def test_a_pointer_with_no_table_row_fails(self):
        d = mutated("ops.md", "| `scatter` | 6.4, 6.5 | KISS-OPS-6.11-0006, KISS-OPS-6.11-0010, KISS-OPS-6.11-0024 |",
                    "| `scatter` | 6.4, 6.5 | KISS-OPS-6.11-0006, KISS-OPS-6.11-0024 |")
        self.assertTrue(any("no §6.22 row lists it" in p for p in KR.check(d, OPS)))

    def test_a_section_kernel_md_does_not_list_fails(self):
        d = mutated("kernel.md", "| 6.5 | the race rule", "| 6.9 | the race rule")
        self.assertTrue(any("does not list" in p for p in KR.check(d, OPS)))

    def test_a_table_op_that_is_not_an_op_fails(self):
        self.assertTrue(any("is not a KISS-Ops op" in p for p in KR.check(SPEC, OPS - {"scatter_add"})))


if __name__ == "__main__":
    unittest.main()
