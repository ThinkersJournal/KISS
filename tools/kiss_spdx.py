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

⚠️ A FILE COVERED BY A `REUSE.toml` ANNOTATION IS NOT STAMPED. That is how a verbatim third-party
file keeps its origin's terms without being edited: `conformance/cuda/generated/` is Baracuda's
generator output, committed verbatim and marked "do not edit", and stamping it would falsify its
PROVENANCE.md ("remains the generator's verbatim output").

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


def reuse_code_paths(root):
    """Path globs that REUSE.toml annotates as CODE — those files are exempt from stamping.

    Only annotations whose licence is NOT CC0 count: the CC0 annotations cover text, and a glob
    like `**` there must not silently exempt every code file from the in-file rule.
    """
    p = os.path.join(root, "REUSE.toml")
    if not os.path.exists(p):
        return []
    globs, cur_paths, cur_lic = [], [], None
    for line in io.open(p, encoding="utf-8"):
        s = line.strip()
        if s == "[[annotations]]":
            if cur_paths and cur_lic and "CC0" not in cur_lic:
                globs.extend(cur_paths)
            cur_paths, cur_lic = [], None
        elif s.startswith("path"):
            val = s.split("=", 1)[1].strip()
            cur_paths = [x.strip().strip('"') for x in val.strip("[]").split(",") if x.strip()]
        elif s.startswith("SPDX-License-Identifier"):
            cur_lic = s.split("=", 1)[1].strip().strip('"')
    if cur_paths and cur_lic and "CC0" not in cur_lic:
        globs.extend(cur_paths)
    return globs


def exempt(path, globs):
    return any(fnmatch.fnmatch(path, g) or fnmatch.fnmatch(path, g.replace("**/", "")) for g in globs)


def has_header(text):
    # the header must be in the first three lines: after a shebang, an encoding line, or @echo off
    return any(SPDX in l for l in text.splitlines()[:3])


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
    globs = reuse_code_paths(root)
    code = [f for f in files if code_class(f)]
    missing, exempted, stamped = [], [], []
    for f in code:
        if exempt(f, globs):
            exempted.append(f)
            continue
        full = os.path.join(root, f)
        with open(full, "rb") as fh:
            raw = fh.read()
        text = raw.decode("utf-8")
        if has_header(text):
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
    print(f"  exempt (REUSE.toml)  {len(exempted)}   {exempted if exempted else ''}")
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
