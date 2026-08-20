"""Round-trip controls for the UNBACKED.tsv ledger writer (#272).

`write_ledger` promises in its own docstring that `--update-ledger` "never silently
drops a curated categorization." It broke that promise for the `untested` category:
an untested clause carrying a curated note (why it is untested, what would close it)
had its note written back EMPTY, so a routine `--update-ledger` erased the ledger's
institutional memory with no diff signal beyond the note column emptying.

The born-red control below fails on the pre-#272 writer and passes on the fix; the
paired control pins that a first-time-unbacked clause (no prior) still writes an empty
note, so the fix is "preserve a prior note," not "invent one."

Run: python tools/test_kiss_ledger.py
"""
import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt  # noqa: E402

CID = "KISS-CLASSIFY-6.1-0013"
NOTE = ("MX scale bit-layout lives in the §6.1 table + dtype_manifest.json, enforced by "
        "the kiss_dtypes table lint; candidate upgrade to lint:kiss_tables once kiss_dtypes "
        "declares this clause (Lane B).")


def _write_raw(path, rows):
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("# ledger\n")
        for cid, test, cat, note in rows:
            fh.write(f"{cid}\t{test}\t{cat}\t{note}\n")


def test_update_ledger_preserves_an_untested_note():
    """BORN RED before #272: a curated note on an `untested` row survives a re-write."""
    with tempfile.TemporaryDirectory() as d:
        led = os.path.join(d, "UNBACKED.tsv")
        _write_raw(led, [(CID, "test_classify_mx_scale_format", "untested", NOTE)])
        prior = kt.read_ledger(led)
        assert prior[CID]["note"] == NOTE, "read_ledger did not carry the note in"
        assert prior[CID]["category"] == "untested"
        # --update-ledger re-writes the clause, still unbacked, with `prior` as the source.
        kt.write_ledger(led, {CID: "test_classify_mx_scale_format"}, prior)
        after = kt.read_ledger(led)
        assert after[CID]["note"] == NOTE, (
            "write_ledger dropped a curated note on an untested row (#272) — "
            f"got {after[CID]['note']!r}")


def test_first_time_unbacked_writes_an_empty_note():
    """The paired control: no prior -> empty note, so the fix preserves rather than invents."""
    with tempfile.TemporaryDirectory() as d:
        led = os.path.join(d, "UNBACKED.tsv")
        kt.write_ledger(led, {"KISS-OPS-6.99-0001": "test_fixture"}, {})
        after = kt.read_ledger(led)
        assert after["KISS-OPS-6.99-0001"]["note"] == "", "a first-time row must have no note"


def test_a_lint_category_note_still_round_trips():
    """Regression guard on the branch the bug did NOT touch: a non-untested note is kept."""
    with tempfile.TemporaryDirectory() as d:
        led = os.path.join(d, "UNBACKED.tsv")
        _write_raw(led, [("KISS-OPS-6.8-0012", "t", "lint:kiss_comparators", "enforced by lint")])
        prior = kt.read_ledger(led)
        kt.write_ledger(led, {"KISS-OPS-6.8-0012": "t"}, prior)
        after = kt.read_ledger(led)
        assert after["KISS-OPS-6.8-0012"]["category"] == "lint"
        assert after["KISS-OPS-6.8-0012"]["note"] == "enforced by lint"


def main():
    test_update_ledger_preserves_an_untested_note()
    test_first_time_unbacked_writes_an_empty_note()
    test_a_lint_category_note_still_round_trips()
    print("ok - the ledger writer round-trips a curated note on every category, incl. untested")
    return 0


if __name__ == "__main__":
    sys.exit(main())
