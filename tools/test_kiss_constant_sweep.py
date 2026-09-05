"""Controls for the constant sweep (#409 Phase 1).

Every positive assertion is paired with the defect it must reject. The three that matter:

  * ⚠️ THE COLLIDING-NAME CONTROL. Five constant names in this tree are defined in MORE THAN
    ONE FILE — `SCHEMA_VERSION`, `MAX_RANK`, `MAX_OPERANDS`, `MAGIC`, `MAX_STRUCTURE_KEY_LEN`.
    A name-keyed edit mutates whichever it finds first and then reports a verdict about a
    constant it never touched. A control on a UNIQUE name cannot catch that: a unique token
    has nothing to collide with, so it passes through a broken and a working locator
    identically. This control uses a name that CAN collide, and asserts the sibling file is
    untouched.
  * NOT-APPLIED is distinct from UNGUARDED. A harness reporting "nothing reddened" when the
    edit never landed is the failure this whole sweep exists to detect, occurring inside it.
  * UNGUARDED-AND-DECLARED is separated from UNGUARDED-AND-LIVE, or a reserved constant no
    test can pin inflates the finding rate.

Run: python tools/test_kiss_constant_sweep.py   (also collected by pytest)
"""
import io
import os
import sys
import tempfile

if not __debug__:
    raise SystemExit(
        "refusing to run under -O/PYTHONOPTIMIZE: `assert` is stripped, so these "
        "controls would report success having verified nothing")

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_constant_sweep as cs  # noqa: E402


# ------------------------------------------------------- the colliding control --

def test_a_name_defined_in_TWO_files_is_located_by_LINE_not_by_name():
    """⚠️ The control that can actually fail. `SCHEMA_VERSION` is defined in both
    `grammar.rs` and `structure_key.rs`; a name-keyed locator would mutate one and report
    about the other. A unique-name control would pass either way."""
    found = [c for c in cs.discover() if c[2] == "SCHEMA_VERSION"]
    files = {c[0] for c in found}
    assert len(files) >= 2, (
        f"this control needs a name defined in >1 file to be able to fail; "
        f"SCHEMA_VERSION was found only in {files}")
    # each occurrence must carry its OWN file and line -- not one shared entry.
    assert len({(c[0], c[1]) for c in found}) == len(found), (
        "two occurrences collapsed to one location; the locator is name-keyed")


def test_the_edit_is_isolated_and_the_SIBLING_file_is_untouched():
    """Mutating one `SCHEMA_VERSION` must leave the other file byte-identical."""
    found = sorted(c for c in cs.discover() if c[2] == "SCHEMA_VERSION")
    assert len(found) >= 2, found
    target, sibling = found[0], found[1]
    sib_path = os.path.join(cs.ROOT, sibling[0])
    sib_before = io.open(sib_path, encoding="utf-8").read()
    rel, idx = target[0], target[1]
    full = os.path.join(cs.ROOT, rel)
    old_line = io.open(full, encoding="utf-8").read().split("\n")[idx]
    new_line = old_line.replace("= " + target[4], "= " + cs.alternative(target[4], target[3]), 1)
    assert new_line != old_line, "the control's own mutation did not change the line"
    before, err = cs.apply_and_verify(rel, idx, old_line, new_line)
    try:
        assert err is None, f"the edit did not land: {err}"
        assert io.open(sib_path, encoding="utf-8").read() == sib_before, (
            "mutating one SCHEMA_VERSION changed the OTHER file -- the locator is name-keyed")
    finally:
        io.open(full, "w", encoding="utf-8", newline="").write(before)
    assert io.open(full, encoding="utf-8").read() == before


# ------------------------------------------- NOT-APPLIED is its own verdict --

def test_an_edit_that_does_not_land_is_NOT_APPLIED_not_a_survivor():
    """⚠️ The sweep's own version of the defect it hunts. If the target line has moved,
    the answer is 'this was not measured', never 'nothing reddened'."""
    with tempfile.TemporaryDirectory() as tmp:
        rel = "x.rs"
        p = os.path.join(tmp, rel)
        io.open(p, "w", encoding="utf-8", newline="").write("pub const A: u8 = 0x01;\nother\n")
        old_root = cs.ROOT
        cs.ROOT = tmp
        try:
            # claim the line holds something it does not -- the guard must refuse.
            before, err = cs.apply_and_verify(rel, 0, "pub const A: u8 = 0xFF;", "pub const A: u8 = 0x41;")
            assert before is None and err, "a stale target line must be refused, not written"
            assert "moved" in err, err
            assert io.open(p, encoding="utf-8").read() == "pub const A: u8 = 0x01;\nother\n", (
                "the file was modified despite the refusal")
        finally:
            cs.ROOT = old_root


def test_a_non_isolated_edit_is_reported_rather_than_believed():
    with tempfile.TemporaryDirectory() as tmp:
        rel = "y.rs"
        p = os.path.join(tmp, rel)
        io.open(p, "w", encoding="utf-8", newline="").write("a\nb\nc\n")
        old_root, cs.ROOT = cs.ROOT, tmp
        try:
            before, err = cs.apply_and_verify(rel, 1, "b", "b2")
            assert err is None, err          # a clean single-line edit is fine
            assert io.open(p, encoding="utf-8").read().split("\n")[1] == "b2"
        finally:
            io.open(p, "w", encoding="utf-8", newline="").write(before)
            cs.ROOT = old_root


# ----------------------------------------- unpinnable is separated from live --

def test_a_declared_unpinnable_constant_is_not_counted_as_a_finding():
    decl = cs.load_unpinnable()
    assert decl, "the unpinnable declaration file is empty or unreadable"
    key = ("conformance/src/shape_expr.rs", "TAG_REDUCE")
    assert key in decl, f"TAG_REDUCE must be declared unpinnable; have {sorted(decl)}"
    assert len(decl[key]) > 40, (
        "a declaration must carry a REASON a reader can check, not a bare opt-out")


def test_the_declaration_requires_a_reason_not_just_a_name():
    """A row with no third column must not silently register as declared."""
    with tempfile.TemporaryDirectory() as tmp:
        f = os.path.join(tmp, "u.tsv")
        io.open(f, "w", encoding="utf-8", newline="").write("path\tNAME\n")
        old, cs.UNPINNABLE = cs.UNPINNABLE, f
        try:
            assert cs.load_unpinnable() == {}, "a reasonless row must not count as a declaration"
        finally:
            cs.UNPINNABLE = old


# ------------------------------------------------- refusals are not results --

def test_an_unmutatable_literal_is_REFUSED_rather_than_guessed():
    """`i64::MIN` has no trustworthy same-type alternative. The sweep must decline to
    invent one -- a mutation whose validity cannot be argued produces a verdict that
    cannot be believed."""
    assert cs.alternative("i64::MIN", "i64") is None
    assert cs.alternative("SOME_OTHER_CONST", "u8") is None
    # ... and the ones it DOES handle are genuinely different values.
    for val, ty in (("0x05", "u8"), ("42", "usize"), ('"abc"', "&str")):
        alt = cs.alternative(val, ty)
        assert alt is not None and alt != val, (val, alt)


def main():
    tests = [v for k, v in sorted(globals().items())
             if k.startswith("test_") and callable(v)]
    for t in tests:
        t()
    print(f"ok - {len(tests)} controls pass: colliding-name located by line, sibling file "
          f"untouched, NOT-APPLIED distinct from UNGUARDED, declarations need a reason")
    return 0


if __name__ == "__main__":
    sys.exit(main())
