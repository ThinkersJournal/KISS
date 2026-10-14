# SPDX-License-Identifier: MIT OR Apache-2.0
"""Controls for tools/kiss_version_sync.py: it must FAIL on drift and on an absent reading, and
PASS on agreement -- and the real tree's two files must be readable by the same parsers."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kiss_version_sync as v  # noqa: E402

MANIFEST = '[package]\nname = "kiss-conformance"\nversion = "0.2.24"\nrust-version = "1.77"\n\n[dependencies]\nfoo = { version = "9.9.9" }\n'
CHANGELOG = "# Changelog\n\nprose with 1.2.3 inside\n\n## 0.2.24 — 2026-10-14\n\n- x\n\n## 0.2.23 — 2026-10-14\n"


def test_agreement_passes():
    assert v.check(MANIFEST, CHANGELOG)[0]


def test_manifest_behind_changelog_fails():
    # the real incident: manifest 0.2.21, changelog 0.2.23
    ok, msg = v.check(MANIFEST.replace("0.2.24", "0.2.21"), CHANGELOG)
    assert not ok and "0.2.21" in msg and "0.2.24" in msg, msg


def test_changelog_behind_manifest_fails():
    assert not v.check(MANIFEST.replace("0.2.24", "0.2.25"), CHANGELOG)[0]


def test_absent_readings_fail_not_pass():
    assert not v.check("[dependencies]\nfoo = { version = \"1.0.0\" }\n", CHANGELOG)[0]
    assert not v.check(MANIFEST, "# Changelog\n\nno headings\n")[0]


def test_a_dependency_version_is_not_the_crate_version():
    # `[dependencies]` carries 9.9.9; the parser must take [package]'s
    assert v.manifest_version(MANIFEST) == "0.2.24"


def test_bracketed_keep_a_changelog_heading_parses():
    assert v.changelog_top("## [0.4.4] — 2026-10-14\n") == "0.4.4"


def test_the_real_tree_is_readable_and_in_sync():
    with open(os.path.join(v.ROOT, "conformance", "Cargo.toml"), encoding="utf-8") as f:
        m = f.read()
    with open(os.path.join(v.ROOT, "CHANGELOG.md"), encoding="utf-8") as f:
        c = f.read()
    assert v.manifest_version(m) is not None and v.changelog_top(c) is not None
    ok, msg = v.check(m, c)
    assert ok, msg


if __name__ == "__main__":
    tests = [(n, f) for n, f in sorted(globals().items()) if n.startswith("test_") and callable(f)]
    failed = 0
    for n, f in tests:
        try:
            f()
            print(f"  ok   {n}")
        except AssertionError as e:
            failed += 1
            print(f"  FAIL {n}: {e}")
    print(f"{len(tests) - failed}/{len(tests)} controls passed")
    sys.exit(1 if failed else 0)
