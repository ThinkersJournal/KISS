# SPDX-License-Identifier: MIT OR Apache-2.0
r"""KISS licence split: every CODE file carries an SPDX header; TEXT and DATA are CC0 via REUSE.toml.

CireSnave (quoted verbatim, CIRESNAVE-EXPECTATIONS.md §6.4c): "KISS should be CC0 for the
standard's TEXT (normal and correct for a standard; it is how you get universal adoption) and
MIT/Apache for the CODE (normal for Rust)."

    CODE  = MIT OR Apache-2.0   .rs .c .cu .py .sh .bat .toml .yml   (in-file SPDX header)
    TEXT  = CC0-1.0              everything else                       (REUSE.toml, by path)

⚠️ WHY TEXT IS NEVER STAMPED IN-FILE. An added top line shifts every line number in a spec, and
spec line numbers are cited across issues and other repos. The corpus JSON is sha256-pinned and
cannot carry a comment at all. So text is annotated by PATH in REUSE.toml, which touches no byte of
the files it covers.

⚠️ ENUMERATION IS `git ls-files`, NEVER A DISK WALK. A disk walk in a shared checkout sees other
lanes' worktrees, and a WRITER that walks the disk edits them (portfolio CLAUDE.md §5b). The index
cannot see another worktree by construction — a property, not an exclusion list.

⚠️ ONLY A `precedence = "override"` ANNOTATION EXEMPTS A CODE FILE. That is how a verbatim
third-party file keeps its origin's terms without being edited: `conformance/cuda/generated/` is
Baracuda's generator output, marked "do not edit", and stamping it would falsify its PROVENANCE.md.
An `aggregate` annotation does NOT exempt: it is the normal way to add copyright text over a glob such
as `**/*.rs`, and if it exempted, one such entry would silently switch the in-file rule off for every
Rust file.

⚠️ ANY `SPDX-License-Identifier` COUNTS AS PRESENT. EXPECTATIONS §6.4a requires a vendored file to
carry its ORIGIN's identifier, "never the project's default". Such a file is reported as FOREIGN
(printed, not failed), and `--stamp` never touches a file that already has an identifier. Otherwise
the gate would report it missing and the stamper would add a second, conflicting licence.

Run:
    python tools/kiss_spdx.py --check     # CI gate: exit 1 if any code file lacks its header
    python tools/kiss_spdx.py --stamp     # add missing headers (idempotent)
"""
import argparse
import fnmatch
import io
import os
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

SPDX = "SPDX-License-Identifier: MIT OR Apache-2.0"

# extension -> line-comment prefix. ⚠️ This set IS the CODE class; anything else is TEXT/DATA.
CODE = {
    ".rs": "//", ".c": "//", ".cu": "//",
    ".py": "#", ".sh": "#", ".toml": "#", ".yml": "#",
    ".bat": "REM",
}


def code_class(path):
    return CODE.get(os.path.splitext(path)[1].lower())


def git_ls_files(root):
    out = subprocess.run(["git", "ls-files", "-z"], cwd=root, capture_output=True, check=True)
    return [p for p in out.stdout.decode("utf-8").split("\0") if p]


def reuse_override_paths(root):
    """Path globs from REUSE.toml annotations with `precedence = "override"` — the only exempt ones.

    ⚠️ NOT "any non-CC0 annotation". An `aggregate` annotation over `**/*.rs` is the normal way to add
    copyright text, and treating it as an exemption would switch the in-file rule off for every
    Rust file while the gate stayed green.
    """
    p = os.path.join(root, "REUSE.toml")
    if not os.path.exists(p):
        return []
    globs, cur_paths, cur_prec = [], [], None

    def flush():
        if cur_paths and cur_prec == "override":
            globs.extend(cur_paths)

    for line in io.open(p, encoding="utf-8"):
        t = line.strip()
        if t == "[[annotations]]":
            flush()
            cur_paths, cur_prec = [], None
        elif t.startswith("path"):
            val = t.split("=", 1)[1].strip()
            cur_paths = [x.strip().strip('"') for x in val.strip("[]").split(",") if x.strip()]
        elif t.startswith("precedence"):
            cur_prec = t.split("=", 1)[1].strip().strip('"')
    flush()
    return globs


def exempt(path, globs):
    return any(fnmatch.fnmatch(path, g) or fnmatch.fnmatch(path, g.replace("**/", "")) for g in globs)


def header_id(text):
    """The SPDX identifier in the first three lines, or None.

    ANY identifier counts: a vendored file carries its origin's (§6.4a), and treating that as
    missing would make the stamper add a second, conflicting licence.
    """
    for line in text.splitlines()[:3]:
        k = line.find("SPDX-License-Identifier:")
        if k >= 0:
            return line[k + len("SPDX-License-Identifier:"):].strip()
    return None


def stamp_text(text, prefix):
    nl = "\r\n" if "\r\n" in text else "\n"
    lines = text.split(nl)
    at = 0
    # a SHEBANG is `#!/`; `#![...]` is a Rust INNER ATTRIBUTE and a // comment may precede it
    if lines and lines[0].startswith("#!") and not lines[0].startswith("#!["):
        at = 1
    if len(lines) > at and ("coding:" in lines[at] or "coding=" in lines[at]) and lines[at].startswith("#"):
        at += 1
    if prefix == "REM" and lines and lines[0].strip().lower() == "@echo off":
        at = 1  # a REM before `@echo off` would itself be echoed
    lines.insert(at, f"{prefix} {SPDX}")
    return nl.join(lines)


def run(root, stamp):
    files = git_ls_files(root)
    globs = reuse_override_paths(root)
    code = [f for f in files if code_class(f)]
    missing, exempted, stamped, foreign = [], [], [], []
    for f in code:
        if exempt(f, globs):
            exempted.append(f)
            continue
        full = os.path.join(root, f)
        with open(full, "rb") as fh:
            raw = fh.read()
        text = raw.decode("utf-8")
        ident = header_id(text)
        if ident is not None:
            if ident != SPDX.split(":", 1)[1].strip():
                foreign.append((f, ident))   # reported, never failed, never stamped over
            continue
        if stamp:
            with open(full, "wb") as fh:
                fh.write(stamp_text(text, code_class(f)).encode("utf-8"))
            stamped.append(f)
        else:
            missing.append(f)

    print("-" * 72)
    print("  KISS licence split (#508) — CODE = MIT OR Apache-2.0, in-file SPDX")
    # ⚠️ THE ENUMERATION AND POPULATION ARE PART OF THE RESULT. "0 missing" over 0 files is not a
    # pass, and a narrowed enumeration reports a clean run over less.
    print(f"  enumeration          git ls-files ({len(files)} tracked)")
    print(f"  code files           {len(code)}   by class: " + ", ".join(
        f"{e}={sum(1 for x in code if x.lower().endswith(e))}" for e in sorted(CODE)))
    print(f"  exempt (override)    {len(exempted)}   {exempted if exempted else ''}")
    print(f"  foreign identifier   {len(foreign)}   {foreign if foreign else ''}")
    if stamp:
        print(f"  stamped now          {len(stamped)}")
    else:
        print(f"  missing header       {len(missing)}")
    print("-" * 72)

    if not code:
        print("  COULD NOT MEASURE: zero code files enumerated — broken enumeration, not a clean tree.")
        return 2
    if missing:
        for f in missing[:40]:
            print(f"  missing: {f}")
        print(f"  Add `{SPDX}` (python tools/kiss_spdx.py --stamp).")
        return 1
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--check", action="store_true")
    g.add_argument("--stamp", action="store_true")
    ap.add_argument("--root", default=ROOT, help=argparse.SUPPRESS)
    a = ap.parse_args()
    return run(a.root, a.stamp)


if __name__ == "__main__":
    raise SystemExit(main())
