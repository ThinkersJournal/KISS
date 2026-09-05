"""Find prose citations ORPHANED by a diff that removed the wording they quote (#407).

This repo's discipline is citation — `Backs:`, `*Test:*`, clause ids, `Enforces` — and all
of those have tooling. But the reasoning travels in PROSE QUOTATIONS of clause and
docstring text, and those have none: a citation is joined to its target only by a string a
human chose. When the target changes the citation does not move, does not error and does
not go red. It becomes false while reading exactly as it did when it was true.

Four instances in one day (#407), every one caught by its author rather than by anything
in the repo. The sharpest: a function docstring listed an invariant among those "asserted
by the born-red control" for a property no control can check.

WHAT IT DOES. For each line a diff REMOVES, extract the quoted phrases; for each phrase
long enough to be distinctive, search the working tree for surviving occurrences. Each
survivor is a CANDIDATE ORPHAN — a place that still quotes wording this change deleted.

⚠️ CANDIDATES, NOT DEFECTS. A survivor may be an unrelated legitimate use of the same
words. The output is a list to adjudicate, in the non-gating posture of the reverse-citation
audit — which is why the exit codes separate "found candidates" from "could not measure".

⚠️ A ZERO MUST NOT BE INDISTINGUISHABLE FROM A BROKEN RUN. Every invocation exercises a
POSITIVE CONTROL through the same extraction and search path: a phrase taken from the tree
at runtime, which must be found. If the control fails the tool reports COULD NOT MEASURE
(exit 2) rather than a clean zero. stderr is never discarded.

⚠️ AND IT MUST NOT BECOME THE THING IT CHECKS. Its parameters are PRINTED from the
constants at runtime rather than described in this docstring, so a changed threshold cannot
leave a stale description behind. The one number named in prose here -- none -- is
deliberate; `test_kiss_orphaned_citations.py` asserts that this docstring states no
threshold, so the documentation cannot drift from the code.

Run: python tools/kiss_orphaned_citations.py --base-ref origin/main
"""
import argparse
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

# A quoted phrase: backticked, double-quoted, or double-quoted inside markdown emphasis.
# Backticks dominate in this repo; the double-quoted forms carry the clause quotations.
QUOTE_PATTERNS = (
    ("backtick", re.compile(r"`([^`\n]{3,200})`")),
    ("double", re.compile(r'"([^"\n]{3,200})"')),
)

# A phrase must carry at least this many WORDS to be distinctive. Below it, `proven_set`
# and "yes" match everywhere and the report is noise.
MIN_WORDS = 4

# Directories whose contents are not prose citations of anything.
SKIP_DIRS = {".git", "target", "node_modules", "__pycache__", ".github/workflows"}
TEXT_EXT = {".rs", ".md", ".py", ".toml", ".yml", ".yaml", ".tsv", ".json"}


# ⚠️ WORDS, not whitespace-separated tokens. MEASURED on this tree: a plain token count
# admits `00 00 00 00`, `+ - * /` and `01 01 01 01` -- hex dumps and operator lists, which
# are not prose citations of anything and would have been most of the report. A word here
# must carry at least two ASCII letters.
RE_WORD = re.compile(r"[A-Za-z]{2,}")


def words(phrase):
    return len([w for w in re.split(r"\s+", phrase.strip()) if RE_WORD.search(w)])


def extract_phrases(line):
    """Every distinctive quoted phrase in one line, as (kind, phrase)."""
    out = []
    for kind, pat in QUOTE_PATTERNS:
        for m in pat.finditer(line):
            p = m.group(1).strip()
            if words(p) >= MIN_WORDS:
                out.append((kind, p))
    return out


def removed_lines(base_ref):
    """Lines the diff REMOVES, base_ref..worktree. Raises if git did not run."""
    r = subprocess.run(["git", "diff", "--unified=0", base_ref, "--"],
                       capture_output=True, text=True, cwd=ROOT, timeout=300)
    if r.returncode != 0:
        raise RuntimeError(f"git diff against {base_ref!r} failed (exit {r.returncode}):\n{r.stderr}")
    return [ln[1:] for ln in r.stdout.splitlines()
            if ln.startswith("-") and not ln.startswith("---")]


def tree_files():
    for dirpath, dirnames, filenames in os.walk(ROOT):
        rel = os.path.relpath(dirpath, ROOT).replace("\\", "/")
        dirnames[:] = [d for d in dirnames
                       if d not in SKIP_DIRS and f"{rel}/{d}".lstrip("./") not in SKIP_DIRS]
        for fn in filenames:
            if os.path.splitext(fn)[1] in TEXT_EXT:
                yield os.path.join(dirpath, fn)


def survivors(phrase, corpus):
    """(path, lineno) for every surviving occurrence of `phrase` in the tree."""
    hits = []
    for path, lines in corpus:
        for n, line in enumerate(lines, 1):
            if phrase in line:
                hits.append((os.path.relpath(path, ROOT).replace("\\", "/"), n))
    return hits


def load_corpus():
    corpus = []
    for path in tree_files():
        try:
            with open(path, encoding="utf-8") as fh:
                corpus.append((path, fh.read().splitlines()))
        except (OSError, UnicodeDecodeError):
            continue
    return corpus


def control_phrase(corpus):
    """A phrase taken FROM THE TREE at runtime, which the search must find.

    Derived rather than hardcoded so it cannot go stale, and drawn from the corpus the
    scan itself loaded, so it exercises the real search path rather than a local
    shortcut. Deliberately NOT tied to one path: a control that depends on a particular
    file being present reports COULD NOT MEASURE in any tree lacking it, which turns a
    working scanner into a permanent refusal.

    It is a control the search CAN fail: the phrase is known to be in the corpus, so an
    empty result means `survivors` is broken, which is exactly what it is here to catch.
    """
    for path, lines in sorted(corpus):
        for line in lines:
            for _kind, phrase in extract_phrases(line):
                return phrase
    return None


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--base-ref", required=True,
                    help="ref of the PRE-change state (e.g. origin/main)")
    args = ap.parse_args(argv)

    # Parameters PRINTED from the constants, never described in prose: a changed
    # threshold cannot leave a stale description behind (#407's own failure mode).
    print("-" * 68)
    print("  ORPHANED-CITATION CANDIDATES (#407)")
    print(f"  base-ref            {args.base_ref}")
    print(f"  min words / phrase  {MIN_WORDS}")
    print(f"  quote forms         {', '.join(k for k, _ in QUOTE_PATTERNS)}")

    corpus = load_corpus()
    print(f"  files scanned       {len(corpus)}")

    # ⚠️ POSITIVE CONTROL, in this invocation, through the same code path. A zero below
    # means "searched and found nothing" ONLY if this found something.
    ctrl = control_phrase(corpus)
    if ctrl is None:
        print("-" * 68)
        print("  COULD NOT MEASURE: no phrase could be extracted from the corpus at all,")
        print("  so the extractor is not known to work and a zero would be meaningless.")
        return 2
    ctrl_hits = survivors(ctrl, corpus)
    if not ctrl_hits:
        print("-" * 68)
        print(f"  COULD NOT MEASURE: the control phrase {ctrl!r} was extracted from the")
        print("  tree and then NOT FOUND in it. The search is broken, so a zero below")
        print("  would report success having searched nothing.")
        return 2
    print(f"  control             OK - {ctrl[:44]!r} found in {len(ctrl_hits)} place(s)")

    try:
        removed = removed_lines(args.base_ref)
    except RuntimeError as exc:
        print("-" * 68)
        print(f"  COULD NOT MEASURE: {exc}")
        return 2
    print(f"  removed lines       {len(removed)}")

    seen, candidates = set(), []
    for line in removed:
        for _kind, phrase in extract_phrases(line):
            if phrase in seen:
                continue
            seen.add(phrase)
            for path, lineno in survivors(phrase, corpus):
                candidates.append((phrase, path, lineno))

    print("-" * 68)
    if not candidates:
        print(f"  CLEAN - {len(seen)} distinct phrase(s) removed, none still quoted anywhere.")
        return 0

    print(f"  {len(candidates)} CANDIDATE(S) - wording this change removes is still quoted:")
    for phrase, path, lineno in candidates:
        print(f"      {path}:{lineno}")
        print(f"          still quotes: {phrase[:88]!r}")
    print()
    print("  These are CANDIDATES, not defects: a survivor may be an unrelated legitimate")
    print("  use of the same words. Adjudicate each -- update the citation, or confirm it")
    print("  does not refer to the wording this change deleted.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
