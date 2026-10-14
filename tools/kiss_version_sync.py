# SPDX-License-Identifier: MIT OR Apache-2.0
r"""The crate manifest version must equal the newest CHANGELOG heading.

WHY: `conformance/Cargo.toml` sat at 0.2.21 while `CHANGELOG.md` reached 0.2.23 -- two PRs (#558,
#559) each wrote a CHANGELOG entry and neither bumped the manifest, and nothing failed. The version
the PM allocates at gate time lives in two places, and a number that can disagree with itself is not
a version a consumer can read.

Checks, on the tracked files only (never a disk walk):
  * `conformance/Cargo.toml` `[package] version`;
  * the FIRST `## X.Y.Z` heading of `CHANGELOG.md` (a heading such as `## 0.2.24 — date`).
They must be equal. A missing manifest version or a CHANGELOG with no such heading is a failure,
not a pass: an absent reading must not look like agreement.

Run:
    python tools/kiss_version_sync.py                      # CI gate
    python tools/kiss_version_sync.py --manifest M --changelog C   # explicit paths (the controls)
"""
import argparse
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

_VERSION_LINE = re.compile(r'^version\s*=\s*"([^"]+)"\s*(?:#.*)?$')
_HEADING = re.compile(r"^##\s+\[?v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)\]?(?:\s|$)")


def manifest_version(text):
    """The `[package]` table's `version`, or None. Stops at the next table so a dependency's
    `version = ` can never be mistaken for the crate's."""
    in_package = False
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("["):
            in_package = s == "[package]"
            continue
        if in_package:
            m = _VERSION_LINE.match(s)
            if m:
                return m.group(1)
    return None


def changelog_top(text):
    """The first `## X.Y.Z` heading's version, or None."""
    for line in text.splitlines():
        m = _HEADING.match(line)
        if m:
            return m.group(1)
    return None


def check(manifest_text, changelog_text):
    """(ok, message)."""
    mv = manifest_version(manifest_text)
    cv = changelog_top(changelog_text)
    if mv is None:
        return False, "no [package] version found in the manifest"
    if cv is None:
        return False, "no `## X.Y.Z` heading found in the CHANGELOG"
    if mv != cv:
        return False, f"manifest version {mv} != top CHANGELOG heading {cv}"
    return True, f"manifest and CHANGELOG agree at {mv}"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--manifest", default=os.path.join(ROOT, "conformance", "Cargo.toml"))
    ap.add_argument("--changelog", default=os.path.join(ROOT, "CHANGELOG.md"))
    a = ap.parse_args(argv)
    with open(a.manifest, encoding="utf-8") as f:
        m = f.read()
    with open(a.changelog, encoding="utf-8") as f:
        c = f.read()
    ok, msg = check(m, c)
    print(("OK: " if ok else "VERSION DRIFT: ") + msg)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
