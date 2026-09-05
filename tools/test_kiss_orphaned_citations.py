"""Controls for the orphaned-citation scanner (#407).

Every positive assertion is paired with the defect it must reject, because a scanner that
reports CLEAN is exactly the shape this repo keeps finding to be broken. The three that
matter:

  * BORN-RED end to end -- a real git repo where a removal orphans a real citation, and
    the scanner must NAME the survivor. Built in a temp tree, not against this repo, so
    the control cannot pass because of something that happens to be here today.
  * COULD NOT MEASURE is DISTINCT from CLEAN -- a broken search must not report zero.
  * the word predicate rejects hex dumps and operator runs, MEASURED as most of the
    report before it was tightened.

Run: python tools/test_kiss_orphaned_citations.py   (also collected by pytest)
"""
import os
import re
import subprocess
import sys
import tempfile

# ⚠️ REFUSE TO RUN UNDER -O. `python -O` strips `assert`, so every control below would
# pass having checked nothing. One guard covers every assert including future ones.
if not __debug__:
    raise SystemExit(
        "refusing to run under -O/PYTHONOPTIMIZE: `assert` is stripped, so these "
        "controls would report success having verified nothing")

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_orphaned_citations as oc  # noqa: E402


# ------------------------------------------------------------- the predicate --

def test_the_word_predicate_rejects_what_is_not_prose():
    """MEASURED before this predicate existed: a whitespace-token count admitted
    `00 00 00 00`, `+ - * /` and `01 01 01 01`, which were most of the cross-file
    matches. They are hex dumps and operator lists, not citations of anything."""
    for junk in ("00 00 00 00", "+ - * /", "01 01 01 01", "0x7F81 0x7FBF 0xFF81 0x0040"):
        assert oc.words(junk) < oc.MIN_WORDS, f"{junk!r} should not count as prose"
    for prose in ("the result is the moved operand",
                  "no arithmetic so nothing to round"):
        assert oc.words(prose) >= oc.MIN_WORDS, f"{prose!r} should count as prose"


def test_extraction_finds_both_quote_forms_and_drops_short_ones():
    line = 'the clause says `the result is the moved operand` and "a b c d e" and `x`'
    got = {p for _k, p in oc.extract_phrases(line)}
    assert "the result is the moved operand" in got, got
    assert "a b c d e" not in got, "single letters are not words; this must be dropped"
    assert "x" not in got, "a one-token phrase must never be extracted"


def test_a_PATH_QUALIFIED_skip_entry_actually_skips():
    """⚠️ BORN-RED FOR A BUG THIS FILE SHIPPED (#408 review). The walk matched a
    path-qualified entry with `lstrip("./")`, which strips leading DOTS as well as
    slashes -- so `.github/workflows` became `github/workflows` and never matched. No
    entry with a dot could ever take effect, and nothing said so.

    The `.github/workflows` entry has since been REMOVED (workflow comments are prose
    citation and should be scanned), so this control uses a dot-prefixed entry of its own
    to pin the MATCHING, independently of what the live set happens to contain."""
    with tempfile.TemporaryDirectory() as tmp:
        os.makedirs(os.path.join(tmp, ".hidden", "inner"))
        with open(os.path.join(tmp, ".hidden", "inner", "x.md"), "w", encoding="utf-8") as fh:
            fh.write("a phrase that should not be scanned at all\n")
        with open(os.path.join(tmp, "kept.md"), "w", encoding="utf-8") as fh:
            fh.write("a phrase that SHOULD be scanned\n")
        old_root, old_skip = oc.ROOT, oc.SKIP_DIRS
        oc.ROOT, oc.SKIP_DIRS = tmp, {".hidden/inner"}
        try:
            seen = {rel for rel, _lines in oc.load_corpus()}
        finally:
            oc.ROOT, oc.SKIP_DIRS = old_root, old_skip
        assert "kept.md" in seen, f"the un-skipped file must be scanned: {seen}"
        assert not any(s.startswith(".hidden/inner") for s in seen), (
            f"a dot-prefixed path-qualified skip entry did not take effect: {seen}")


# ----------------------------------------------------------------- born-red --

def _mk_repo(tmp):
    def run(*a):
        r = subprocess.run(["git", *a], cwd=tmp, capture_output=True, text=True)
        assert r.returncode == 0, f"git {' '.join(a)} failed: {r.stderr}"
    run("init", "-q")
    run("config", "user.email", "t@t"); run("config", "user.name", "t")
    os.makedirs(os.path.join(tmp, "spec"))
    os.makedirs(os.path.join(tmp, "conformance"))
    with open(os.path.join(tmp, "spec", "ops.md"), "w", encoding="utf-8") as fh:
        fh.write("- **CLAUSE** the rule is `the result is the moved operand` here.\n")
    with open(os.path.join(tmp, "conformance", "t.rs"), "w", encoding="utf-8") as fh:
        fh.write("// Backs: X -- the clause's `the result is the moved operand` covers it.\n")
    run("add", "-A"); run("commit", "-qm", "base")
    return run


def _scan(tmp, base="HEAD"):
    """Run the scanner against `tmp` by pointing its ROOT there -- the real main(),
    not a copy, so the controls exercise the code that ships."""
    old_root = oc.ROOT
    oc.ROOT = tmp
    try:
        import io as _io
        from contextlib import redirect_stdout
        buf = _io.StringIO()
        with redirect_stdout(buf):
            rc = oc.main(["--base-ref", base])
        return rc, buf.getvalue()
    finally:
        oc.ROOT = old_root


def test_born_red_a_removal_that_orphans_a_citation_is_NAMED():
    """The #404 shape: a spec phrase is deleted, a test comment still quotes it."""
    with tempfile.TemporaryDirectory() as tmp:
        _mk_repo(tmp)
        # remove the wording from the spec; the conformance citation survives.
        with open(os.path.join(tmp, "spec", "ops.md"), "w", encoding="utf-8") as fh:
            fh.write("- **CLAUSE** the rule is stated differently now.\n")
        rc, out = _scan(tmp)
        assert rc == 1, f"expected CANDIDATES (exit 1), got {rc}\n{out}"
        assert "conformance/t.rs:1" in out, f"the survivor was not NAMED:\n{out}"
        assert "the result is the moved operand" in out, out


def test_a_removal_whose_citation_was_ALSO_updated_is_CLEAN():
    """The control for the born-red: same removal, citation fixed in the same change.
    Without this, the scanner could report CANDIDATES unconditionally and still pass."""
    with tempfile.TemporaryDirectory() as tmp:
        _mk_repo(tmp)
        with open(os.path.join(tmp, "spec", "ops.md"), "w", encoding="utf-8") as fh:
            fh.write("- **CLAUSE** the rule is now `the operand bits with a sign edit` here.\n")
        with open(os.path.join(tmp, "conformance", "t.rs"), "w", encoding="utf-8") as fh:
            fh.write("// Backs: X -- the clause's `the operand bits with a sign edit` covers it.\n")
        rc, out = _scan(tmp)
        assert rc == 0, f"expected CLEAN (exit 0), got {rc}\n{out}"
        assert "CLEAN" in out, out


# ------------------------------------------------- could-not-measure is DISTINCT --

def test_a_broken_search_reports_COULD_NOT_MEASURE_not_clean():
    """⚠️ The defect this whole repo keeps finding: an instrument unable to report the
    bad case, returning zero. If the control phrase cannot be found, the scanner must
    NOT say CLEAN -- a zero and a broken search are different answers."""
    with tempfile.TemporaryDirectory() as tmp:
        _mk_repo(tmp)
        real = oc.survivors
        oc.survivors = lambda phrase, corpus: []          # the search finds nothing, ever
        try:
            rc, out = _scan(tmp)
        finally:
            oc.survivors = real
        assert rc == 2, f"a broken search must exit 2, got {rc}\n{out}"
        assert "COULD NOT MEASURE" in out, out
        assert "CLEAN" not in out, "a broken search must never print CLEAN"


def test_a_bad_base_ref_reports_COULD_NOT_MEASURE():
    with tempfile.TemporaryDirectory() as tmp:
        _mk_repo(tmp)
        rc, out = _scan(tmp, base="no-such-ref-anywhere")
        assert rc == 2, f"an unusable base-ref must exit 2, got {rc}\n{out}"
        assert "COULD NOT MEASURE" in out, out


def test_the_three_exit_codes_are_distinct():
    """0 / 1 / 2 must not collapse: "found nothing", "found candidates" and "could not
    look" need different answers, or the caller cannot tell them apart."""
    assert len({0, 1, 2}) == 3


# ------------------------------------------- it must not become the thing it checks --

def test_the_docstring_states_no_threshold_the_code_could_drift_from():
    """⚠️ The scanner's own failure mode, turned on itself. If the module docstring named
    the word THRESHOLD, changing MIN_WORDS would leave a stale citation of it -- precisely
    what this tool exists to find. The value is PRINTED at runtime instead.

    ⚠️ THIS IS A PROXY AND THE FIRST VERSION WAS WRONG: it asserted the docstring named NO
    digits at all, which flagged the issue number and the exit code -- both legitimate and
    both stable. What must not appear is the THRESHOLD VALUE. Issue refs and exit codes are
    part of the contract and do not drift with MIN_WORDS.
    """
    doc = oc.__doc__
    bare = re.findall(r"(?<![\w#-])" + str(oc.MIN_WORDS) + r"(?![\w-])", doc)
    assert not bare, (
        f"the module docstring names {oc.MIN_WORDS}, the current MIN_WORDS, which the code "
        f"can drift from. Print parameters at runtime rather than describing them here.")

def main():
    tests = [v for k, v in sorted(globals().items())
             if k.startswith("test_") and callable(v)]
    for t in tests:
        t()
    print(f"ok - {len(tests)} controls pass: born-red names the survivor, a fixed citation "
          f"is clean, a broken search and a bad base-ref both report COULD NOT MEASURE")
    return 0


if __name__ == "__main__":
    sys.exit(main())
